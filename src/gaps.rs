pub use crate::ledger::read_reports;
use crate::ledger::{self, Cause, LedgerText};
use crate::report::FileReport;

pub fn atoms(detail: &str) -> Vec<String> {
    let body = detail.strip_prefix("solang:").unwrap_or(detail).trim();
    let mut seen = Vec::new();
    for part in body.split(';') {
        let a = part.trim();
        if !a.is_empty() && !seen.iter().any(|s| s == a) {
            seen.push(a.to_string());
        }
    }
    seen
}

const HARNESS_MARKER: &str = "file not found";
/// The single canonical key every multi-file test collapses under.
const HARNESS_CANON: &str =
    "file not found 'X' (multi-file import — sorobench limitation, not a solang gap)";

/// Map one GAP `detail` to its `(cluster key, this file's raw diagnostic, harness_noise)`.
fn classify(detail: &str) -> (String, String, bool) {
    let parts = atoms(detail);
    if let Some(nf) = parts.iter().find(|a| a.contains(HARNESS_MARKER)) {
        (HARNESS_CANON.to_string(), nf.clone(), true)
    } else {
        match parts.first() {
            Some(primary) => (ledger::canonicalize(primary, true), primary.clone(), false),
            None => ("(no detail)".to_string(), "(no detail)".to_string(), false),
        }
    }
}

pub fn cluster(reports: &[FileReport]) -> Vec<Cause> {
    ledger::cluster(reports, crate::report::Bucket::Gap, classify)
}

const TEXT: LedgerText = LedgerText {
    title: "solang gap",
    command: "gaps",
    blurb: "A **GAP** is a file solang rejected with a *clean* compile error on portable \
            source — not filtered (no EVM-only feature), not a crash, not a timeout. \
            Soroban can express these, so each is a solang TODO.",
    total_label: "GAP files",
    causes_label: "distinct solang root causes",
    noise_label: "multi-file-import files (sorobench limitation, not a solang gap)",
    table_heading: "Major gaps (by file count)",
    cause_col: "root cause",
    noise_tag: " *(sorobench multi-file limitation — not a solang gap)*",
};

pub fn render_markdown(causes: &[Cause], source: &str) -> String {
    ledger::render_markdown(causes, source, &TEXT)
}
