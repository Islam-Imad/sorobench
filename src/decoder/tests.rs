//! Decoder tests: the round-trip `from_val(to_val(x)) == x` across every
//! supported type, the token→bytes step, and that an EVM-ABI expected value
//! decodes to the same `NativeValue` as the matching Soroban actual value.

use soroban_sdk::Env;

use super::native::NativeValue;
use super::types::SorobanType::{self, *};
use super::{abi, bytes_utils, nofaithful, val};
use crate::expectation::{parse_calls, Builtins};

fn parse_one(src: &str) -> crate::expectation::ast::FunctionCall {
    parse_calls(src, &Builtins::new())
        .expect("parse")
        .into_iter()
        .next()
        .expect("one call")
}

/// The round-trip check: `from_val(to_val(x)) == x` (no solang involved).
fn assert_roundtrip(x: NativeValue, ty: SorobanType) {
    let env = Env::default();
    let v = val::to_val(&env, &x, &ty);
    let back = val::from_val(&env, v, &ty).expect("from_val");
    assert_eq!(back, x, "round-trip mismatch for type {ty:?}");
}

// ---------- round-trip: one case per supported type ----------

#[test]
fn rt_bool() {
    assert_roundtrip(NativeValue::Bool(true), Bool);
    assert_roundtrip(NativeValue::Bool(false), Bool);
}

#[test]
fn rt_small_unsigned() {
    for ty in [U32, U64] {
        assert_roundtrip(NativeValue::uint_u64(6), ty.clone());
        assert_roundtrip(NativeValue::uint_u64(0), ty);
    }
}

#[test]
fn rt_small_signed() {
    for ty in [I32, I64] {
        assert_roundtrip(NativeValue::int_i128(-2), ty.clone());
        assert_roundtrip(NativeValue::int_i128(7), ty);
    }
}

#[test]
fn rt_wide_unsigned() {
    assert_roundtrip(NativeValue::uint_u128(1_000_000_000_000), U128);
    assert_roundtrip(NativeValue::uint_u128(1_000_000_000_000), U256);
}

#[test]
fn rt_wide_signed() {
    assert_roundtrip(NativeValue::int_i128(-1_000_000_000_000), I128);
    assert_roundtrip(NativeValue::int_i128(-1_000_000_000_000), I256);
}

#[test]
fn rt_bytes_and_string() {
    assert_roundtrip(NativeValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef]), Bytes);
    assert_roundtrip(NativeValue::Bytes(vec![]), Bytes);
    assert_roundtrip(NativeValue::Str(b"hello world".to_vec()), Str);
}

#[test]
fn rt_vec() {
    let x = NativeValue::Vec(vec![
        NativeValue::uint_u64(1),
        NativeValue::uint_u64(2),
        NativeValue::uint_u64(3),
    ]);
    assert_roundtrip(x, Vec(Box::new(U64)));
}

#[test]
fn rt_struct_symbol_keyed_map() {
    // A struct is a Symbol-keyed Map, read back by field key.
    let x = NativeValue::Tuple(vec![NativeValue::uint_u64(13), NativeValue::Bool(true)]);
    let ty = Struct(vec![("x".to_string(), U64), ("flag".to_string(), Bool)]);
    assert_roundtrip(x, ty);
}

#[test]
fn rt_vec_of_struct() {
    let x = NativeValue::Vec(vec![
        NativeValue::Tuple(vec![NativeValue::uint_u64(13)]),
        NativeValue::Tuple(vec![NativeValue::uint_u64(99)]),
    ]);
    let ty = Vec(Box::new(Struct(vec![("x".to_string(), U64)])));
    assert_roundtrip(x, ty);
}

// ---------- token→bytes materializer (BytesUtils port) ----------

fn word_with_last(byte: u8) -> std::vec::Vec<u8> {
    let mut w = vec![0u8; 32];
    w[31] = byte;
    w
}

#[test]
fn materialize_uint_right_aligned() {
    let call = parse_one("// f(uint256): 5 -> 6");
    assert_eq!(
        bytes_utils::encode_params(&call.arguments.parameters),
        word_with_last(5)
    );
    assert_eq!(
        bytes_utils::encode_params(&call.expectations.result),
        word_with_last(6)
    );
}

#[test]
fn materialize_negative_twos_complement() {
    let call = parse_one("// f(): -2 ->");
    let mut want = vec![0xffu8; 32];
    want[31] = 0xfe;
    assert_eq!(bytes_utils::encode_params(&call.arguments.parameters), want);
}

#[test]
fn materialize_bool() {
    let call = parse_one("// f(bool): true -> false");
    assert_eq!(
        bytes_utils::encode_params(&call.arguments.parameters),
        word_with_last(1)
    );
    assert_eq!(
        bytes_utils::encode_params(&call.expectations.result),
        word_with_last(0)
    );
}

#[test]
fn materialize_hex_string_unpadded() {
    let call = parse_one("// f(bytes): hex\"4200ef\" ->");
    // hex-string is the one token that is NOT padded to 32.
    assert_eq!(
        bytes_utils::encode_params(&call.arguments.parameters),
        vec![0x42, 0x00, 0xef]
    );
}

#[test]
fn materialize_string_offset_len_data() {
    // A string is encoded as: 0x20 (offset) | 3 (len) | "any" (left-aligned data).
    let call = parse_one("// f() -> 0x20, 3, \"any\"");
    let b = bytes_utils::encode_params(&call.expectations.result);
    assert_eq!(b.len(), 96);
    assert_eq!(b[31], 0x20);
    assert_eq!(b[63], 3);
    assert_eq!(&b[64..67], b"any");
    assert_eq!(&b[67..96], &[0u8; 29]); // right-zero-padded
}

// ---------- abi_decode (materialize → alloy → NativeValue) ----------

#[test]
fn abi_decode_uint() {
    let call = parse_one("// f() -> 6");
    let buf = bytes_utils::encode_params(&call.expectations.result);
    let nv = abi::abi_decode_params(&buf, "(uint256)").unwrap();
    assert_eq!(nv, NativeValue::Tuple(vec![NativeValue::uint_u64(6)]));
}

#[test]
fn abi_decode_string_consumes_offset() {
    let call = parse_one("// f() -> 0x20, 3, \"any\"");
    let buf = bytes_utils::encode_params(&call.expectations.result);
    // alloy consumes the 0x20 offset + 3 length; only "any" survives.
    let nv = abi::abi_decode_params(&buf, "(string)").unwrap();
    assert_eq!(
        nv,
        NativeValue::Tuple(vec![NativeValue::Str(b"any".to_vec())])
    );
}

// ---------- expected vs actual: decode both sides, compare as NativeValue ----------

#[test]
fn oracle_uint_scalar() {
    // f(uint): 5 -> 6. Expected via ABI; actual as a Soroban U64.
    let call = parse_one("// f(uint256): 5 -> 6");
    let expected = abi::abi_decode_params(
        &bytes_utils::encode_params(&call.expectations.result),
        "(uint256)",
    )
    .unwrap();

    let env = Env::default();
    let actual_val = val::to_val(&env, &NativeValue::uint_u64(6), &U64);
    let actual = val::from_val(&env, actual_val, &U64).unwrap();

    // Expected is a params tuple; unwrap the single element and compare in native space.
    let NativeValue::Tuple(items) = expected else {
        panic!("expected a tuple")
    };
    assert_eq!(items[0], actual);
}

#[test]
fn oracle_struct_array_return() {
    // f() -> 0x20, 1, 13 with return type S[] where S { uint x }.
    let call = parse_one("// f() -> 0x20, 1, 13");
    let expected = abi::abi_decode_params(
        &bytes_utils::encode_params(&call.expectations.result),
        "((uint256)[])",
    )
    .unwrap();
    // The 0x20 offset + 1 length are consumed; only the element survives.
    let want = NativeValue::Tuple(vec![NativeValue::Vec(vec![NativeValue::Tuple(vec![
        NativeValue::uint_u64(13),
    ])])]);
    assert_eq!(expected, want);

    // Actual side: S[] as a VecObject of Symbol-keyed Maps {x: 13}.
    let env = Env::default();
    let sty = Vec(Box::new(Struct(vec![("x".to_string(), U64)])));
    let native_actual = NativeValue::Vec(vec![NativeValue::Tuple(vec![NativeValue::uint_u64(13)])]);
    let actual = val::from_val(&env, val::to_val(&env, &native_actual, &sty), &sty).unwrap();

    // Native-space compare: unwrap the expected params tuple; the array matches.
    let NativeValue::Tuple(items) = expected else {
        panic!()
    };
    assert_eq!(items[0], actual);
}

// ---------- width/sign-agnostic native compare ----------

#[test]
fn native_compare_is_width_agnostic() {
    // A Soroban U64(6) and an ABI uint256(6) both canonicalize to Int(6).
    let env = Env::default();
    let from_u64 = val::from_val(
        &env,
        val::to_val(&env, &NativeValue::uint_u64(6), &U64),
        &U64,
    )
    .unwrap();
    let from_abi = abi::abi_decode_params(
        &bytes_utils::encode_params(&parse_one("// f() -> 6").expectations.result),
        "(uint256)",
    )
    .unwrap();
    let NativeValue::Tuple(items) = from_abi else {
        panic!()
    };
    assert_eq!(from_u64, items[0]);
}

// ---------- NoFaithful ----------

#[test]
fn nofaithful_flags_address() {
    let twenty = NativeValue::Address(vec![0x11; 20]);
    assert_eq!(
        nofaithful::classify(&Address, &twenty),
        Some(nofaithful::NoFaithful::Address20)
    );
    assert_eq!(nofaithful::classify(&U64, &NativeValue::uint_u64(6)), None);
}
