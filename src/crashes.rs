use crate::ledger::{self, Cause, LedgerText};
use crate::report::FileReport;

fn canonicalize(msg: &str) -> String {
    ledger::canonicalize(msg, false)
}

const HARNESS_MARKERS: &[&str] = &["spawn failed", "produced no valid record", "unknown state"];

fn classify(detail: &str) -> (String, String, bool) {
    let d = detail.trim();

    let (canon, noise) = if HARNESS_MARKERS.iter().any(|m| d.contains(m)) {
        let key = d.split_once(": ").map(|(w, _)| w.trim()).unwrap_or(d);
        (canonicalize(key), true)
    } else {
        match d.split_once(": ") {
            // A message-bearing crash: cluster on the panic message itself.
            Some((_wrapper, msg)) => (canonicalize(msg.trim()), false),
            // No message (a bare signal / exit code): keep the wrapper verbatim so
            // signal 11 and signal 6 stay distinct causes.
            None => (d.to_string(), false),
        }
    };

    (canon, detail.to_string(), noise)
}

pub fn cluster(reports: &[FileReport]) -> Vec<Cause> {
    ledger::cluster(reports, crate::report::Bucket::Crash, classify)
}

const TEXT: LedgerText = LedgerText {
    title: "solang crash",
    command: "crashes",
    blurb: "A **CRASH** is a file whose test process died — a signal (SIGSEGV/SIGABRT), \
            a solang panic mid-compile (ICE), or a non-zero exit with no record — rather \
            than failing a check or compile-erroring cleanly. Each is a solang robustness \
            bug: it should reject cleanly, not crash.",
    total_label: "CRASH files",
    causes_label: "distinct crash messages",
    noise_label: "sorobench plumbing failures (not a solang crash)",
    table_heading: "Crashes (by file count)",
    cause_col: "crash message",
    noise_tag: " *(sorobench plumbing failure — not a solang crash)*",
};

pub fn render_markdown(causes: &[Cause], source: &str) -> String {
    ledger::render_markdown(causes, source, &TEXT)
}
