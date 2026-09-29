use std::ffi::OsStr;
use std::fmt;

use solang::codegen::Options;
use solang::file_resolver::FileResolver;
use solang::sema::ast::Namespace;
use solang::{compile, Target};
use solang_parser::diagnostics::Level;

// A successful Soroban compile.
pub struct Compiled {
    // wasm of the contract under test (the last instantiable one, per solc).
    pub wasm: Vec<u8>,
    // Every instantiable contract's wasm.
    pub all_wasm: Vec<Vec<u8>>,
    pub main_contract: usize,
    // The resolved `Namespace` — the source of function export names and
    // param/return types for the decoder.
    pub ns: Namespace,
    // Solang's warnings (deduplicated, noise removed). They can explain
    // behaviour differences, e.g. integer widths rounded up on Soroban.
    pub warnings: Vec<String>,
}

pub struct CompileError {
    pub messages: Vec<String>,
    pub ns: Namespace,
    pub warnings: Vec<String>,
}

/// Warnings that appear in nearly every test and explain nothing.
fn is_noise(msg: &str) -> bool {
    msg.starts_with("storage type not specified")
        || msg.starts_with("function can be declared")
        || msg.starts_with("function parameter")
        || msg.contains("has never been used")
        || msg.contains("has been assigned, but never read")
}

fn collect_warnings(ns: &Namespace) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for d in ns.diagnostics.iter() {
        if d.level == Level::Warning && !is_noise(&d.message) && !out.contains(&d.message) {
            out.push(d.message.clone());
        }
    }
    out
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.messages.is_empty() {
            f.write_str("solang produced no instantiable contract")
        } else {
            write!(f, "solang: {}", self.messages.join("; "))
        }
    }
}

impl fmt::Debug for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompileError({self})")
    }
}

pub fn compile_soroban(src: &str) -> Result<Compiled, CompileError> {
    compile_sources(&[("test.sol".to_string(), src.to_string())], "test.sol")
}

/// Compile a (possibly multi-source) test. Every source is registered with the
/// file resolver under its `==== Source: NAME ====` name, so `import "NAME";`
/// resolves; `main` is the source compiled (solc's last source).
pub fn compile_sources(sources: &[(String, String)], main: &str) -> Result<Compiled, CompileError> {
    let file = OsStr::new(main);
    let mut cache = FileResolver::default();
    for (name, content) in sources {
        cache.set_file_contents(name, content.clone());
    }

    let opts = Options {
        log_runtime_errors: true,
        ..Default::default()
    };

    let (results, ns) = compile(
        file,
        &mut cache,
        Target::Soroban,
        &opts,
        vec!["sorobench".to_string()],
        "0.0.1",
    );
    let warnings = collect_warnings(&ns);

    if ns.diagnostics.any_errors() || results.is_empty() {
        let messages = ns
            .diagnostics
            .iter()
            .filter(|d| d.level == Level::Error)
            .map(|d| d.message.clone())
            .collect();
        return Err(CompileError {
            messages,
            ns,
            warnings,
        });
    }

    // solc's convention: the LAST contract in the file is the one under test.
    // `results` are emitted in `ns.contracts` order (one per instantiable
    // contract), so the last result is the last instantiable contract's wasm.
    let all_wasm: Vec<Vec<u8>> = results.into_iter().map(|(w, _)| w).collect();
    let wasm = all_wasm
        .last()
        .expect("results non-empty ⇒ at least one wasm")
        .clone();
    let main_contract = ns
        .contracts
        .iter()
        .rposition(|c| c.instantiable)
        .expect("results non-empty ⇒ an instantiable contract exists");
    Ok(Compiled {
        wasm,
        all_wasm,
        main_contract,
        ns,
        warnings,
    })
}
