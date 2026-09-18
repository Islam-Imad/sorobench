use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub const DEFAULT_CORPUS: &str =
    "/home/islam/contrib/solang/testdata/solidity/test/libsolidity/semanticTests";

pub const EXPECTED_COUNT: usize = 1503;

pub fn corpus_root(cli: Option<&str>) -> PathBuf {
    if let Some(p) = cli {
        return PathBuf::from(p);
    }
    if let Ok(p) = env::var("SOROBENCH_CORPUS") {
        return PathBuf::from(p);
    }
    PathBuf::from(DEFAULT_CORPUS)
}

pub fn enumerate(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let path = entry.path();
        if ft.is_dir() {
            walk(&path, out)?;
        } else if ft.is_file() && path.extension().is_some_and(|e| e == "sol") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_pinned_corpus() {
        let root = corpus_root(None);
        if !root.exists() {
            eprintln!("skipping: corpus not present at {}", root.display());
            return;
        }
        let paths = enumerate(&root).expect("walk corpus");
        assert_eq!(
            paths.len(),
            EXPECTED_COUNT,
            "corpus count drifted from pinned solc v0.8.22 (found {}, expected {})",
            paths.len(),
            EXPECTED_COUNT
        );
    }
}
