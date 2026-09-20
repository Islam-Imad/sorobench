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
}

pub struct CompileError {
    pub messages: Vec<String>,
    pub ns: Namespace,
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
    let file = OsStr::new("test.sol");
    let mut cache = FileResolver::default();
    cache.set_file_contents("test.sol", src.to_string());

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

    if ns.diagnostics.any_errors() || results.is_empty() {
        let messages = ns
            .diagnostics
            .iter()
            .filter(|d| d.level == Level::Error)
            .map(|d| d.message.clone())
            .collect();
        return Err(CompileError { messages, ns });
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
    })
}
