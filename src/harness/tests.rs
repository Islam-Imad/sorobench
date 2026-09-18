use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, IntoVal, Val};

use super::{compile_soroban, run_isolated, run_source, Exit, Outcome, RunReport, SorobanEnv};
use crate::decoder::val::{from_val, to_val};
use crate::decoder::NativeValue;
use crate::decoder::SorobanType::{U32, U64};

fn deploy(src: &str) -> (SorobanEnv, Address) {
    let compiled = compile_soroban(src).expect("compile ok");
    let mut h = SorobanEnv::new();
    let addr = h.register_contract(&compiled.wasm);
    (h, addr)
}

#[test]
fn probe_scalar_return() {
    let (h, addr) = deploy(
        "contract C { function f(uint64 x) public pure returns (uint64) { return x + 1; } }",
    );
    let arg = to_val(h.env(), &NativeValue::uint_u64(5), &U64);
    let v = h.invoke_contract(&addr, "f", vec![arg]);
    assert_eq!(
        from_val(h.env(), v, &U64).unwrap(),
        NativeValue::uint_u64(6)
    );
}

#[test]
fn probe_revert_is_failure() {
    let (h, addr) = deploy(
        "contract C { function f(uint64 x) public pure returns (uint64) { require(x > 100); return x; } }",
    );
    let arg = to_val(h.env(), &NativeValue::uint_u64(5), &U64);
    assert!(
        h.try_invoke_contract(&addr, "f", vec![arg]).is_trap(),
        "require(false) should trap"
    );
}

#[test]
fn probe_state_persists() {
    let (h, addr) = deploy(
        "contract C { uint64 s; \
         function set(uint64 x) public { s = x; } \
         function get() public view returns (uint64) { return s; } }",
    );
    let arg = to_val(h.env(), &NativeValue::uint_u64(42), &U64);
    h.invoke_contract(&addr, "set", vec![arg]);
    let v = h.invoke_contract(&addr, "get", vec![]);
    assert_eq!(
        from_val(h.env(), v, &U64).unwrap(),
        NativeValue::uint_u64(42)
    );
}

#[test]
fn probe_width_rounding_uint8() {
    let (h, addr) =
        deploy("contract C { function f(uint8 x) public pure returns (uint8) { return x; } }");
    let arg = to_val(h.env(), &NativeValue::uint_u64(200), &U32);
    let v = h.invoke_contract(&addr, "f", vec![arg]);
    eprintln!("uint8 return tag = {:?}", v.get_tag());
    assert_eq!(
        from_val(h.env(), v, &U32).unwrap(),
        NativeValue::uint_u64(200)
    );
}

#[test]
fn probe_overload_mangled_name() {
    let (h, addr) = deploy(
        "contract C { \
         function f(uint64 a) public pure returns (uint64) { return a; } \
         function f(uint64 a, uint64 b) public pure returns (uint64) { return a + b; } }",
    );
    let a = to_val(h.env(), &NativeValue::uint_u64(10), &U64);
    let b = to_val(h.env(), &NativeValue::uint_u64(20), &U64);
    let Outcome::Returned(v) = h.try_invoke_contract(&addr, "f_uint64_uint64", vec![a, b]) else {
        panic!("mangled overload f_uint64_uint64 not found")
    };
    assert_eq!(
        from_val(h.env(), v, &U64).unwrap(),
        NativeValue::uint_u64(30)
    );
}

#[test]
fn probe_multi_return_packing() {
    let src =
        "contract C { function f() public pure returns (uint64, bool) { return (7, true); } }";
    match compile_soroban(src) {
        Err(e) => eprintln!("multi-return: compile rejected → {e}"),
        Ok(c) => {
            let mut h = SorobanEnv::new();
            let addr = h.register_contract(&c.wasm);
            match h.try_invoke_contract(&addr, "f", vec![]) {
                Outcome::Trapped => eprintln!("multi-return: invoke trapped"),
                Outcome::Returned(v) => {
                    eprintln!(
                        "multi-return (uint64,bool) packed as tag = {:?}",
                        v.get_tag()
                    )
                }
            }
        }
    }
}

#[test]
fn probe_address_tag() {
    let src = "contract C { function f(address a) public pure returns (address) { return a; } }";
    match compile_soroban(src) {
        Err(e) => eprintln!("address: compile rejected → {e}"),
        Ok(c) => {
            let mut h = SorobanEnv::new();
            let addr = h.register_contract(&c.wasm);
            let a: Val = Address::generate(h.env()).into_val(h.env());
            match h.try_invoke_contract(&addr, "f", vec![a]) {
                Outcome::Trapped => eprintln!("address: invoke trapped"),
                Outcome::Returned(v) => {
                    eprintln!(
                        "address return tag = {:?} (no faithful equivalent downstream)",
                        v.get_tag()
                    )
                }
            }
        }
    }
}

#[test]
fn isolate_survives_sigabrt() {
    let res = run_isolated("sh", ["-c", "kill -ABRT $$"]).expect("spawn sh");
    assert!(
        res.exit.is_crash(),
        "expected a signal exit, got {:?}",
        res.exit
    );
    assert_eq!(res.exit, Exit::Signal(6));
}

#[test]
fn isolate_captures_clean_exit() {
    let res = run_isolated("sh", ["-c", "printf hello"]).expect("spawn sh");
    assert!(res.exit.is_success(), "got {:?}", res.exit);
    assert_eq!(res.stdout, "hello");
}

#[test]
fn probe_deploy_with_constructor_args() {
    // Deploy directly through the env with a runtime-built arg list.
    let c = compile_soroban(
        "contract C { uint64 s; \
         constructor(uint64 x) { s = x; } \
         function get() public view returns (uint64) { return s; } }",
    )
    .expect("compile ok");
    let mut h = SorobanEnv::new();
    let arg = to_val(h.env(), &NativeValue::uint_u64(7), &U64);
    let addr = h.register_contract_with_arg_vals(&c.wasm, vec![arg]);
    let v = h.invoke_contract(&addr, "get", vec![]);
    assert_eq!(from_val(h.env(), v, &U64).unwrap(), NativeValue::uint_u64(7));
}

#[test]
fn run_source_deploys_parameterized_constructor() {
    // End-to-end: the `constructor(): 7` line drives the deploy, then get() -> 7.
    let src = "contract C { uint64 s; \
         constructor(uint64 x) { s = x; } \
         function get() public view returns (uint64) { return s; } }\n\
         // ----\n\
         // constructor(): 7 ->\n\
         // get() -> 7";
    let RunReport::Ran(verdicts) = run_source(src) else {
        panic!("expected Ran, got a non-Ran report");
    };
    // constructor() is skipped (handled at registration); get() must pass.
    let get = verdicts
        .iter()
        .find(|v| v.signature.starts_with("get"))
        .expect("get() verdict");
    assert!(get.verdict.is_pass(), "get() verdict = {:?}", get.verdict);
}
