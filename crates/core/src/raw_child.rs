// SPDX-License-Identifier: GPL-3.0-only
//! macOS's RAW develop (`ImageIO`) in a child process (ADR-0060, NFR-004).
//!
//! `ImageIO` keeps about 120 MB of dead memory a develop that nothing in process frees (not
//! `kCGImageSourceShouldCache`, `CGImageSourceRemoveCacheAtIndex` nor an autorelease pool;
//! OPTIMIZATION §5, 2026-11-20). The child is the app's own executable started again with
//! [`FLAG`]: it serves develops one after another, so `ImageIO`'s pipeline stays warm (the first
//! develop of a process takes about 600 ms, the next ones about 375), and it is replaced after
//! [`DEVELOPS_PER_CHILD`] develops or [`IDLE`] without one, which gives its memory back.
//!
//! The child's answer is untrusted input (SECURITY §3): every size is checked before anything
//! is allocated. A child that dies or answers wrongly while it develops a file fails that file
//! for `ImageIO`: the next decoder of `raw::decoders` takes it (our own develop, without the
//! system's lens correction), and `ImageIO` never runs on it in process, where it could take the
//! app down. Only a child that cannot start falls back to `ImageIO` in process.

#[cfg(target_os = "macos")]
use fotollection_oscodec::{OsCodecError, OsImage, RawKind};

/// The first argument that makes the app's executable a develop child.
pub const FLAG: &str = "--os-decode";

/// Serves develops when this process was started as a child ([`FLAG`] first), then returns
/// `true` and the caller exits; otherwise returns `false` at once. The first thing `main`
/// does, before any window, log or thread.
#[cfg(target_os = "macos")]
pub fn serve() -> bool {
    if std::env::args_os().nth(1).is_some_and(|a| a == FLAG) {
        imp::serve();
        return true;
    }
    false
}

/// Only macOS develops in a child.
#[cfg(not(target_os = "macos"))]
pub fn serve() -> bool {
    false
}

/// Develops through a child started from `exe` from now on. Without it (unit tests, tools)
/// macOS develops in process.
#[cfg(target_os = "macos")]
pub fn use_child(exe: std::path::PathBuf) {
    let _ = imp::EXE.set(exe);
}

/// Only macOS develops in a child.
#[cfg(not(target_os = "macos"))]
pub fn use_child(_exe: std::path::PathBuf) {}

/// How many develops a child has served in this process, for the tests and benches.
#[cfg(target_os = "macos")]
pub fn served() -> u64 {
    imp::SERVED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Only macOS develops in a child.
#[cfg(not(target_os = "macos"))]
pub fn served() -> u64 {
    0
}

/// The develop through the child, or `None` when there is none to use: no executable set, or
/// a new child could not start (logged once as `raw.child_failed`, ERR-RUS-0007).
#[cfg(target_os = "macos")]
pub(crate) fn decode(bytes: &[u8], kind: RawKind) -> Option<Result<OsImage, OsCodecError>> {
    imp::decode(bytes, kind)
}

#[cfg(target_os = "macos")]
mod imp {
    use std::io::{self, Read, Write};
    use std::path::PathBuf;
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Mutex, OnceLock, mpsc};
    use std::time::{Duration, Instant};

    use fotollection_oscodec::{Format, OsCodecError, OsImage, RawKind};

    use super::FLAG;
    use crate::codec::{MAX_EDGE, MAX_PIXELS};

    const MAGIC: &[u8; 4] = b"FOSD";
    const VERSION: u8 = 1;
    /// Develops a child serves before it is replaced. Each leaves about 120 MB behind, and
    /// starting a child again costs about 220 ms of warming up: at 4 a RAW export was 11.6 %
    /// slower than in process, so 12 (owner, 2026-11-21; OPTIMIZATION §5).
    pub(super) const DEVELOPS_PER_CHILD: u32 = 12;
    /// A child with no develop for this long exits, giving its memory back.
    pub(super) const IDLE: Duration = Duration::from_secs(30);
    /// A develop that takes longer has hung: its child is killed and the file fails.
    const TIMEOUT: Duration = Duration::from_secs(120);
    /// No input photo is larger (`import::MAX_FILE_BYTES`).
    const MAX_INPUT: u64 = crate::import::MAX_FILE_BYTES;
    const MAX_ICC: u32 = 4 << 20;
    const MAX_MESSAGE: u32 = 1024;
    const KINDS: [RawKind; 8] = [
        RawKind::Rw2,
        RawKind::Cr2,
        RawKind::Cr3,
        RawKind::Nef,
        RawKind::Arw,
        RawKind::Raf,
        RawKind::Orf,
        RawKind::Dng,
    ];

    /// The kind's byte on the wire, its place in [`KINDS`]; a new kind fails to build here.
    fn code(kind: RawKind) -> u8 {
        match kind {
            RawKind::Rw2 => 0,
            RawKind::Cr2 => 1,
            RawKind::Cr3 => 2,
            RawKind::Nef => 3,
            RawKind::Arw => 4,
            RawKind::Raf => 5,
            RawKind::Orf => 6,
            RawKind::Dng => 7,
        }
    }

    pub(super) static EXE: OnceLock<PathBuf> = OnceLock::new();
    pub(super) static SERVED: AtomicU64 = AtomicU64::new(0);
    /// The running child, owned by the develop gate's turn: at most one, its memory bounded
    /// by [`DEVELOPS_PER_CHILD`] develops (about 0.75 GB plus 120 MB each, ADR-0060); emptied
    /// when it is replaced, when it exits idle after [`IDLE`], and when the app quits (the
    /// child sees its stdin end).
    static CHILD: Mutex<Option<Worker>> = Mutex::new(None);
    static LOGGED: AtomicBool = AtomicBool::new(false);

    // ---- The child ----

    /// Answers requests until stdin ends (the app quit or replaced it) or [`IDLE`] passes.
    pub(super) fn serve() {
        // Seconds since `start` of the last request; `u64::MAX` while one is being served.
        static LAST: AtomicU64 = AtomicU64::new(0);
        let start = Instant::now();
        crate::supervise::spawn(
            "raw-child-idle",
            move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let last = LAST.load(Ordering::Relaxed);
                    if last != u64::MAX
                        && start.elapsed().as_secs().saturating_sub(last) >= IDLE.as_secs()
                    {
                        std::process::exit(0);
                    }
                }
            },
            || std::process::exit(0),
        );
        let mut input = io::stdin().lock();
        let mut output = io::stdout().lock();
        while let Ok(Some((seq, kind, bytes))) = request(&mut input) {
            LAST.store(u64::MAX, Ordering::Relaxed);
            // The answer starts as soon as the request is read: the parent then knows that a
            // child that dies from here on died on this file.
            let mut head = Vec::with_capacity(8);
            head.extend_from_slice(MAGIC);
            head.extend_from_slice(&seq.to_le_bytes());
            if output
                .write_all(&head)
                .and_then(|()| output.flush())
                .is_err()
            {
                return;
            }
            let decoded =
                fotollection_oscodec::decode(&bytes, Format::Raw(kind), crate::raw::LIMITS);
            drop(bytes);
            if reply(&mut output, &decoded).is_err() {
                return;
            }
            LAST.store(start.elapsed().as_secs(), Ordering::Relaxed);
        }
    }

    /// One request, or `None` at the end of stdin or on anything malformed.
    fn request(input: &mut impl Read) -> io::Result<Option<(u32, RawKind, Vec<u8>)>> {
        let mut head = [0u8; 4 + 1 + 4 + 1 + 8];
        if input.read_exact(&mut head).is_err() || &head[..4] != MAGIC || head[4] != VERSION {
            return Ok(None);
        }
        let seq = u32::from_le_bytes([head[5], head[6], head[7], head[8]]);
        let Some(&kind) = KINDS.get(usize::from(head[9])) else {
            return Ok(None);
        };
        let mut len = [0u8; 8];
        len.copy_from_slice(&head[10..18]);
        let len = u64::from_le_bytes(len);
        if len > MAX_INPUT {
            return Ok(None);
        }
        let mut bytes = vec![0u8; usize::try_from(len).map_err(|_| io::ErrorKind::InvalidData)?];
        input.read_exact(&mut bytes)?;
        Ok(Some((seq, kind, bytes)))
    }

    /// The rest of the answer: status, width, height, the extra length, then the payload.
    fn reply(output: &mut impl Write, decoded: &Result<OsImage, OsCodecError>) -> io::Result<()> {
        let (status, w, h, payload): (u8, u32, u32, Vec<&[u8]>) = match decoded {
            Ok(image) => (
                0,
                image.width,
                image.height,
                vec![
                    image.rgb.as_slice(),
                    image.icc.as_deref().unwrap_or_default(),
                ],
            ),
            Err(OsCodecError::Unsupported) => (1, 0, 0, vec![]),
            Err(OsCodecError::CodecMissing) => (2, 0, 0, vec![]),
            Err(OsCodecError::TooLarge(w, h)) => (3, *w, *h, vec![]),
            Err(OsCodecError::Decode(why)) => {
                let mut end = why.len().min(MAX_MESSAGE as usize);
                while !why.is_char_boundary(end) {
                    end -= 1;
                }
                (4, 0, 0, vec![&why.as_bytes()[..end]])
            }
        };
        let extra = payload.last().map_or(0, |p| p.len());
        let mut head = Vec::with_capacity(13);
        head.push(status);
        head.extend_from_slice(&w.to_le_bytes());
        head.extend_from_slice(&h.to_le_bytes());
        head.extend_from_slice(
            &u32::try_from(extra)
                .map_err(|_| io::ErrorKind::InvalidData)?
                .to_le_bytes(),
        );
        output.write_all(&head)?;
        for part in payload {
            output.write_all(part)?;
        }
        output.flush()
    }

    // ---- The app ----

    /// A running child and its pipes. Dropping it ends the child and waits for it.
    pub(super) struct Worker {
        child: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
        served: u32,
        seq: u32,
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    /// Why a request failed.
    #[derive(Debug, PartialEq, Eq)]
    pub(super) enum Failure {
        /// Before the child took the request: it had exited (idle) or could not start.
        NotTaken,
        /// It answered out of turn: the stream is broken before this file ran.
        OutOfTurn,
        /// After it took it: it died, hung or answered wrongly on this file.
        OnFile(String),
    }

    impl Worker {
        fn spawn(command: &mut Command) -> io::Result<Self> {
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;
            let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::ErrorKind::BrokenPipe.into());
            };
            Ok(Self {
                child,
                stdin,
                stdout,
                served: 0,
                seq: 0,
            })
        }

        /// One develop, with `timeout` before the child is killed.
        fn request(
            &mut self,
            bytes: &[u8],
            kind: RawKind,
            timeout: Duration,
        ) -> Result<Result<OsImage, OsCodecError>, Failure> {
            self.seq = self.seq.wrapping_add(1);
            let pid = self.child.id();
            let (done, watch) = mpsc::channel::<()>();
            // Kills by pid only, holding nothing, so a hung child cannot hold up the app. The
            // answer ends its wait at once. Only after a 120 s hang, and in the milliseconds
            // between the answer and its message, could the pid already be another process's.
            crate::supervise::spawn(
                "raw-child-watchdog",
                move || {
                    if watch.recv_timeout(timeout) == Err(mpsc::RecvTimeoutError::Timeout) {
                        let _ = Command::new("/bin/kill")
                            .args(["-9", &pid.to_string()])
                            .status();
                    }
                },
                || {},
            );
            let answer = self.exchange(bytes, kind);
            let _ = done.send(());
            answer
        }

        fn exchange(
            &mut self,
            bytes: &[u8],
            kind: RawKind,
        ) -> Result<Result<OsImage, OsCodecError>, Failure> {
            let Ok(len) = u64::try_from(bytes.len()) else {
                return Ok(Err(OsCodecError::TooLarge(u32::MAX, u32::MAX)));
            };
            let mut head = Vec::with_capacity(18);
            head.extend_from_slice(MAGIC);
            head.push(VERSION);
            head.extend_from_slice(&self.seq.to_le_bytes());
            head.push(code(kind));
            head.extend_from_slice(&len.to_le_bytes());
            let sent = self
                .stdin
                .write_all(&head)
                .and_then(|()| self.stdin.write_all(bytes))
                .and_then(|()| self.stdin.flush());
            let mut taken = [0u8; 8];
            if sent.is_err() || self.stdout.read_exact(&mut taken).is_err() {
                return Err(Failure::NotTaken);
            }
            // Bytes left over from an earlier answer: the stream is broken, not this file.
            if &taken[..4] != MAGIC || taken[4..] != self.seq.to_le_bytes() {
                return Err(Failure::OutOfTurn);
            }
            read_answer(&mut self.stdout).map_err(Failure::OnFile)
        }
    }

    /// The answer after it was taken, checked before anything is allocated.
    fn read_answer(out: &mut impl Read) -> Result<Result<OsImage, OsCodecError>, String> {
        let died = |_| "the develop's child stopped while developing this file".to_owned();
        let mut head = [0u8; 13];
        out.read_exact(&mut head).map_err(died)?;
        let word = |i: usize| u32::from_le_bytes([head[i], head[i + 1], head[i + 2], head[i + 3]]);
        let (w, h, extra) = (word(1), word(5), word(9));
        let read = |out: &mut dyn Read, n: u32| -> Result<Vec<u8>, String> {
            let mut v = vec![0u8; n as usize];
            out.read_exact(&mut v).map_err(died)?;
            Ok(v)
        };
        match head[0] {
            0 => {
                let pixels = u64::from(w) * u64::from(h);
                if w == 0
                    || h == 0
                    || w > MAX_EDGE
                    || h > MAX_EDGE
                    || pixels > MAX_PIXELS
                    || extra > MAX_ICC
                {
                    return Err("the develop's child answered an impossible size".into());
                }
                let mut rgb = vec![0u8; usize::try_from(pixels * 3).map_err(|_| "too large")?];
                out.read_exact(&mut rgb).map_err(died)?;
                let icc = (extra > 0).then(|| read(out, extra)).transpose()?;
                Ok(Ok(OsImage {
                    width: w,
                    height: h,
                    rgb,
                    icc,
                }))
            }
            1 if extra == 0 => Ok(Err(OsCodecError::Unsupported)),
            2 if extra == 0 => Ok(Err(OsCodecError::CodecMissing)),
            3 if extra == 0 => Ok(Err(OsCodecError::TooLarge(w, h))),
            4 if extra <= MAX_MESSAGE => Ok(Err(OsCodecError::Decode(
                String::from_utf8_lossy(&read(out, extra)?).into_owned(),
            ))),
            _ => Err("the develop's child answered something unknown".into()),
        }
    }

    pub(super) fn decode(bytes: &[u8], kind: RawKind) -> Option<Result<OsImage, OsCodecError>> {
        let exe = EXE.get()?;
        let answer = through(
            &CHILD,
            || {
                let mut command = Command::new(exe);
                command.arg(FLAG);
                command
            },
            bytes,
            kind,
            TIMEOUT,
        );
        if answer.is_some() {
            SERVED.fetch_add(1, Ordering::Relaxed);
        }
        answer
    }

    /// The develop through the child in `slot`, started with `command` when there is none.
    /// A child that had served before and did not take the request is replaced once (it may
    /// have just exited, idle); `None` only when a new child does not take it either.
    pub(super) fn through(
        slot: &Mutex<Option<Worker>>,
        command: impl Fn() -> Command,
        bytes: &[u8],
        kind: RawKind,
        timeout: Duration,
    ) -> Option<Result<OsImage, OsCodecError>> {
        let mut held = slot.lock().unwrap_or_else(|poisoned| {
            slot.clear_poison();
            let mut held = poisoned.into_inner();
            *held = None;
            held
        });
        for _ in 0..2 {
            let reused = held.is_some();
            let mut worker = match held.take() {
                Some(worker) => worker,
                None => match Worker::spawn(&mut command()) {
                    Ok(worker) => worker,
                    Err(e) => return failed(&format!("cannot start: {e}")),
                },
            };
            match worker.request(bytes, kind, timeout) {
                Ok(answer) => {
                    worker.served += 1;
                    if worker.served < DEVELOPS_PER_CHILD {
                        *held = Some(worker);
                    }
                    return Some(answer);
                }
                Err(Failure::NotTaken | Failure::OutOfTurn) if reused => {}
                Err(Failure::OutOfTurn) => {
                    return Some(Err(OsCodecError::Decode(
                        "the develop's child answered out of turn".into(),
                    )));
                }
                Err(Failure::NotTaken) => return failed("a new child did not take the develop"),
                Err(Failure::OnFile(why)) => return Some(Err(OsCodecError::Decode(why))),
            }
        }
        failed("a new child did not take the develop")
    }

    /// Logs the first failure of the session; the next develop tries a child again.
    fn failed(why: &str) -> Option<Result<OsImage, OsCodecError>> {
        if !LOGGED.swap(true, Ordering::Relaxed) {
            crate::error_log::record("raw.child_failed", why);
        }
        None
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// A fake child: `sh -c script`, whose output is what the parent reads.
        fn fake(script: &'static str) -> impl Fn() -> Command {
            move || {
                let mut c = Command::new("/bin/sh");
                c.args(["-c", script]);
                c
            }
        }

        fn run(script: &'static str, timeout: Duration) -> Option<Result<OsImage, OsCodecError>> {
            let slot = Mutex::new(None);
            through(&slot, fake(script), b"raw", RawKind::Rw2, timeout)
        }

        #[test]
        fn nfr_004_a_child_that_cannot_start_falls_back_in_process() {
            let slot = Mutex::new(None);
            let answer = through(
                &slot,
                || Command::new("/nonexistent/fotollection"),
                b"raw",
                RawKind::Rw2,
                Duration::from_secs(5),
            );
            assert!(answer.is_none());
        }

        #[test]
        fn nfr_004_a_child_that_exits_before_taking_the_develop_falls_back() {
            assert!(run("cat >/dev/null", Duration::from_secs(5)).is_none());
        }

        #[test]
        fn nfr_004_an_answer_out_of_turn_fails_the_file_not_the_app() {
            let answer = run(
                r"cat >/dev/null & printf 'FOSD\007\000\000\000'",
                Duration::from_secs(5),
            );
            assert!(
                matches!(answer, Some(Err(OsCodecError::Decode(_)))),
                "{answer:?}"
            );
        }

        #[test]
        fn nfr_004_an_impossible_size_is_refused_before_it_is_allocated() {
            // Status 0, 40,000 × 40,000: over MAX_EDGE.
            const SCRIPT: &str = concat!(
                r"head -c 21 >/dev/null; printf 'FOSD\001\000\000\000\000",
                r"\100\234\000\000\100\234\000\000\000\000\000\000'; sleep 5"
            );
            let answer = run(SCRIPT, Duration::from_secs(5));
            assert!(
                matches!(answer, Some(Err(OsCodecError::Decode(_)))),
                "{answer:?}"
            );
        }

        #[test]
        fn nfr_004_a_hung_develop_is_killed_and_fails_the_file() {
            let script = "head -c 21 >/dev/null; printf 'FOSD\\001\\000\\000\\000'; exec sleep 30";
            let start = Instant::now();
            let answer = run(script, Duration::from_millis(500));
            assert!(start.elapsed() < Duration::from_secs(10));
            assert!(
                matches!(answer, Some(Err(OsCodecError::Decode(_)))),
                "{answer:?}"
            );
        }

        #[test]
        fn nfr_004_unsupported_from_the_child_stays_unsupported() {
            // So the next decoder in `raw::decoders` takes the file, as in process.
            let answer = run(
                concat!(
                    r"head -c 21 >/dev/null; printf 'FOSD\001\000\000\000\001",
                    r"\000\000\000\000\000\000\000\000\000\000\000\000'"
                ),
                Duration::from_secs(5),
            );
            assert!(
                matches!(answer, Some(Err(OsCodecError::Unsupported))),
                "{answer:?}"
            );
        }

        #[test]
        fn nfr_004_a_child_gone_idle_is_replaced_without_falling_back() {
            // The first child serves one develop and exits, as on its idle timeout; the request
            // after it must go to a new child, not fall back in process.
            const ONE: &str = concat!(
                r"head -c 21 >/dev/null; printf 'FOSD\001\000\000\000\001",
                r"\000\000\000\000\000\000\000\000\000\000\000\000'"
            );
            let slot = Mutex::new(None);
            let first = through(
                &slot,
                fake(ONE),
                b"raw",
                RawKind::Rw2,
                Duration::from_secs(5),
            );
            assert!(matches!(first, Some(Err(OsCodecError::Unsupported))));
            std::thread::sleep(Duration::from_millis(200));
            // Its sequence restarts at 1 in the new child, which this script answers.
            let second = through(
                &slot,
                fake(ONE),
                b"raw",
                RawKind::Rw2,
                Duration::from_secs(5),
            );
            assert!(
                matches!(second, Some(Err(OsCodecError::Unsupported))),
                "{second:?}"
            );
        }

        #[test]
        fn nfr_004_kinds_round_trip_on_the_wire() {
            for kind in KINDS {
                assert_eq!(KINDS[usize::from(code(kind))], kind);
            }
        }

        #[test]
        fn nfr_004_stray_bytes_after_an_answer_do_not_fail_the_next_file() {
            // Answers request 1 with Unsupported plus four stray bytes, then waits for the next.
            const STRAY: &str = concat!(
                r#"while head -c 21 >/dev/null; do printf "FOSD\001\000\000\000\001"#,
                r#"\000\000\000\000\000\000\000\000\000\000\000\000XXXX"; done"#
            );
            let slot = Mutex::new(None);
            let first = through(
                &slot,
                fake(STRAY),
                b"raw",
                RawKind::Rw2,
                Duration::from_secs(5),
            );
            assert!(
                matches!(first, Some(Err(OsCodecError::Unsupported))),
                "{first:?}"
            );
            // Request 2 reads "XXXX…" as its head: the child is replaced, and the new one answers.
            let second = through(
                &slot,
                fake(STRAY),
                b"raw",
                RawKind::Rw2,
                Duration::from_secs(5),
            );
            assert!(
                matches!(second, Some(Err(OsCodecError::Unsupported))),
                "{second:?}"
            );
        }

        #[test]
        fn nfr_004_a_child_is_replaced_after_its_develops() {
            // Answers each request with its own sequence number (in octal, 1 to 255) and Unsupported.
            const SERVE: &str = concat!(
                r#"n=1; while head -c 21 >/dev/null; do s=$(printf '\\%03o' $n); "#,
                r#"printf "FOSD${s}\000\000\000\001\000\000\000\000\000\000\000\000\000\000\000\000"; "#,
                r#"n=$((n+1)); done"#
            );
            let slot = Mutex::new(None);
            for n in 1..=DEVELOPS_PER_CHILD {
                assert!(
                    through(
                        &slot,
                        fake(SERVE),
                        b"raw",
                        RawKind::Rw2,
                        Duration::from_secs(5)
                    )
                    .is_some()
                );
                let kept = slot.lock().is_ok_and(|s| s.is_some());
                assert_eq!(kept, n < DEVELOPS_PER_CHILD, "after {n} develops");
            }
        }
    }
}
