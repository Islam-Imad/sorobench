//! Splits a `.sol` test file into its parts. Ported from solc's `TestCaseReader`.
//!
//! A test file has three kinds of content:
//! - the Solidity source(s); multiple sources are separated by
//!   `==== Source: NAME ====`.
//! - the settings — the `// key: value` lines after a `// ====` marker.
//! - the raw expectation block after the `// ----` marker (handed to
//!   [`crate::expectation`] for parsing, not parsed here).

use std::collections::BTreeMap;

const EXPECTATIONS_DELIMITER: &str = "// ----";
const SETTINGS_DELIMITER: &str = "// ====";
const SOURCE_DELIMITER_START: &str = "==== Source:";
const EXTERNAL_SOURCE_DELIMITER_START: &str = "==== ExternalSource:";
const SOURCE_DELIMITER_END: &str = "====";
const COMMENT: &str = "// ";

/// A named Solidity source unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Source name (empty for a single unnamed source).
    pub name: String,
    pub content: String,
}

/// An `==== ExternalSource: [name=]path ====` reference (recorded, not loaded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSource {
    pub name: String,
    pub target: String,
}

/// A split test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFile {
    pub sources: Vec<Source>,
    /// Name of the main (last) source — the one `TestCaseReader::source()` returns.
    pub main_source: String,
    pub external_sources: Vec<ExternalSource>,
    pub settings: BTreeMap<String, String>,
    /// Raw expectation text (lines after `// ----`, joined by `\n`), or `None`
    /// if the file has no expectation block.
    pub expectations: Option<String>,
}

impl TestFile {
    /// The main source content (empty string if none).
    pub fn main_source_content(&self) -> &str {
        self.sources
            .iter()
            .find(|s| s.name == self.main_source)
            .map(|s| s.content.as_str())
            .unwrap_or("")
    }
}

/// An error while splitting regions (settings/source malformation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitError {
    pub message: String,
}

impl SplitError {
    fn new(message: impl Into<String>) -> SplitError {
        SplitError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SplitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SplitError {}

/// Split a test-file's text into its sources, settings, and expectation block.
/// Mirrors solc's `TestCaseReader`.
pub fn split(text: &str) -> Result<TestFile, SplitError> {
    let mut sources: Vec<Source> = Vec::new();
    let mut external_sources: Vec<ExternalSource> = Vec::new();
    let mut settings: BTreeMap<String, String> = BTreeMap::new();
    let mut current_source_name = String::new();
    let mut current_source = String::new();
    let mut source_part = true;
    let mut expectations: Option<String> = None;

    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        // Anything below `// ----` is the expectations block; capture it verbatim.
        if line.starts_with(EXPECTATIONS_DELIMITER) {
            let rest: Vec<&str> = lines.collect();
            expectations = Some(rest.join("\n"));
            break;
        }

        if line.starts_with(SETTINGS_DELIMITER) {
            source_part = false;
        } else if source_part {
            if line.starts_with(SOURCE_DELIMITER_START) && line.ends_with(SOURCE_DELIMITER_END) {
                if !(current_source_name.is_empty() && current_source.is_empty()) {
                    sources.push(Source {
                        name: std::mem::take(&mut current_source_name),
                        content: std::mem::take(&mut current_source),
                    });
                }
                let inner =
                    &line[SOURCE_DELIMITER_START.len()..line.len() - SOURCE_DELIMITER_END.len()];
                current_source_name = inner.trim().to_string();
                if sources.iter().any(|s| s.name == current_source_name) {
                    return Err(SplitError::new(format!(
                        "Multiple definitions of test source \"{current_source_name}\"."
                    )));
                }
            } else if line.starts_with(EXTERNAL_SOURCE_DELIMITER_START)
                && line.ends_with(SOURCE_DELIMITER_END)
            {
                let inner = &line[EXTERNAL_SOURCE_DELIMITER_START.len()
                    ..line.len() - SOURCE_DELIMITER_END.len()];
                let external = inner.trim();
                let (name, target) = match external.find('=') {
                    Some(pos) => (external[..pos].trim(), external[pos + 1..].trim()),
                    None => (external, external),
                };
                external_sources.push(ExternalSource {
                    name: name.to_string(),
                    target: target.to_string(),
                });
            } else {
                current_source.push_str(line);
                current_source.push('\n');
            }
        } else if line.starts_with(COMMENT) {
            match line.find(':') {
                None => return Err(SplitError::new("Expected \":\" inside setting.")),
                Some(colon) => {
                    let key = line[COMMENT.len()..colon].trim().to_string();
                    let value = line[colon + 1..].trim().to_string();
                    settings.insert(key, value);
                }
            }
        } else {
            return Err(SplitError::new(
                "Expected \"//\" or \"// ---\" to terminate settings and source.",
            ));
        }
    }

    // Register the last (main) source.
    sources.push(Source {
        name: current_source_name.clone(),
        content: current_source,
    });

    Ok(TestFile {
        sources,
        main_source: current_source_name,
        external_sources,
        settings,
        expectations,
    })
}
