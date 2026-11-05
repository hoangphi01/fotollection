---
name: pre-work
description: Start every Fotollection task the same way, before reading code or changing anything. Use at the start of every session and every new request (code, docs, website, release, debugging, research, benchmarks): read DOCS_GUIDE and LESSONS, find the rows and lessons that apply, check resources, and say which ones apply before working.
---

# Pre-work (Fotollection)

The owner's rule (2026-11-05): **read `docs/DOCS_GUIDE.md` and `docs/LESSONS.md` before
doing anything.** Most mistakes in this project were made before, and their rule is
already written down. This skill makes reading them the first step, not an afterthought
at commit time.

## 1. Read, in this order

1. **`docs/DOCS_GUIDE.md`, the whole page** (it is short). Pick the §2 rows that match
   the task. Most tasks match several: a feature is "behaviour", "UI text", "anything a
   user can see" and often "the website".
2. **`docs/LESSONS.md`, the whole page.** Note every entry whose area the task touches.
   Search it as well for the files, crates and words of the task
   (`grep -n -i "<word>" docs/LESSONS.md`).
3. **`docs/STATUS.md`, "Next, in order"**: the session plan. Take the first open box
   unless the owner asked for something else.
4. **Only what the task needs from [docs/README.md](../../../docs/README.md)'s map**:
   PRD for the FR, ARCHITECTURE for the module, STYLE_RULE §4 for any UI (then the
   `style-audit` skill), OPTIMIZATION §3 for any budget, ERROR_CONTROL for any error.

Never work from memory of these files: they change every session.

## 2. Say what applies, in a few lines

Before the first edit, tell the owner (in Vietnamese, per the memory):

- the DOCS_GUIDE rows that will need updating;
- the lessons that apply, by title, and what each means for this task;
- anything unclear that is the owner's decision (LESSONS: "Ask before building a layout
  the user described in words", "a constraint that costs users a feature is the
  owner's decision").

Keep it short. It is a checklist, not a report.

## 3. Rules that come up nearly every time

These are in LESSONS and DOCS_GUIDE; they are listed here because they are the ones
most often missed. The files win if they disagree.

- **Resources:** `scripts/resources.sh` before, during and after builds and tests.
  Free disk with `scripts/clean.sh` (whole profiles), **never by deleting part of a
  profile** such as `target/debug/incremental`; if that happened, `cargo clean -p` the
  crates of the workspace before the next debug build.
- **The dev app:** while it runs, wait for its rebuild before `cargo test` or clippy.
  Port 5174 is ours; 5173 belongs to another project, so never kill it.
- **Checks on their own:** never chain on a piped check (`| tail`). Chain with `&&`.
  Run `scripts/check-docs.sh` on its own, and `scripts/check-docs-with-code.sh
  origin/main..HEAD` before a push.
- **Collage mode:** every change is checked there too (two photos to a sheet with
  their own edits, a photo picked and none, the panel and the sheet's preview,
  postcards).
- **Locks and rayon:** never hold a lock while rayon work runs.
- **Fixtures:** make a fixture fail on the old code before trusting it.
- **Measure first:** time each stage, and check that the tool measures what the app does.
- **Commits:** Conventional Commits with the ID, no co-author line, docs in the same
  commit. Push only when the owner says so.

## 4. After the work

- A bug that took more than one look gets a LESSONS entry in the fix's commit.
- A rule the task added that this page lacks goes into DOCS_GUIDE §2 (its §4 asks for
  it), and here if it comes up often.
