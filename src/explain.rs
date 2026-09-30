//! Explain a finished run: match every failure against the explanation
//! dictionary (`dictionary/soroban.toml`) and write a report anyone can read.
//!
//! The dictionary maps failure patterns to a category (grounded in Solang's own
//! Soroban docs), a plain-language meaning, and a suggested fix. A failure no
//! rule explains is listed as "needs review", never assumed to be a bug.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use serde::Deserialize;

use crate::report::{Bucket, FileReport};

/// The dictionary shipped with sorobench (used when no path is given).
pub const DEFAULT_DICTIONARY: &str = include_str!("../dictionary/soroban.toml");

#[derive(Debug, Clone, Deserialize)]
pub struct Dictionary {
    pub rule: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub kind: Vec<String>,
    #[serde(default, rename = "match")]
    pub all: Vec<String>,
    #[serde(default)]
    pub any: Vec<String>,
    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default)]
    pub sig: Vec<String>,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub narrow_int: bool,
    #[serde(default)]
    pub handle_bytes: bool,
    #[serde(default)]
    pub other_target: Option<String>,
    pub category: String,
    pub title: String,
    pub meaning: String,
    pub suggestion: String,
    #[serde(default)]
    pub docs: String,
}

pub fn parse(text: &str) -> Result<Dictionary, String> {
    toml::from_str(text).map_err(|e| e.to_string())
}

/// Categories in report order (most actionable first), with a one-line meaning.
pub const CATEGORIES: &[(&str, &str)] = &[
    (
        "bug",
        "Violates Solang's documented behaviour: compiler crashes and wrong results",
    ),
    (
        "review",
        "Not explained yet: needs a human to check it against the docs",
    ),
    (
        "soroban-gap",
        "Missing on Soroban, not documented, works on Solang's other targets",
    ),
    (
        "solang-gap",
        "Rejected by Solang on every target: a general Solang limitation",
    ),
    (
        "documented-unsupported",
        "Solang's docs list the feature as not supported on Soroban",
    ),
    (
        "documented-difference",
        "Solang's docs describe this behaviour as intended on Soroban",
    ),
    (
        "evm-only",
        "Relies on an EVM concept that Soroban does not have",
    ),
    (
        "tool",
        "A sorobench or test-environment limitation, not Solang",
    ),
];

fn category_rank(c: &str) -> usize {
    CATEGORIES
        .iter()
        .position(|(n, _)| *n == c)
        .unwrap_or(CATEGORIES.len())
}

/// One failure to explain: a whole file (gap, crash, …) or one failing call.
#[derive(Debug, Clone)]
pub struct Item<'a> {
    pub kind: &'static str,
    pub text: &'a str,
    pub sig: &'a str,
}

/// The failures in one file report.
pub fn items(r: &FileReport) -> Vec<Item<'_>> {
    let file_kind = match r.bucket {
        Bucket::Gap => Some("gap"),
        Bucket::Crash => Some("crash"),
        Bucket::Timeout => Some("timeout"),
        Bucket::Filtered => Some("filtered"),
        Bucket::Unsupported => Some("unsupported"),
        _ => None,
    };
    if let Some(kind) = file_kind {
        return vec![Item {
            kind,
            text: &r.detail,
            sig: "",
        }];
    }
    if r.bucket != Bucket::HasFail {
        return Vec::new();
    }
    r.calls
        .iter()
        .filter_map(|c| {
            let kind = match c.verdict.as_str() {
                "MISMATCH" => "mismatch",
                "TRAP" => "trap",
                "NO-REVERT" => "no-revert",
                _ => return None,
            };
            Some(Item {
                kind,
                text: &c.detail,
                sig: &c.sig,
            })
        })
        .collect()
}

/// True if the signature has an integer type Soroban rounds up
/// (any width other than 32/64/128/256 bits).
pub fn has_narrow_int(sig: &str) -> bool {
    let b = sig.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let rest = &sig[i..];
        let skip = if rest.starts_with("uint") {
            4
        } else if rest.starts_with("int") && (i == 0 || !b[i - 1].is_ascii_alphanumeric()) {
            3
        } else {
            i += 1;
            continue;
        };
        let digits: String = rest[skip..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = digits.parse::<u32>() {
            if ![32, 64, 128, 256].contains(&n) {
                return true;
            }
        }
        i += skip + digits.len().max(1);
    }
    false
}

/// True if the `got Bytes([...])` part of a mismatch is a sequence of raw
/// Soroban object handles: 8 bytes each, tag byte 64..=77 then three zeros.
pub fn is_handle_bytes(detail: &str) -> bool {
    let Some(start) = detail.find("got Bytes([") else {
        return false;
    };
    let body = &detail[start + "got Bytes([".len()..];
    let Some(end) = body.find(']') else {
        return false;
    };
    let bytes: Vec<u32> = body[..end]
        .split(',')
        .filter_map(|x| x.trim().parse().ok())
        .collect();
    !bytes.is_empty()
        && bytes.len() % 8 == 0
        && bytes
            .chunks(8)
            .all(|c| (64..=77).contains(&c[0]) && c[1] == 0 && c[2] == 0 && c[3] == 0)
}

fn other_target_state(r: &FileReport) -> Option<&'static str> {
    if r.other_target == "compiles" {
        Some("compiles")
    } else if r.other_target.starts_with("fails") {
        Some("fails")
    } else {
        None
    }
}

/// The first rule that explains `item`, if any.
pub fn classify<'d>(dict: &'d Dictionary, r: &FileReport, item: &Item) -> Option<&'d Rule> {
    dict.rule.iter().find(|rule| {
        rule.kind.iter().any(|k| k == item.kind)
            && rule.all.iter().all(|s| item.text.contains(s.as_str()))
            && (rule.any.is_empty() || rule.any.iter().any(|s| item.text.contains(s.as_str())))
            && (rule.path.is_empty() || rule.path.iter().any(|s| r.path.contains(s.as_str())))
            && (rule.sig.is_empty() || rule.sig.iter().any(|s| item.sig.contains(s.as_str())))
            && rule
                .warning
                .as_ref()
                .is_none_or(|w| r.warnings.iter().any(|x| x.contains(w.as_str())))
            && (!rule.narrow_int || has_narrow_int(item.sig))
            && (!rule.handle_bytes || is_handle_bytes(item.text))
            && rule
                .other_target
                .as_ref()
                .is_none_or(|t| other_target_state(r) == Some(t.as_str()))
    })
}

/// Per-file classification: the rules that explain it, and whether any
/// failure was left unexplained.
pub struct FileExplained<'a> {
    pub report: &'a FileReport,
    pub rules: BTreeSet<usize>,
    pub unexplained: Vec<Item<'a>>,
}

pub fn explain<'a>(dict: &Dictionary, reports: &'a [FileReport]) -> Vec<FileExplained<'a>> {
    reports
        .iter()
        .map(|r| {
            let mut rules = BTreeSet::new();
            let mut unexplained = Vec::new();
            for item in items(r) {
                match classify(dict, r, &item) {
                    Some(rule) => {
                        let idx = dict
                            .rule
                            .iter()
                            .position(|x| std::ptr::eq(x, rule))
                            .unwrap();
                        rules.insert(idx);
                    }
                    None => unexplained.push(item),
                }
            }
            FileExplained {
                report: r,
                rules,
                unexplained,
            }
        })
        .collect()
}

fn one_line(s: &str, max: usize) -> String {
    let s: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.replace('|', "/");
    if s.chars().count() > max {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    } else {
        s
    }
}

/// A short, human description of one failure.
fn describe(item: &Item) -> String {
    let what = match item.kind {
        "gap" => "rejected",
        "crash" => "compiler crashed",
        "timeout" => "timed out",
        "filtered" => "excluded",
        "unsupported" => "not runnable",
        "mismatch" => "wrong value",
        "trap" => "failed at runtime",
        "no-revert" => "should have reverted",
        _ => item.kind,
    };
    let text = item.text.strip_prefix("solang: ").unwrap_or(item.text);
    if item.sig.is_empty() {
        format!("{what}: {}", one_line(text, 180))
    } else {
        format!("{what} `{}`: {}", item.sig, one_line(text, 160))
    }
}

/// Render the explained report (`report/EXPLAINED.md`).
pub fn render(dict: &Dictionary, reports: &[FileReport]) -> String {
    let explained = explain(dict, reports);
    let mut s = String::new();

    // Headline, same accounting as summary.md.
    let count = |b: Bucket| reports.iter().filter(|r| r.bucket == b).count();
    let filtered = count(Bucket::Filtered);
    let housekeeping = count(Bucket::NoBlock) + count(Bucket::FrontendError);
    let candidates = reports.len().saturating_sub(filtered + housekeeping);
    let pass = count(Bucket::PassAll) + count(Bucket::PassSome);
    let pct = if candidates > 0 {
        100.0 * pass as f64 / candidates as f64
    } else {
        0.0
    };

    let _ = writeln!(s, "# sorobench — explained results\n");
    let _ = writeln!(
        s,
        "{} tests from the solc semantic test suite were compiled with Solang for Soroban and run. \
         **{pass} of {candidates} ({pct:.1}%) pass.** {filtered} tests are excluded because they \
         rely on EVM-only features, and {housekeeping} have nothing to check.\n",
        reports.len()
    );
    let _ = writeln!(
        s,
        "Every failure below is matched against sorobench's explanation dictionary, which is \
         grounded in Solang's own Soroban documentation. A failure nothing explains is listed \
         under **review**; it is not assumed to be a bug.\n"
    );

    // Files per category (a file with several kinds of failure counts once per category).
    let mut per_cat: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for fe in &explained {
        for &i in &fe.rules {
            per_cat
                .entry(dict.rule[i].category.as_str())
                .or_default()
                .insert(&fe.report.path);
        }
        if !fe.unexplained.is_empty() {
            per_cat.entry("review").or_default().insert(&fe.report.path);
        }
    }
    let _ = writeln!(s, "## What the failures are\n");
    let _ = writeln!(s, "| category | files | meaning |");
    let _ = writeln!(s, "|---|---:|---|");
    for (cat, meaning) in CATEGORIES {
        let n = per_cat.get(cat).map_or(0, |f| f.len());
        if n > 0 {
            let _ = writeln!(s, "| **{cat}** | {n} | {meaning} |");
        }
    }
    let _ = writeln!(
        s,
        "\nA file with more than one kind of failure is counted in each of its categories.\n"
    );

    // Issues, ranked: category first, then number of files.
    let mut files_per_rule: BTreeMap<usize, Vec<&FileExplained>> = BTreeMap::new();
    for fe in &explained {
        for &i in &fe.rules {
            files_per_rule.entry(i).or_default().push(fe);
        }
    }
    let mut ranked: Vec<(usize, Vec<&FileExplained>)> = files_per_rule.into_iter().collect();
    ranked.sort_by(|(a, fa), (b, fb)| {
        category_rank(&dict.rule[*a].category)
            .cmp(&category_rank(&dict.rule[*b].category))
            .then(fb.len().cmp(&fa.len()))
    });

    let _ = writeln!(s, "## Issues\n");
    let _ = writeln!(s, "| # | issue | category | files |");
    let _ = writeln!(s, "|---:|---|---|---:|");
    for (n, (i, files)) in ranked.iter().enumerate() {
        let r = &dict.rule[*i];
        let _ = writeln!(
            s,
            "| {} | [{}](#{}) | {} | {} |",
            n + 1,
            r.title,
            r.id,
            r.category,
            files.len()
        );
    }
    let _ = writeln!(s);

    for (i, files) in &ranked {
        let r = &dict.rule[*i];
        let _ = writeln!(s, "<a id=\"{}\"></a>\n### {}\n", r.id, r.title);
        let _ = writeln!(
            s,
            "**Category:** {} · **Files:** {} · **Rule:** `{}`\n",
            r.category,
            files.len(),
            r.id
        );
        let _ = writeln!(s, "**What it means.** {}\n", r.meaning);
        let _ = writeln!(s, "**Suggested fix.** {}\n", r.suggestion);
        if !r.docs.is_empty() {
            let _ = writeln!(s, "**Docs.** {}\n", r.docs);
        }
        let _ = writeln!(s, "<details><summary>Files</summary>\n");
        for fe in files {
            // Show the first failure of this file that this rule explains.
            let shown = items(fe.report)
                .into_iter()
                .find(|it| classify(dict, fe.report, it).is_some_and(|x| x.id == r.id));
            match shown {
                Some(it) => {
                    let _ = writeln!(s, "- `{}` — {}", fe.report.path, describe(&it));
                }
                None => {
                    let _ = writeln!(s, "- `{}`", fe.report.path);
                }
            }
        }
        let _ = writeln!(s, "\n</details>\n");
    }

    // Unexplained failures.
    let review: Vec<&FileExplained> = explained
        .iter()
        .filter(|fe| !fe.unexplained.is_empty())
        .collect();
    if !review.is_empty() {
        let _ = writeln!(s, "## Needs review — {} file(s)\n", review.len());
        let _ = writeln!(
            s,
            "No dictionary rule explains these failures yet. Check each against Solang's Soroban \
             docs; if it is documented or explainable, add a rule to `dictionary/soroban.toml`, \
             otherwise it is a candidate bug.\n"
        );
        for fe in review {
            let first = &fe.unexplained[0];
            let more = if fe.unexplained.len() > 1 {
                format!(" (+{} more)", fe.unexplained.len() - 1)
            } else {
                String::new()
            };
            let hint = if fe.report.warnings.is_empty() {
                String::new()
            } else {
                format!(
                    " [warning: {}]",
                    one_line(&fe.report.warnings.join("; "), 120)
                )
            };
            let _ = writeln!(
                s,
                "- `{}` — {}{more}{hint}",
                fe.report.path,
                describe(first)
            );
        }
        let _ = writeln!(s);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dictionary_parses() {
        let d = parse(DEFAULT_DICTIONARY).expect("dictionary parses");
        assert!(!d.rule.is_empty());
        for r in &d.rule {
            assert!(
                CATEGORIES.iter().any(|(c, _)| *c == r.category),
                "rule {} has unknown category {}",
                r.id,
                r.category
            );
            assert!(!r.kind.is_empty(), "rule {} has no kind", r.id);
        }
    }

    #[test]
    fn enum_rule_wins_over_integer_rounding() {
        let d = parse(DEFAULT_DICTIONARY).unwrap();
        let r: FileReport = serde_json::from_str(
            r#"{"path":"types/mapping_enum_key_v1.sol","report":"ran","bucket":"HAS_FAIL",
                "pass":1,"fail":1,"other":0,"detail":"",
                "calls":[{"sig":"get(uint8)","verdict":"NO-REVERT","detail":"returned Int(0) instead of reverting"}],
                "warnings":["uint8 is not supported by the Soroban runtime and will be rounded up to uint32"]}"#,
        )
        .unwrap();
        let item = &items(&r)[0];
        assert_eq!(classify(&d, &r, item).unwrap().id, "enum-range-not-checked");
    }

    #[test]
    fn narrow_int_detection() {
        assert!(has_narrow_int("g(uint8,uint8)"));
        assert!(has_narrow_int("f(int16)"));
        assert!(has_narrow_int("f(uint160[])"));
        assert!(!has_narrow_int("f(uint256,int64)"));
        assert!(!has_narrow_int("f(uint32,bytes32)"));
        assert!(!has_narrow_int("print(string)"));
    }

    #[test]
    fn handle_bytes_detection() {
        assert!(is_handle_bytes(
            "expected [Bytes([0])], got Bytes([75, 0, 0, 0, 22, 0, 0, 0])"
        ));
        assert!(is_handle_bytes(
            "got Bytes([75, 0, 0, 0, 1, 0, 0, 0, 72, 0, 0, 0, 2, 0, 0, 0])"
        ));
        assert!(!is_handle_bytes("got Bytes([1, 2, 3, 4, 5, 6, 7, 8])"));
        assert!(!is_handle_bytes("got Int(5)"));
    }
}
