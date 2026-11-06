// SPDX-License-Identifier: GPL-3.0-only
//! What this computer is, for the Settings window's *For this computer* (FR-086): its
//! processor and cores. Read from what the system already says, without `unsafe`: `sysctl`
//! on macOS, the registry through `reg` on Windows, `/proc/cpuinfo` on Linux.

/// The processor's name as the system gives it, or `None` when it does not say.
pub fn cpu_name() -> Option<String> {
    #[cfg(target_os = "macos")]
    let name = {
        let out = std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()?;
        String::from_utf8(out.stdout).ok()?
    };
    #[cfg(target_os = "windows")]
    let name = {
        let out = std::process::Command::new("reg")
            .args([
                "query",
                r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
                "/v",
                "ProcessorNameString",
            ])
            .output()
            .ok()?;
        // `    ProcessorNameString    REG_SZ    <name>`
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.lines()
            .find_map(|l| l.split_once("REG_SZ").map(|(_, n)| n.to_owned()))?
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let name = {
        let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        info.lines()
            .find_map(|l| l.strip_prefix("model name"))
            .and_then(|l| l.split_once(':'))
            .map(|(_, n)| n.to_owned())?
    };
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    (!name.is_empty()).then_some(name)
}

/// The cores the app can run on at once.
pub fn cores() -> u32 {
    std::thread::available_parallelism()
        .ok()
        .and_then(|n| u32::try_from(n.get()).ok())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The machine running the tests says what it is: at least one core, and a name with
    /// no runs of spaces when it gives one.
    #[test]
    fn fr_086_this_computer_says_what_it_is() {
        assert!(cores() >= 1);
        if let Some(name) = cpu_name() {
            assert!(!name.contains("  ") && name == name.trim(), "{name:?}");
        }
    }
}
