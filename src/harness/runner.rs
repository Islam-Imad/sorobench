use solang::sema::ast::Namespace;
use soroban_sdk::{Address, Env, Val};

use crate::decoder::val::{from_val, to_val};
use crate::decoder::{abi, bytes_utils, NativeValue, SorobanType};
use crate::expectation::ast::{FunctionCall, Kind, Parameter};
use crate::expectation::{parse_calls, semantic_test_builtins};
use crate::testfile;

use super::compile::{compile_soroban, Compiled};
use super::env::{Outcome, SorobanEnv};
use super::typemap::{resolve_constructor, resolve_fn, MappedType, ResolvedFn};

#[derive(Debug)]
pub enum Verdict {
    // Actual == expected (native-space).
    Pass,
    // Expected `FAILURE` and the call trapped.
    FailureAsExpected,
    // Values differ.
    Mismatch { expected: String, actual: String },
    // Expected a value but the call trapped.
    Trapped,
    // Expected `FAILURE` but the call returned a value.
    ExpectedFailure,
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
            Verdict::Trapped => "TRAP",
            Verdict::ExpectedFailure => "NO-REVERT",
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
            Verdict::Mismatch { .. } | Verdict::Trapped | Verdict::ExpectedFailure
        )
    }

    pub fn detail(&self) -> String {
        match self {
            Verdict::Mismatch { expected, actual } => format!("expected {expected}, got {actual}"),
            Verdict::NoFaithful(r) | Verdict::Skipped(r) | Verdict::Unsupported(r) => r.clone(),
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
    // solang could not compile the source, or panicked while compiling.
    CompileFailed(String),
    // A whole-file limitation of the runner (e.g. a constructor that needs
    // args, which isn't supported yet). Not a solang bug.
    Unsupported(String),
    // Per-call verdicts.
    Ran(Vec<CallVerdict>),
}

pub fn run_source(text: &str) -> RunReport {
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

    let compiled = match compile_guarded(file.main_source_content()) {
        Ok(c) => c,
        Err(msg) => return RunReport::CompileFailed(msg),
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

fn compile_guarded(src: &str) -> Result<Compiled, String> {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compile_soroban(src)));
    std::panic::set_hook(prev);
    match res {
        Ok(Ok(c)) => Ok(c),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("solang panicked mid-compile (ICE)".into()),
    }
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
    let arity = call.arguments.parameters.len();
    let resolved = match resolve_fn(ns, contract_no, &bare, arity) {
        Ok(r) => r,
        Err(e) => return Verdict::Unsupported(e),
    };

    let args = match build_args(h, call, &resolved) {
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

fn build_args(
    h: &SorobanEnv,
    call: &FunctionCall,
    resolved: &ResolvedFn,
) -> Result<Vec<Val>, String> {
    encode_args(h.env(), &resolved.params, &call.arguments.parameters)
}

fn encode_args(
    env: &Env,
    params: &[MappedType],
    parameters: &[Parameter],
) -> Result<Vec<Val>, String> {
    if params.is_empty() {
        return Ok(Vec::new());
    }
    let buf = bytes_utils::encode_params(parameters);
    let decoded =
        abi::abi_decode_params(&buf, &tuple_abi(params)).map_err(|e| format!("arg decode: {e}"))?;
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
    let mut vals = Vec::with_capacity(items.len());
    for (nv, p) in items.iter().zip(params) {
        if p.soroban == SorobanType::Address {
            return Err("address argument (NoFaithful)".into());
        }
        vals.push(to_val(env, nv, &p.soroban));
    }
    Ok(vals)
}

fn compare(
    h: &SorobanEnv,
    call: &FunctionCall,
    resolved: &ResolvedFn,
    outcome: Outcome,
) -> Verdict {
    let expects_failure = call.expectations.failure;

    let ret_val = match outcome {
        Outcome::Trapped => {
            return if expects_failure {
                Verdict::FailureAsExpected
            } else {
                Verdict::Trapped
            };
        }
        Outcome::Returned(v) => v,
    };
    if expects_failure {
        return Verdict::ExpectedFailure;
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
