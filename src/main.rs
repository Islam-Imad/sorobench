use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sorobench::corpus;
use sorobench::expectation::{self, Builtins};
use sorobench::testfile;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("list-tests") => list_tests(args.get(2).map(String::as_str)),
        Some("parse") => parse_cmd(args.get(2).map(String::as_str)),
        Some("run") => run_cmd(args.get(2).map(String::as_str)),
        Some("run-one") => run_one_json(args.get(2).map(String::as_str)),
        Some("exec-one") => exec_one_json(args.get(2).map(String::as_str)),
        Some("run-all") => run_all(args.get(2).map(String::as_str)),
        Some("-h") | Some("--help") | Some("help") => {
            usage(&mut std::io::stdout().lock());
            ExitCode::SUCCESS
        }
        _ => {
            usage(&mut std::io::stderr().lock());
            ExitCode::FAILURE
        }
    }
}

/// `sorobench list-tests [CORPUS_DIR]` — print every `*.sol` path.
fn list_tests(cli_root: Option<&str>) -> ExitCode {
    let root = corpus::corpus_root(cli_root);
    match corpus::enumerate(&root) {
        Ok(paths) => {
            for p in &paths {
                println!("{}", p.display());
            }
            eprintln!("{} test(s) under {}", paths.len(), root.display());
            if paths.len() != corpus::EXPECTED_COUNT {
                eprintln!(
                    "note: expected {} (pinned solc v0.8.22)",
                    corpus::EXPECTED_COUNT
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: walking {}: {e}", root.display());
            eprintln!("hint: set $SOROBENCH_CORPUS or pass the dir as an argument");
            ExitCode::FAILURE
        }
    }
}

fn parse_cmd(arg: Option<&str>) -> ExitCode {
    let builtins = expectation::semantic_test_builtins();
    if let Some(a) = arg {
        let p = PathBuf::from(a);
        if p.is_file() {
            return parse_one_verbose(&p, &builtins);
        }
    }
    let root = corpus::corpus_root(arg);
    let paths = match corpus::enumerate(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: walking {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    parse_corpus(&root, &paths, &builtins)
}

struct Tally {
    files: usize,
    no_block: usize,
    split_errors: Vec<(PathBuf, String)>,
    parse_errors: Vec<(PathBuf, String)>,
    parsed_ok: usize,
    total_calls: usize,
}

fn parse_corpus(root: &Path, paths: &[PathBuf], builtins: &Builtins) -> ExitCode {
    let mut t = Tally {
        files: paths.len(),
        no_block: 0,
        split_errors: Vec::new(),
        parse_errors: Vec::new(),
        parsed_ok: 0,
        total_calls: 0,
    };

    for path in paths {
        let text = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                t.split_errors.push((path.clone(), format!("read: {e}")));
                continue;
            }
        };
        let file = match testfile::split(&text) {
            Ok(f) => f,
            Err(e) => {
                t.split_errors.push((path.clone(), e.message));
                continue;
            }
        };
        let Some(block) = file.expectations.as_deref() else {
            t.no_block += 1;
            continue;
        };
        match expectation::parse_calls(block, builtins) {
            Ok(calls) => {
                t.parsed_ok += 1;
                t.total_calls += calls.len();
            }
            Err(e) => t.parse_errors.push((path.clone(), e.message)),
        }
    }

    let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).display().to_string();

    println!("sorobench parse — corpus expectation coverage");
    println!("  root:          {}", root.display());
    println!("  files:         {}", t.files);
    println!("  no // ---- :   {}", t.no_block);
    println!(
        "  parsed OK:     {}  ({} calls)",
        t.parsed_ok, t.total_calls
    );
    println!("  split errors:  {}", t.split_errors.len());
    println!("  parse errors:  {}", t.parse_errors.len());

    let show = |label: &str, errs: &[(PathBuf, String)]| {
        if errs.is_empty() {
            return;
        }
        println!("\n{label}:");
        for (p, why) in errs.iter().take(40) {
            println!("  {}: {}", rel(p), why);
        }
        if errs.len() > 40 {
            println!("  … and {} more", errs.len() - 40);
        }
    };
    show("SPLIT ERRORS", &t.split_errors);
    show("PARSE ERRORS", &t.parse_errors);

    let unparsed = t.split_errors.len() + t.parse_errors.len();
    let accounted = t.parsed_ok + t.no_block + unparsed;
    println!(
        "\n{} / {} files accounted for ({} parsed, {} no-block, {} unparsed-with-reason)",
        accounted, t.files, t.parsed_ok, t.no_block, unparsed
    );
    // Exit non-zero only if a file was left completely unaccounted (should never happen).
    if accounted == t.files {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse_one_verbose(path: &Path, builtins: &Builtins) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let file = match testfile::split(&text) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("split error: {}", e.message);
            return ExitCode::FAILURE;
        }
    };
    println!("sources: {}", file.sources.len());
    if !file.settings.is_empty() {
        println!("settings:");
        for (k, v) in &file.settings {
            println!("  {k}: {v}");
        }
    }
    let Some(block) = file.expectations.as_deref() else {
        println!("(no // ---- block)");
        return ExitCode::SUCCESS;
    };
    match expectation::parse_calls(block, builtins) {
        Ok(calls) => {
            println!("calls: {}", calls.len());
            for c in &calls {
                println!(
                    "  {:?}  {}  args={} exp={}{}{}",
                    c.kind,
                    c.signature,
                    c.arguments.parameters.len(),
                    c.expectations.result.len(),
                    if c.expectations.failure {
                        " FAILURE"
                    } else {
                        ""
                    },
                    if c.expected_side_effects.is_empty() {
                        String::new()
                    } else {
                        format!(" ~{}", c.expected_side_effects.len())
                    },
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("parse error: {}", e.message);
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "harness")]
fn run_cmd(arg: Option<&str>) -> ExitCode {
    use sorobench::harness::TIMEOUT;

    // Default target = the custom_tests/ focus directory.
    let target = PathBuf::from(arg.unwrap_or("custom_tests"));

    let exe = match own_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot find own executable to isolate tests: {e}");
            return ExitCode::FAILURE;
        }
    };

    if target.is_dir() {
        let files = match corpus::enumerate(&target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: reading {}: {e}", target.display());
                return ExitCode::FAILURE;
            }
        };
        if files.is_empty() {
            println!("(no .sol files under {})", target.display());
            return ExitCode::SUCCESS;
        }
        let (mut pass, mut fail, mut other) = (0, 0, 0);
        for path in &files {
            let rel = path
                .strip_prefix(&target)
                .unwrap_or(path)
                .display()
                .to_string();
            println!("===== {rel} =====");
            let abs = path.to_string_lossy().into_owned();
            let report = isolate_one(&exe, &abs, rel, TIMEOUT);
            let (p, f, o) = print_file_report(&report);
            pass += p;
            fail += f;
            other += o;
            println!();
        }
        println!(
            "TOTAL across {} file(s): {pass} pass, {fail} fail, {other} skipped/unsupported/nofaithful",
            files.len()
        );
        ExitCode::SUCCESS
    } else if target.is_file() {
        let abs = target.to_string_lossy().into_owned();
        let report = isolate_one(&exe, &abs, abs.clone(), TIMEOUT);
        print_file_report(&report);
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "error: {} is neither a file nor a directory",
            target.display()
        );
        eprintln!("usage: sorobench run [FILE.sol | DIR]   (default: custom_tests/)");
        ExitCode::FAILURE
    }
}

/// The corpus-relative path to the current executable, for spawning `exec-one`.
#[cfg(feature = "harness")]
fn own_exe() -> std::io::Result<String> {
    Ok(std::env::current_exe()?.to_string_lossy().into_owned())
}

/// Run one `.sol` file in an isolated subprocess (`exec-one`) under `timeout`,
/// and fold the outcome — success, timeout, crash — into a single [`FileReport`].
/// This is the one isolation path shared by `run`, `run-one`, and `run-all`.
#[cfg(feature = "harness")]
fn isolate_one(
    exe: &str,
    abs: &str,
    rel: String,
    timeout: std::time::Duration,
) -> sorobench::report::FileReport {
    use sorobench::harness::{run_isolated_timeout, Exit};
    use sorobench::report::{Bucket, FileReport, ReportKind};

    let timeout_secs = timeout.as_secs();
    let iso = match run_isolated_timeout(exe, ["exec-one", abs], timeout) {
        Ok(iso) => iso,
        Err(e) => {
            return FileReport::synthetic(
                rel,
                ReportKind::Crashed,
                Bucket::Crash,
                format!("spawn failed: {e}"),
            )
        }
    };

    match iso.exit {
        Exit::Code(0) => match serde_json::from_str::<FileReport>(iso.stdout.trim()) {
            Ok(mut r) => {
                r.path = rel;
                r
            }
            Err(e) => FileReport::synthetic(
                rel,
                ReportKind::Crashed,
                Bucket::Crash,
                format!("exec-one produced no valid record: {e}"),
            ),
        },
        Exit::Timeout => FileReport::synthetic(
            rel,
            ReportKind::TimedOut,
            Bucket::Timeout,
            format!("exceeded {timeout_secs}s"),
        ),
        Exit::Signal(sig) => FileReport::synthetic(
            rel,
            ReportKind::Crashed,
            Bucket::Crash,
            format!("killed by signal {sig}"),
        ),
        Exit::Code(n) => FileReport::synthetic(
            rel,
            ReportKind::Crashed,
            Bucket::Crash,
            format!("exec-one exited {n} without a record"),
        ),
        Exit::Unknown => FileReport::synthetic(
            rel,
            ReportKind::Crashed,
            Bucket::Crash,
            "exec-one ended in an unknown state".into(),
        ),
    }
}

/// Pretty-print one [`FileReport`] and return its (pass, fail, other) tally.
/// Non-`Ran` outcomes count as a single "other" so the `run` totals stay honest.
#[cfg(feature = "harness")]
fn print_file_report(r: &sorobench::report::FileReport) -> (usize, usize, usize) {
    use sorobench::report::ReportKind;

    match r.report {
        ReportKind::FrontendError => {
            println!("  FRONTEND-ERROR: {}", r.detail);
            (0, 0, 1)
        }
        ReportKind::NoExpectations => {
            println!("  (no // ---- block)");
            (0, 0, 1)
        }
        ReportKind::CompileFailed => {
            println!("  COMPILE-FAILED: {}", r.detail);
            (0, 0, 1)
        }
        ReportKind::Unsupported => {
            println!("  UNSUPPORTED: {}", r.detail);
            (0, 0, 1)
        }
        ReportKind::TimedOut => {
            println!("  TIMED-OUT: {}", r.detail);
            (0, 0, 1)
        }
        ReportKind::Crashed => {
            println!("  CRASHED: {}", r.detail);
            (0, 0, 1)
        }
        ReportKind::Ran => {
            for c in &r.calls {
                let tail = if c.detail.is_empty() {
                    String::new()
                } else {
                    format!("  — {}", c.detail)
                };
                println!("  {:<12} {}{}", c.verdict, c.sig, tail);
            }
            println!(
                "  {} pass, {} fail, {} skipped/unsupported/nofaithful",
                r.pass, r.fail, r.other
            );
            (r.pass, r.fail, r.other)
        }
    }
}

#[cfg(not(feature = "harness"))]
fn run_cmd(_arg: Option<&str>) -> ExitCode {
    eprintln!("`run` requires building with --features harness (needs the LLVM16 toolchain)");
    ExitCode::FAILURE
}

#[cfg(feature = "harness")]
fn run_one_json(arg: Option<&str>) -> ExitCode {
    use sorobench::harness::TIMEOUT;

    let Some(path) = arg else {
        eprintln!("usage: sorobench run-one <FILE.sol>");
        return ExitCode::FAILURE;
    };

    let exe = match own_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("run-one: cannot find own executable to isolate the test: {e}");
            return ExitCode::FAILURE;
        }
    };

    let report = isolate_one(&exe, path, path.to_string(), TIMEOUT);
    print_report_json("run-one", &report)
}

/// `sorobench exec-one <FILE>` — the internal leaf worker: run ONE test
/// in-process and print its single JSON [`FileReport`]. This is the unit that
/// `run`, `run-one`, and `run-all` each isolate in a subprocess under a timeout;
/// it is not meant to be invoked directly.
#[cfg(feature = "harness")]
fn exec_one_json(arg: Option<&str>) -> ExitCode {
    use sorobench::harness::run_source;
    use sorobench::report::FileReport;

    let Some(path) = arg else {
        eprintln!("usage: sorobench exec-one <FILE.sol>");
        return ExitCode::FAILURE;
    };

    let report = match std::fs::read_to_string(path) {
        Ok(text) => FileReport::from_run(path.to_string(), &run_source(&text)),
        Err(e) => FileReport::synthetic(
            path.to_string(),
            sorobench::report::ReportKind::FrontendError,
            sorobench::report::Bucket::FrontendError,
            format!("read: {e}"),
        ),
    };
    print_report_json("exec-one", &report)
}

/// Emit one [`FileReport`] as a single JSON line to stdout — the wire format that
/// `run-all` reads back. serde owns the escaping, so the record can never drift
/// from what the runner computed. `who` names the caller for the error message.
#[cfg(feature = "harness")]
fn print_report_json(who: &str, report: &sorobench::report::FileReport) -> ExitCode {
    match serde_json::to_string(report) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{who}: serialize failed: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "harness")]
fn run_all(arg: Option<&str>) -> ExitCode {
    use sorobench::harness::TIMEOUT;
    use sorobench::report::{render_markdown, summarize, Bucket, FileReport, RunMeta};

    let root = corpus::corpus_root(arg);
    let paths = match corpus::enumerate(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: walking {}: {e}", root.display());
            eprintln!("hint: set $SOROBENCH_CORPUS or pass the dir as an argument");
            return ExitCode::FAILURE;
        }
    };

    let exe = match own_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: cannot find own executable to spawn exec-one: {e}");
            return ExitCode::FAILURE;
        }
    };

    let timeout_secs = TIMEOUT.as_secs();

    let total = paths.len();
    eprintln!(
        "run-all: {total} test(s) under {} (timeout {timeout_secs}s each)",
        root.display()
    );

    let mut reports: Vec<FileReport> = Vec::with_capacity(total);
    for (i, path) in paths.iter().enumerate() {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let abs = path.to_string_lossy().into_owned();

        reports.push(isolate_one(&exe, &abs, rel, TIMEOUT));

        if (i + 1) % 50 == 0 || i + 1 == total {
            eprintln!("  {}/{} …", i + 1, total);
        }
    }

    // --- write artifacts ---
    let out_dir = PathBuf::from("report");
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("error: creating {}: {e}", out_dir.display());
        return ExitCode::FAILURE;
    }

    let jsonl_path = out_dir.join("results.jsonl");
    match write_jsonl(&jsonl_path, &reports) {
        Ok(()) => eprintln!("wrote {}", jsonl_path.display()),
        Err(e) => {
            eprintln!("error: writing {}: {e}", jsonl_path.display());
            return ExitCode::FAILURE;
        }
    }

    let summary = summarize(&reports);
    let meta = RunMeta {
        root: root.display().to_string(),
        timeout_secs,
    };
    let md = render_markdown(&reports, &summary, &meta);
    let md_path = out_dir.join("summary.md");
    if let Err(e) = std::fs::write(&md_path, &md) {
        eprintln!("error: writing {}: {e}", md_path.display());
        return ExitCode::FAILURE;
    }
    eprintln!("wrote {}", md_path.display());

    // --- console bucket summary ---
    println!("\nsorobench run-all — {} test(s)", summary.total.files);
    for b in Bucket::ORDER {
        let n = summary.total.count(b);
        if n > 0 {
            println!("  {:<14} {}", b.as_str(), n);
        }
    }
    println!(
        "  calls: {} pass, {} fail, {} other",
        summary.total.calls_pass, summary.total.calls_fail, summary.total.calls_other
    );
    ExitCode::SUCCESS
}

/// Write one JSON [`FileReport`] per line (`results.jsonl`).
#[cfg(feature = "harness")]
fn write_jsonl(path: &Path, reports: &[sorobench::report::FileReport]) -> std::io::Result<()> {
    use std::io::Write;
    let mut buf = String::new();
    for r in reports {
        match serde_json::to_string(r) {
            Ok(line) => {
                buf.push_str(&line);
                buf.push('\n');
            }
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
    std::fs::File::create(path)?.write_all(buf.as_bytes())
}

#[cfg(not(feature = "harness"))]
fn run_one_json(_arg: Option<&str>) -> ExitCode {
    eprintln!("`run-one` requires building with --features harness (needs the LLVM16 toolchain)");
    ExitCode::FAILURE
}

#[cfg(not(feature = "harness"))]
fn exec_one_json(_arg: Option<&str>) -> ExitCode {
    eprintln!("`exec-one` requires building with --features harness (needs the LLVM16 toolchain)");
    ExitCode::FAILURE
}

#[cfg(not(feature = "harness"))]
fn run_all(_arg: Option<&str>) -> ExitCode {
    eprintln!("`run-all` requires building with --features harness (needs the LLVM16 toolchain)");
    ExitCode::FAILURE
}

fn usage(w: &mut impl std::io::Write) {
    let _ = writeln!(
        w,
        "sorobench — solc semantic suite vs solang-on-Soroban\n\
         \n\
         USAGE:\n    \
             sorobench <COMMAND>\n\
         \n\
         COMMANDS:\n    \
             list-tests [DIR]   Enumerate the corpus (*.sol). Default DIR is the\n    \
                                pinned solang submodule; override with $SOROBENCH_CORPUS.\n    \
             parse [FILE|DIR]   Parse `// ----` blocks. A FILE prints its calls; a\n    \
                                DIR (default: corpus) prints a coverage report.\n    \
             run [FILE|DIR]     Run .sol test(s) end-to-end (parse→compile→invoke→\n    \
                                compare), one verdict per call. Each test runs in\n    \
                                an isolated subprocess under a 10s timeout. Default\n    \
                                DIR is custom_tests/. Needs --features harness.\n    \
             run-one <FILE>     Run ONE isolated test (10s timeout) and print a\n    \
                                single JSON record (FileReport) to stdout.\n    \
             run-all [DIR]      Run the whole corpus, each test isolated in a\n    \
                                subprocess under a 10s timeout, and write\n    \
                                report/results.jsonl + report/summary.md.\n    \
             help               Show this message."
    );
}
