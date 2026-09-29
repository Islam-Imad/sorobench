use solang::sema::ast::Namespace;
use soroban_sdk::{Address, Env, Val};

use crate::decoder::val::{from_val, to_val};
use crate::decoder::{abi, bytes_utils, NativeValue, SorobanType};
use crate::expectation::ast::{FunctionCall, Kind, Parameter};
use crate::expectation::{parse_calls, semantic_test_builtins};
use crate::filter::filter_source;
use crate::testfile;

use super::compile::{compile_sources, Compiled};
use super::env::{Outcome, SorobanEnv};
use super::typemap::{resolve_constructor, resolve_overloads, MappedType, ResolvedFn};

#[derive(Debug)]
pub enum Verdict {
    // Actual == expected (native-space).
    Pass,
    // Expected `FAILURE` and the call trapped.
    FailureAsExpected,
    // Values differ.
    Mismatch { expected: String, actual: String },
    // Expected a value but the call trapped. Carries the host error / log.
    Trapped(String),
    // Expected `FAILURE` but the call returned. Carries what it returned.
    ExpectedFailure(String),
    // No faithful Soroban equivalent (e.g. an `address` result).
    NoFaithful(String),
    // Not run (value call / builtin / library / constructor).
    Skipped(String),
    // A type, feature, or decode case the runner doesn't handle yet.
    Unsupported(String),
}

impl Verdict {
    pub fn label(&self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::FailureAsExpected => "PASS(revert)",
            Verdict::Mismatch { .. } => "MISMATCH",
            Verdict::Trapped(_) => "TRAP",
            Verdict::ExpectedFailure(_) => "NO-REVERT",
            Verdict::NoFaithful(_) => "NoFaithful",
            Verdict::Skipped(_) => "SKIP",
            Verdict::Unsupported(_) => "UNSUPPORTED",
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Verdict::Pass | Verdict::FailureAsExpected)
    }

    pub fn is_fail(&self) -> bool {
        matches!(
            self,
            Verdict::Mismatch { .. } | Verdict::Trapped(_) | Verdict::ExpectedFailure(_)
        )
    }

    pub fn detail(&self) -> String {
        match self {
            Verdict::Mismatch { expected, actual } => format!("expected {expected}, got {actual}"),
            Verdict::Trapped(r)
            | Verdict::ExpectedFailure(r)
            | Verdict::NoFaithful(r)
            | Verdict::Skipped(r)
            | Verdict::Unsupported(r) => r.clone(),
            _ => String::new(),
        }
    }
}

pub struct CallVerdict {
    pub signature: String,
    pub verdict: Verdict,
}

pub enum RunReport {
    // parse failed (a tool/front-end issue).
    FrontendError(String),
    // No `// ----` block — nothing to run.
    NoExpectations,
    // Clean compile error (`Level::Error`) on portable source → a solang GAP to
    // fix. (Was `CompileFailed`; the crash and filtered cases split out below.)
    Gap(String),
    // Clean compile error whose source uses an EVM feature Soroban's platform
    // cannot express → EXCLUDED, not solang's fault
    Filtered { feature: String, note: String },
    // solang panicked / ICE'd while compiling. A crash is terminal and is never
    // reclassified as filtered
    Crashed(String),
    // the tool cannot handle yet.
    Unsupported(String),
    // Per-call verdicts.
    Ran(Vec<CallVerdict>),
}

enum Guarded {
    Ok(Box<Compiled>),
    CleanError(String, Vec<String>),
    Ice(String),
}

pub fn run_source(text: &str) -> RunReport {
    run_source_with_warnings(text).0
}

/// Like [`run_source`], but also returns Solang's (non-noise) warnings, which
/// often explain a failure (e.g. `uint8 … will be rounded up to uint32`).
pub fn run_source_with_warnings(text: &str) -> (RunReport, Vec<String>) {
    let mut warnings = Vec::new();
    let report = run_inner(text, &mut warnings);
    (report, warnings)
}

/// The test's sources as (resolver name, content), plus the main source name.
/// A single unnamed source is registered as `test.sol`.
fn sources_of(file: &testfile::TestFile) -> (Vec<(String, String)>, String) {
    let name = |n: &str| {
        if n.is_empty() {
            "test.sol".to_string()
        } else {
            n.to_string()
        }
    };
    let sources = file
        .sources
        .iter()
        .map(|s| (name(&s.name), s.content.clone()))
        .collect();
    (sources, name(&file.main_source))
}

/// Run the EVM-only filter over every source of the test.
pub fn filter_sources(file: &testfile::TestFile) -> Option<crate::filter::FilterReason> {
    file.sources.iter().find_map(|s| filter_source(&s.content))
}

fn run_inner(text: &str, warnings: &mut Vec<String>) -> RunReport {
    let file = match testfile::split(text) {
        Ok(f) => f,
        Err(e) => return RunReport::FrontendError(format!("split: {}", e.message)),
    };
    let Some(block) = file.expectations.as_deref() else {
        return RunReport::NoExpectations;
    };
    let builtins = semantic_test_builtins();
    let calls = match parse_calls(block, &builtins) {
        Ok(c) => c,
        Err(e) => return RunReport::FrontendError(format!("parse: {}", e.message)),
    };

    let (sources, main) = sources_of(&file);
    let compiled = match compile_guarded(&sources, &main) {
        Guarded::Ok(c) => {
            *warnings = c.warnings.clone();
            *c
        }
        // A crash on EVM-only source is excluded like a clean failure would be,
        // but the crash is kept in the note so it is not lost.
        Guarded::Ice(msg) => {
            return match filter_sources(&file) {
                Some(reason) => RunReport::Filtered {
                    feature: reason.feature.to_string(),
                    note: format!("{reason}; also crashed: {msg}"),
                },
                None => RunReport::Crashed(msg),
            }
        }
        Guarded::CleanError(msg, w) => {
            *warnings = w;
            return match filter_sources(&file) {
                Some(reason) => RunReport::Filtered {
                    feature: reason.feature.to_string(),
                    note: reason.to_string(),
                },
                None => RunReport::Gap(msg),
            };
        }
    };

    let mut h = SorobanEnv::new();
    let addr = match deploy(&mut h, &compiled, &calls) {
        Ok(a) => a,
        Err(msg) => return RunReport::Unsupported(msg),
    };

    let verdicts = calls
        .iter()
        .map(|call| CallVerdict {
            signature: call.signature.clone(),
            verdict: run_call(&h, &addr, &compiled.ns, compiled.main_contract, call),
        })
        .collect();
    RunReport::Ran(verdicts)
}

fn deploy(
    h: &mut SorobanEnv,
    compiled: &Compiled,
    calls: &[FunctionCall],
) -> Result<Address, String> {
    let ctor_params = resolve_constructor(&compiled.ns, compiled.main_contract)?;
    if ctor_params.is_empty() {
        return Ok(h.register_contract(&compiled.wasm));
    }

    let Some(ctor_call) = calls.iter().find(|c| c.kind == Kind::Constructor) else {
        return Err(format!(
            "constructor needs {} arg(s) but the test has no constructor() line",
            ctor_params.len()
        ));
    };
    let args = encode_args(h.env(), &ctor_params, &ctor_call.arguments.parameters)
        .map_err(|e| format!("constructor args: {e}"))?;
    Ok(h.register_contract_with_arg_vals(&compiled.wasm, args))
}

fn compile_guarded(sources: &[(String, String)], main: &str) -> Guarded {
    use std::sync::{Arc, Mutex};

    // Capture the panic's message *and* its source location from a hook: the
    // unwind payload alone carries only the message (often a bare "not
    // implemented"), while the location is what actually points at solang's gap.
    // The first panic is kept (it is the origin); the text is kept verbatim,
    // except that machine-specific path prefixes are shortened.
    let slot: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let hook_slot = Arc::clone(&slot);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = panic_payload_message(info.payload());
        let full = match info.location() {
            Some(l) => format!("{msg} (at {}:{}:{})", l.file(), l.line(), l.column()),
            None => msg,
        };
        if let Ok(mut g) = hook_slot.lock() {
            if g.is_none() {
                *g = Some(full);
            }
        }
    }));
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_sources(sources, main)
    }));
    std::panic::set_hook(prev);
    match res {
        Ok(Ok(c)) => Guarded::Ok(Box::new(c)),
        Ok(Err(e)) => Guarded::CleanError(e.to_string(), e.warnings.clone()),
        Err(payload) => {
            let msg = slot
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .unwrap_or_else(|| panic_payload_message(payload.as_ref()));
            Guarded::Ice(format!(
                "solang panicked mid-compile (ICE): {}",
                shorten_paths(&msg)
            ))
        }
    }
}

/// Recover the human-readable message from a `catch_unwind` payload. A `panic!`
/// carries either a `&'static str` or a `String`; anything else has no text we
/// can print, so we fall back to a default note about the panic type.
fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

/// Replace pointer-like hex literals (`0x55835f1aa770`) with `0x…`: they
/// change on every run and would split identical crashes into groups.
fn mask_addresses(msg: &str) -> String {
    let b = msg.as_bytes();
    let mut out = String::with_capacity(msg.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'0' && i + 1 < b.len() && b[i + 1] == b'x' {
            let mut j = i + 2;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j - (i + 2) >= 8 {
                out.push_str("0x…");
                i = j;
                continue;
            }
        }
        // Push the whole UTF-8 char starting at i.
        let ch = msg[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Strip machine-specific cargo prefixes from paths in a message, e.g.
/// `/home/u/.cargo/registry/src/index.crates.io-xxxx/inkwell-0.5.0/src/a.rs`
/// becomes `inkwell-0.5.0/src/a.rs`, and a git checkout of solang becomes
/// `solang/src/...`. Keeps messages comparable across machines.
pub fn shorten_paths(msg: &str) -> String {
    let mut out = mask_addresses(msg);
    // `/…/llvm-project/llvm/lib/IR/x.cpp` -> `llvm/lib/IR/x.cpp`
    while let Some(i) = out.find("/llvm-project/") {
        let start = out[..i]
            .rfind(|c: char| c.is_whitespace())
            .map(|p| p + 1)
            .unwrap_or(0);
        out.replace_range(start..i + "/llvm-project/".len(), "");
    }
    // `thread 'main' (8705) has overflowed` -> `thread 'main' has overflowed`
    if let Some(i) = out.find("' (") {
        if let Some(len) = out[i + 3..].find(')') {
            if len > 0 && out[i + 3..i + 3 + len].bytes().all(|c| c.is_ascii_digit()) {
                out.replace_range(i + 1..i + 3 + len + 1, "");
            }
        }
    }
    for marker in ["/registry/src/", "/git/checkouts/"] {
        while let Some(i) = out.find(marker) {
            let start = out[..i]
                .rfind(|c: char| c.is_whitespace())
                .map(|p| p + 1)
                .unwrap_or(0);
            let after = i + marker.len();
            let rest = &out[after..];
            let mut parts = rest.splitn(3, '/');
            let replacement_and_len = if marker == "/registry/src/" {
                // index dir, then the crate dir is kept.
                parts.next().map(|index| (String::new(), index.len() + 1))
            } else {
                // `<name>-<hash>/<rev>/` -> `<name>/`
                match (parts.next(), parts.next()) {
                    (Some(dir), Some(rev)) => {
                        let name = dir.rsplit_once('-').map(|(n, _)| n).unwrap_or(dir);
                        Some((format!("{name}/"), dir.len() + rev.len() + 2))
                    }
                    _ => None,
                }
            };
            let Some((replacement, skip)) = replacement_and_len else {
                break;
            };
            let end = (after + skip).min(out.len());
            out.replace_range(start..end, &replacement);
        }
    }
    out
}

fn run_call(
    h: &SorobanEnv,
    addr: &Address,
    ns: &Namespace,
    contract_no: usize,
    call: &FunctionCall,
) -> Verdict {
    match call.kind {
        Kind::Builtin => return Verdict::Skipped("framework builtin".into()),
        Kind::Library => return Verdict::Skipped("library declaration".into()),
        Kind::LowLevel => return Verdict::Skipped("low-level call".into()),
        Kind::Constructor => {
            return Verdict::Skipped("constructor (deploy handled at registration)".into())
        }
        Kind::Regular => {}
    }
    if call.value.is_some() {
        return Verdict::Skipped("value call (, N ether/wei)".into());
    }

    let bare = call
        .signature
        .split('(')
        .next()
        .unwrap_or(&call.signature)
        .to_string();

    let candidates = match resolve_overloads(ns, contract_no, &bare) {
        Ok(c) => c,
        Err(e) => return Verdict::Unsupported(e),
    };

    let buf = bytes_utils::encode_params(&call.arguments.parameters);
    let mut hits: Vec<(ResolvedFn, Vec<NativeValue>)> = candidates
        .into_iter()
        .filter_map(|c| {
            decode_arg_words(&c.params, &buf)
                .ok()
                .map(|items| (c, items))
        })
        .collect();
    let (resolved, items) = match hits.len() {
        0 => {
            return Verdict::Unsupported(format!(
                "`{bare}`: {} arg word(s) match no overload's parameters",
                call.arguments.parameters.len()
            ))
        }
        1 => hits.pop().unwrap(),
        _ => return Verdict::Unsupported(format!("ambiguous overload `{bare}` for decoded args")),
    };

    let args = match args_to_vals(h.env(), &resolved.params, &items) {
        Ok(a) => a,
        Err(e) => return Verdict::Unsupported(e),
    };

    let outcome = h.try_invoke_contract(addr, &resolved.export_name, args);
    compare(h, call, &resolved, outcome)
}

fn tuple_abi(types: &[MappedType]) -> String {
    let inner: Vec<&str> = types.iter().map(|m| m.abi.as_str()).collect();
    format!("({})", inner.join(","))
}

fn decode_arg_words(params: &[MappedType], buf: &[u8]) -> Result<Vec<NativeValue>, String> {
    if params.is_empty() {
        return if buf.is_empty() {
            Ok(Vec::new())
        } else {
            Err("no-arg signature but arg words present".into())
        };
    }
    let decoded =
        abi::abi_decode_params(buf, &tuple_abi(params)).map_err(|e| format!("arg decode: {e}"))?;
    let NativeValue::Tuple(items) = decoded else {
        return Err("arg decode did not yield a tuple".into());
    };
    if items.len() != params.len() {
        return Err(format!(
            "arg count {} != {} params",
            items.len(),
            params.len()
        ));
    }
    Ok(items)
}

fn args_to_vals(
    env: &Env,
    params: &[MappedType],
    items: &[NativeValue],
) -> Result<Vec<Val>, String> {
    let mut vals = Vec::with_capacity(items.len());
    for (nv, p) in items.iter().zip(params) {
        if p.soroban == SorobanType::Address {
            return Err("address argument (NoFaithful)".into());
        }
        vals.push(to_val(env, nv, &p.soroban));
    }
    Ok(vals)
}

fn encode_args(
    env: &Env,
    params: &[MappedType],
    parameters: &[Parameter],
) -> Result<Vec<Val>, String> {
    let buf = bytes_utils::encode_params(parameters);
    let items = decode_arg_words(params, &buf)?;
    args_to_vals(env, params, &items)
}

fn compare(
    h: &SorobanEnv,
    call: &FunctionCall,
    resolved: &ResolvedFn,
    outcome: Outcome,
) -> Verdict {
    let expects_failure = call.expectations.failure;

    let ret_val = match outcome {
        Outcome::Trapped(reason) => {
            return if expects_failure {
                Verdict::FailureAsExpected
            } else {
                Verdict::Trapped(reason)
            };
        }
        Outcome::Returned(v) => v,
    };
    if expects_failure {
        let what = match resolved.returns.as_slice() {
            [] => "returned (void) instead of reverting".to_string(),
            [ret] => match from_val(h.env(), ret_val, &ret.soroban) {
                Ok(nv) => format!("returned {nv:?} instead of reverting"),
                Err(_) => "returned a value instead of reverting".to_string(),
            },
            _ => "returned a value instead of reverting".to_string(),
        };
        return Verdict::ExpectedFailure(what);
    }

    // A setup call with no `->`: success = it didn't trap (already known).
    if call.omits_arrow {
        return Verdict::Pass;
    }

    // Void function: pass iff the expectation carries no result value.
    if resolved.returns.is_empty() {
        return if call.expectations.result.is_empty() {
            Verdict::Pass
        } else {
            Verdict::Mismatch {
                expected: format!("{} value(s)", call.expectations.result.len()),
                actual: "void".into(),
            }
        };
    }

    // address result → no faithful equivalent (20 ≠ 32 byte).
    if resolved
        .returns
        .iter()
        .any(|r| r.soroban == SorobanType::Address)
    {
        return Verdict::NoFaithful("address result (20≠32 byte)".into());
    }

    // Soroban has no multi-value return, so solang would have failed to compile it.
    if resolved.returns.len() != 1 {
        return Verdict::Unsupported(format!("{}-value return", resolved.returns.len()));
    }

    // Expected side: `// ----` words → ABI decode with the return type.
    let exp_buf = bytes_utils::encode_params(&call.expectations.result);
    let expected = match abi::abi_decode_params(&exp_buf, &tuple_abi(&resolved.returns)) {
        Ok(NativeValue::Tuple(items)) => items,
        Ok(_) => return Verdict::Unsupported("expected decode not a tuple".into()),
        Err(e) => return Verdict::Unsupported(format!("expected decode: {e}")),
    };

    // Actual side: from_val the single returned value.
    let actual = match from_val(h.env(), ret_val, &resolved.returns[0].soroban) {
        Ok(nv) => nv,
        Err(e) => return Verdict::Unsupported(format!("from_val: {e}")),
    };

    if expected.len() == 1 && expected[0] == actual {
        Verdict::Pass
    } else {
        Verdict::Mismatch {
            expected: format!("{expected:?}"),
            actual: format!("{actual:?}"),
        }
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use super::{mask_addresses, shorten_paths};
    use crate::harness::isolate::crash_summary;

    #[test]
    fn shortens_registry_and_git_paths() {
        let m = "panicked at /home/u/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/inkwell-0.5.0/src/values/enums.rs:325:13: x";
        assert_eq!(
            shorten_paths(m),
            "panicked at inkwell-0.5.0/src/values/enums.rs:325:13: x"
        );
        let g =
            "at /root/.cargo/git/checkouts/solang-2f1b0a9c3d/e6289eb/src/sema/yul/builtin.rs:25:32";
        assert_eq!(shorten_paths(g), "at solang/src/sema/yul/builtin.rs:25:32");
        let l = "sorobench: /home/runner/work/solang-llvm/solang-llvm/llvm-project/llvm/lib/IR/Instructions.cpp:631: x";
        assert_eq!(
            shorten_paths(l),
            "sorobench: llvm/lib/IR/Instructions.cpp:631: x"
        );
        assert_eq!(
            shorten_paths("thread 'main' (8705) has overflowed its stack"),
            "thread 'main' has overflowed its stack"
        );
    }

    #[test]
    fn masks_pointer_addresses_only() {
        assert_eq!(
            mask_addresses("address: 0x55835f1aa770, value 0x20"),
            "address: 0x…, value 0x20"
        );
    }

    #[test]
    fn crash_summary_picks_the_informative_line() {
        let panic =
            "noise\nthread 'main' panicked at src/a.rs:1:2:\nboom\nnote: run with RUST_BACKTRACE";
        assert_eq!(
            crash_summary(panic),
            "thread 'main' panicked at src/a.rs:1:2: | boom"
        );
        let llvm = "x\nsorobench: Instructions.cpp:631: void llvm::CallInst::init(): Assertion `ok' failed.\n";
        assert!(crash_summary(llvm).contains("Assertion"));
        assert_eq!(crash_summary("a\nb\nc\nd"), "b | c | d");
    }
}
