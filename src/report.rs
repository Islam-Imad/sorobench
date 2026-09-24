use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    /// Compiled, deployed, and every `// ----` call ran (see per-call verdicts).
    Ran,
    /// Clean compile-fail on portable source -> a solang gap to fix.
    Gap,
    /// Clean compile-fail whose source uses an EVM feature Soroban can't express
    /// → excluded, not a solang failure (spec §2.2, §5).
    Filtered,
    /// Region-split or `// ----` parse failed (a bug in this tool, not solang).
    FrontendError,
    /// No `// ----` block — nothing to run.
    NoExpectations,
    /// A whole-file limitation of the runner (e.g. the constructor needs args).
    Unsupported,
    Crashed,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bucket {
    /// Ran; ≥1 checked pass and no fails and no other outcomes.
    #[serde(rename = "PASS_ALL")]
    PassAll,
    /// Ran; ≥1 checked pass, no fails, but some calls were skipped/nofaithful.
    #[serde(rename = "PASS_SOME")]
    PassSome,
    /// Ran; ≥1 checked fail (mismatch / trap / no-revert) — a likely solang bug.
    #[serde(rename = "HAS_FAIL")]
    HasFail,
    /// Ran; no call was ever checked (all skipped / unsupported / nofaithful).
    #[serde(rename = "ONLY_OTHER")]
    OnlyOther,
    /// Clean compile-fail, source portable → solang gap to fix.
    #[serde(rename = "GAP")]
    Gap,
    /// Clean compile-fail, source uses an EVM-only feature → excluded.
    #[serde(rename = "FILTERED")]
    Filtered,
    #[serde(rename = "FRONTEND_ERROR")]
    FrontendError,
    #[serde(rename = "UNSUPPORTED")]
    Unsupported,
    #[serde(rename = "NO_BLOCK")]
    NoBlock,
    #[serde(rename = "CRASH")]
    Crash,
    #[serde(rename = "TIMEOUT")]
    Timeout,
}

impl Bucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Bucket::PassAll => "PASS_ALL",
            Bucket::PassSome => "PASS_SOME",
            Bucket::HasFail => "HAS_FAIL",
            Bucket::OnlyOther => "ONLY_OTHER",
            Bucket::Gap => "GAP",
            Bucket::Filtered => "FILTERED",
            Bucket::FrontendError => "FRONTEND_ERROR",
            Bucket::Unsupported => "UNSUPPORTED",
            Bucket::NoBlock => "NO_BLOCK",
            Bucket::Crash => "CRASH",
            Bucket::Timeout => "TIMEOUT",
        }
    }

    pub const ORDER: [Bucket; 11] = [
        Bucket::PassAll,
        Bucket::PassSome,
        Bucket::HasFail,
        Bucket::OnlyOther,
        Bucket::Gap,
        Bucket::Filtered,
        Bucket::Unsupported,
        Bucket::NoBlock,
        Bucket::FrontendError,
        Bucket::Crash,
        Bucket::Timeout,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallReport {
    pub sig: String,
    // PASS / MISMATCH / TRAP
    pub verdict: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    /// The test path (relative to the corpus root when produced by `run-all`).
    pub path: String,
    pub report: ReportKind,
    pub bucket: Bucket,
    pub pass: usize,
    pub fail: usize,
    pub other: usize,
    /// Whole-file detail (compile-error message, crash signal, …); may be empty.
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub calls: Vec<CallReport>,
}

impl FileReport {
    pub fn synthetic(path: String, report: ReportKind, bucket: Bucket, detail: String) -> Self {
        FileReport {
            path,
            report,
            bucket,
            pass: 0,
            fail: 0,
            other: 0,
            detail,
            calls: Vec::new(),
        }
    }
}

/// Bridge the runner's `RunReport` into the wire struct. `harness`-only because
/// `RunReport` lives behind that feature; the struct above is feature-free.
#[cfg(feature = "harness")]
impl FileReport {
    pub fn from_run(path: String, run: &crate::harness::RunReport) -> Self {
        use crate::harness::RunReport;

        let (report, bucket, detail, pass, fail, other, calls) = match run {
            RunReport::FrontendError(e) => (
                ReportKind::FrontendError,
                Bucket::FrontendError,
                e.clone(),
                0,
                0,
                0,
                Vec::new(),
            ),
            RunReport::NoExpectations => (
                ReportKind::NoExpectations,
                Bucket::NoBlock,
                String::new(),
                0,
                0,
                0,
                Vec::new(),
            ),
            RunReport::Gap(e) => (ReportKind::Gap, Bucket::Gap, e.clone(), 0, 0, 0, Vec::new()),
            RunReport::Filtered { feature, note } => (
                ReportKind::Filtered,
                Bucket::Filtered,
                format!("{feature}: {note}"),
                0,
                0,
                0,
                Vec::new(),
            ),
            RunReport::Crashed(e) => (
                ReportKind::Crashed,
                Bucket::Crash,
                e.clone(),
                0,
                0,
                0,
                Vec::new(),
            ),
            RunReport::Unsupported(e) => (
                ReportKind::Unsupported,
                Bucket::Unsupported,
                e.clone(),
                0,
                0,
                0,
                Vec::new(),
            ),
            RunReport::Ran(verdicts) => {
                let (mut pass, mut fail, mut other) = (0, 0, 0);
                let calls: Vec<CallReport> = verdicts
                    .iter()
                    .map(|cv| {
                        if cv.verdict.is_pass() {
                            pass += 1;
                        } else if cv.verdict.is_fail() {
                            fail += 1;
                        } else {
                            other += 1;
                        }
                        CallReport {
                            sig: cv.signature.clone(),
                            verdict: cv.verdict.label().to_string(),
                            detail: cv.verdict.detail(),
                        }
                    })
                    .collect();
                let bucket = if fail > 0 {
                    Bucket::HasFail
                } else if pass > 0 && other == 0 {
                    Bucket::PassAll
                } else if pass > 0 {
                    Bucket::PassSome
                } else {
                    Bucket::OnlyOther
                };
                (
                    ReportKind::Ran,
                    bucket,
                    String::new(),
                    pass,
                    fail,
                    other,
                    calls,
                )
            }
        };

        FileReport {
            path,
            report,
            bucket,
            pass,
            fail,
            other,
            detail,
            calls,
        }
    }
}

/// A rollup of one group of files (the whole corpus, or one sub-directory).
#[derive(Debug, Clone, Default)]
pub struct Rollup {
    pub files: usize,
    pub buckets: std::collections::BTreeMap<&'static str, usize>,
    pub calls_pass: usize,
    pub calls_fail: usize,
    pub calls_other: usize,
}

impl Rollup {
    fn add(&mut self, r: &FileReport) {
        self.files += 1;
        *self.buckets.entry(r.bucket.as_str()).or_insert(0) += 1;
        self.calls_pass += r.pass;
        self.calls_fail += r.fail;
        self.calls_other += r.other;
    }

    pub fn count(&self, b: Bucket) -> usize {
        self.buckets.get(b.as_str()).copied().unwrap_or(0)
    }
}

/// The full corpus summary.
pub struct Summary {
    pub total: Rollup,
    /// Rollups keyed by top-level sub-directory of the corpus.
    pub by_dir: std::collections::BTreeMap<String, Rollup>,
}

/// First path component (the semanticTests sub-directory). A file directly under
/// the corpus root (no `/`) groups under `"(root)"`.
fn top_dir(path: &str) -> String {
    let norm = path.replace('\\', "/");
    match norm.split_once('/') {
        Some((dir, _)) if !dir.is_empty() => dir.to_string(),
        _ => "(root)".to_string(),
    }
}

pub fn summarize(reports: &[FileReport]) -> Summary {
    let mut total = Rollup::default();
    let mut by_dir: std::collections::BTreeMap<String, Rollup> = std::collections::BTreeMap::new();
    for r in reports {
        total.add(r);
        by_dir.entry(top_dir(&r.path)).or_default().add(r);
    }
    Summary { total, by_dir }
}

/// Render the human-facing Markdown report.
pub fn render_markdown(reports: &[FileReport], summary: &Summary, meta: &RunMeta) -> String {
    use std::fmt::Write;
    let t = &summary.total;
    let mut s = String::new();

    let _ = writeln!(s, "# sorobench — corpus report\n");
    let _ = writeln!(s, "- corpus root: `{}`", meta.root);
    let _ = writeln!(s, "- files run: **{}**", t.files);
    let _ = writeln!(s, "- per-test timeout: {}s", meta.timeout_secs);
    let _ = writeln!(s, "- generated by: `sorobench run-all`\n");

    // Headline (spec §5): the denominator is CANDIDATES, not "whatever ran".
    // filtered tests are excluded as platform-mismatch; every remaining failure
    // is attributable to solang. GAP = fail ∪ non-filtered clean compile-fail.
    let filtered = t.count(Bucket::Filtered);
    let housekeeping = t.count(Bucket::NoBlock) + t.count(Bucket::FrontendError);
    let candidates = t.files.saturating_sub(filtered + housekeeping);
    let pass = t.count(Bucket::PassAll) + t.count(Bucket::PassSome);
    let gaps = t.count(Bucket::HasFail) + t.count(Bucket::Gap);
    let crashes = t.count(Bucket::Crash);
    let timeouts = t.count(Bucket::Timeout);
    let _ = writeln!(s, "## Headline\n");
    if candidates > 0 {
        let pct = 100.0 * pass as f64 / candidates as f64;
        let _ = writeln!(
            s,
            "Of **{candidates}** candidates ({} − {filtered} filtered − {housekeeping} housekeeping), \
             **{pass} ({pct:.1}%)** pass; **{gaps}** gaps (mismatch/trap + non-filtered \
             compile-fail) — solang's TODO list; **{crashes}** crashes and **{timeouts}** timeouts \
             (solang should reject cleanly / not hang).",
            t.files
        );
    } else {
        let _ = writeln!(s, "No candidate files (all filtered or housekeeping).");
    }
    let _ = writeln!(
        s,
        "\n**{filtered}** tests excluded (EVM-only, Soroban platform can't express — see the \
         exclusion ledger below).",
    );
    let _ = writeln!(
        s,
        "\nCall-level across ran files: **{} pass**, **{} fail**, {} other \
         (skipped/nofaithful/unsupported).\n",
        t.calls_pass, t.calls_fail, t.calls_other
    );

    // Bucket table.
    let _ = writeln!(s, "## Buckets (file-level)\n");
    let _ = writeln!(s, "| bucket | files | % |");
    let _ = writeln!(s, "|---|---:|---:|");
    for b in Bucket::ORDER {
        let n = t.count(b);
        if n == 0 {
            continue;
        }
        let pct = 100.0 * n as f64 / t.files.max(1) as f64;
        let _ = writeln!(s, "| {} | {} | {:.1}% |", b.as_str(), n, pct);
    }
    let _ = writeln!(s);

    // Per-directory breakdown.
    let _ = writeln!(s, "## By directory\n");
    let _ = writeln!(
        s,
        "| dir | files | pass_all | pass_some | has_fail | gap | filtered | other |"
    );
    let _ = writeln!(s, "|---|---:|---:|---:|---:|---:|---:|---:|");
    for (dir, r) in &summary.by_dir {
        let other = r.files
            - r.count(Bucket::PassAll)
            - r.count(Bucket::PassSome)
            - r.count(Bucket::HasFail)
            - r.count(Bucket::Gap)
            - r.count(Bucket::Filtered);
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            dir,
            r.files,
            r.count(Bucket::PassAll),
            r.count(Bucket::PassSome),
            r.count(Bucket::HasFail),
            r.count(Bucket::Gap),
            r.count(Bucket::Filtered),
            other,
        );
    }
    let _ = writeln!(s);

    // Detail lists — the actionable part.
    section(
        &mut s,
        "Failures (mismatch / trap)",
        reports,
        Bucket::HasFail,
        60,
    );
    section(
        &mut s,
        "Crashes (isolated in a subprocess)",
        reports,
        Bucket::Crash,
        60,
    );
    section(&mut s, "Timeouts", reports, Bucket::Timeout, 60);
    section(
        &mut s,
        "Gaps — clean compile-fail, portable source (solang TODO)",
        reports,
        Bucket::Gap,
        60,
    );
    section(
        &mut s,
        "Exclusion ledger — filtered (EVM-only, platform can't express)",
        reports,
        Bucket::Filtered,
        200,
    );

    s
}

/// One "list files in this bucket, with detail" section.
fn section(s: &mut String, title: &str, reports: &[FileReport], bucket: Bucket, limit: usize) {
    use std::fmt::Write;
    let hits: Vec<&FileReport> = reports.iter().filter(|r| r.bucket == bucket).collect();
    if hits.is_empty() {
        return;
    }
    let _ = writeln!(s, "## {} — {} file(s)\n", title, hits.len());
    for r in hits.iter().take(limit) {
        // Prefer a failing call's detail; else the file-level detail.
        let detail = r
            .calls
            .iter()
            .find(|c| c.verdict != "PASS" && c.verdict != "PASS(revert)" && !c.detail.is_empty())
            .map(|c| format!("{}: {}", c.verdict, c.detail))
            .unwrap_or_else(|| r.detail.clone());
        let detail = detail.replace('\n', " ");
        let detail = if detail.len() > 160 {
            format!("{}…", &detail[..160])
        } else {
            detail
        };
        if detail.is_empty() {
            let _ = writeln!(s, "- `{}`", r.path);
        } else {
            let _ = writeln!(s, "- `{}` — {}", r.path, detail);
        }
    }
    if hits.len() > limit {
        let _ = writeln!(s, "- … and {} more", hits.len() - limit);
    }
    let _ = writeln!(s);
}

/// Metadata threaded into the rendered report.
pub struct RunMeta {
    pub root: String,
    pub timeout_secs: u64,
}
