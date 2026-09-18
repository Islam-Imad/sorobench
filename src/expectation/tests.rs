//! Accept/reject cases ported from solc's TestFileParserTests.cpp.
//!
//! These check the parser's structure and type classification (signature, kind,
//! display mode, value, alignment, literals, comments, side effects, failure),
//! and every reject boundary. The exact ABI byte encoding is checked separately
//! in the decoder tests.

use super::ast::*;
use super::{parse_calls, Builtins};

fn parse(src: &str) -> Vec<FunctionCall> {
    parse_calls(src, &Builtins::new()).expect("expected successful parse")
}

fn parse_b(src: &str, builtins: &[&str]) -> Vec<FunctionCall> {
    let set: Builtins = builtins.iter().map(|s| s.to_string()).collect();
    parse_calls(src, &set).expect("expected successful parse")
}

fn reject(src: &str) {
    let r = parse_calls(src, &Builtins::new());
    assert!(r.is_ok().not(), "expected a parse error, got {:?}", r);
}

fn reject_b(src: &str, builtins: &[&str]) {
    let set: Builtins = builtins.iter().map(|s| s.to_string()).collect();
    let r = parse_calls(src, &set);
    assert!(r.is_ok().not(), "expected a parse error, got {:?}", r);
}

trait BoolNot {
    fn not(self) -> bool;
}
impl BoolNot for bool {
    fn not(self) -> bool {
        !self
    }
}

fn dec(negative: bool, digits: &str) -> Literal {
    Literal::Decimal {
        negative,
        digits: digits.to_string(),
    }
}

// ---------- ACCEPT ----------

#[test]
fn smoke_test() {
    assert_eq!(parse("").len(), 0);
}

#[test]
fn call_success() {
    let c = parse("// success() ->");
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].signature, "success()");
    assert_eq!(c[0].display_mode, DisplayMode::SingleLine);
    assert!(!c[0].expectations.failure);
    assert!(!c[0].omits_arrow);
    assert!(c[0].arguments.parameters.is_empty());
    assert!(c[0].expectations.result.is_empty());
}

#[test]
fn non_existent_call_revert_single_line() {
    let c = parse("// i_am_not_there() -> FAILURE");
    assert_eq!(c[0].signature, "i_am_not_there()");
    assert_eq!(c[0].display_mode, DisplayMode::SingleLine);
    assert!(c[0].expectations.failure);
    assert_eq!(
        c[0].expectations.result[0].abi_type.kind,
        AbiTypeKind::Failure
    );
}

#[test]
fn call_arguments_success() {
    let c = parse("// f(uint256): 1\n// ->");
    assert_eq!(c[0].signature, "f(uint256)");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert!(!c[0].expectations.failure);
    assert_eq!(c[0].arguments.parameters.len(), 1);
    assert_eq!(c[0].arguments.parameters[0].literal, dec(false, "1"));
    assert!(c[0].expectations.result.is_empty());
}

#[test]
fn call_arguments_comments_success() {
    let c = parse(
        "// f(uint256, uint256): 1, 1\n\
         // # Comment on the parameters. #\n\
         // ->\n\
         // # This call should not return a value, but still succeed. #\n\
         // f()\n\
         // # Comment on no parameters. #\n\
         // -> 1\n\
         // # This comment should be parsed. #",
    );
    assert_eq!(c.len(), 2);

    assert_eq!(c[0].signature, "f(uint256,uint256)");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert_eq!(c[0].arguments.parameters.len(), 2);
    assert_eq!(c[0].arguments.comment, " Comment on the parameters. ");
    assert!(c[0].expectations.result.is_empty());
    assert_eq!(
        c[0].expectations.comment,
        " This call should not return a value, but still succeed. "
    );

    assert_eq!(c[1].signature, "f()");
    assert_eq!(c[1].arguments.comment, " Comment on no parameters. ");
    assert_eq!(c[1].expectations.result.len(), 1);
    assert_eq!(
        c[1].expectations.comment,
        " This comment should be parsed. "
    );
}

#[test]
fn simple_single_line_call_comment_success() {
    let c = parse(
        "// f(uint256): 1 -> # f(uint256) does not return a value. #\n\
         // f(uint256): 1 -> 1",
    );
    assert_eq!(c.len(), 2);
    assert_eq!(c[0].display_mode, DisplayMode::SingleLine);
    assert_eq!(
        c[0].expectations.comment,
        " f(uint256) does not return a value. "
    );
    assert!(c[0].expectations.result.is_empty());
    assert_eq!(c[1].expectations.result.len(), 1);
}

#[test]
fn multiple_single_line() {
    let c = parse("// f(uint256): 1 -> 1\n// g(uint256): 1 ->");
    assert_eq!(c.len(), 2);
    assert_eq!(c[0].signature, "f(uint256)");
    assert_eq!(c[0].expectations.result.len(), 1);
    assert_eq!(c[1].signature, "g(uint256)");
    assert!(c[1].expectations.result.is_empty());
}

#[test]
fn non_existent_call_revert_multiline() {
    let c = parse("// i_am_not_there()\n// -> FAILURE");
    assert_eq!(c[0].signature, "i_am_not_there()");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert!(c[0].expectations.failure);
}

#[test]
fn call_revert_message() {
    let c = parse("// f() -> FAILURE, hex\"08c379a0\", 0x20, 6, \"Revert\"");
    assert_eq!(c[0].signature, "f()");
    assert!(c[0].expectations.failure);
    // FAILURE + trailing revert-data words are all captured as params.
    assert_eq!(
        c[0].expectations.result[0].abi_type.kind,
        AbiTypeKind::Failure
    );
    assert_eq!(c[0].expectations.result.len(), 5);
}

#[test]
fn call_expectations_empty_single_line() {
    let c = parse("// _exp_() ->");
    assert_eq!(c[0].signature, "_exp_()");
    assert_eq!(c[0].display_mode, DisplayMode::SingleLine);
    assert!(!c[0].expectations.failure);
}

#[test]
fn call_expectations_empty_multiline() {
    let c = parse("// _exp_()\n// ->");
    assert_eq!(c[0].signature, "_exp_()");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
}

#[test]
fn call_comments() {
    let c = parse(
        "// f() # Parameter comment # -> 1 # Expectation comment #\n\
         // f() # Parameter comment #\n\
         // -> 1 # Expectation comment #",
    );
    assert_eq!(c.len(), 2);
    assert_eq!(c[0].display_mode, DisplayMode::SingleLine);
    assert_eq!(c[0].arguments.comment, " Parameter comment ");
    assert_eq!(c[0].expectations.comment, " Expectation comment ");
    assert_eq!(c[1].display_mode, DisplayMode::MultiLine);
    assert_eq!(c[1].arguments.comment, " Parameter comment ");
    assert_eq!(c[1].expectations.comment, " Expectation comment ");
}

#[test]
fn call_arguments_wei() {
    let c = parse("// f(uint256), 314 wei: 5 # optional wei value #\n// -> 4");
    assert_eq!(c[0].signature, "f(uint256)");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert_eq!(
        c[0].value,
        Some(FunctionValue {
            digits: "314".into(),
            unit: ValueUnit::Wei
        })
    );
    assert_eq!(c[0].arguments.parameters[0].literal, dec(false, "5"));
    assert_eq!(c[0].arguments.comment, " optional wei value ");
    assert_eq!(c[0].expectations.result[0].literal, dec(false, "4"));
}

#[test]
fn call_arguments_ether() {
    let c = parse("// f(uint256), 1 ether: 5 # optional ether value #\n// -> 4");
    assert_eq!(
        c[0].value,
        Some(FunctionValue {
            digits: "1".into(),
            unit: ValueUnit::Ether
        })
    );
    assert_eq!(c[0].arguments.comment, " optional ether value ");
}

#[test]
fn call_arguments_bool() {
    let c = parse("// f(bool): true -> false");
    assert_eq!(c[0].signature, "f(bool)");
    assert_eq!(c[0].arguments.parameters[0].literal, Literal::Boolean(true));
    assert_eq!(
        c[0].arguments.parameters[0].abi_type.kind,
        AbiTypeKind::Boolean
    );
    assert_eq!(c[0].expectations.result[0].literal, Literal::Boolean(false));
}

#[test]
fn scanner_hex_values() {
    let c = parse("// f(uint256): \"\\x20\\x00\\xFf\" ->");
    let p = &c[0].arguments.parameters[0];
    assert_eq!(p.literal, Literal::Text(vec![0x20, 0x00, 0xff]));
    assert_eq!(p.abi_type.kind, AbiTypeKind::String);
    assert_eq!(p.abi_type.size, 3);
}

#[test]
fn scanner_hex_values_single_digit() {
    let c = parse("// f(uint256): \"\\x1\" ->");
    assert_eq!(
        c[0].arguments.parameters[0].literal,
        Literal::Text(vec![0x01])
    );
}

#[test]
fn call_arguments_hex_string() {
    let c = parse("// f(bytes): hex\"4200ef\" -> hex\"ab0023\"");
    assert_eq!(c[0].signature, "f(bytes)");
    let a = &c[0].arguments.parameters[0];
    assert_eq!(a.literal, Literal::HexString(vec![0x42, 0x00, 0xef]));
    assert_eq!(a.abi_type.kind, AbiTypeKind::HexString);
    assert_eq!(a.abi_type.size, 3);
    assert_eq!(
        c[0].expectations.result[0].literal,
        Literal::HexString(vec![0xab, 0x00, 0x23])
    );
}

#[test]
fn call_arguments_string() {
    let c = parse("// f(string): 0x20, 3, \"any\" ->");
    let p = &c[0].arguments.parameters;
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].literal, Literal::HexNumber("0x20".into()));
    assert_eq!(p[1].literal, dec(false, "3"));
    assert_eq!(p[2].literal, Literal::Text(b"any".to_vec()));
}

#[test]
fn call_hex_number() {
    let c = parse("// f(bytes32, bytes32): 0x616, 0x1042 -> 1");
    assert_eq!(c[0].signature, "f(bytes32,bytes32)");
    assert_eq!(
        c[0].arguments.parameters[0].literal,
        Literal::HexNumber("0x616".into())
    );
    assert_eq!(
        c[0].arguments.parameters[1].literal,
        Literal::HexNumber("0x1042".into())
    );
    assert_eq!(c[0].expectations.result[0].literal, dec(false, "1"));
}

#[test]
fn call_return_string() {
    let c = parse("// f() -> 0x20, 3, \"any\"");
    let r = &c[0].expectations.result;
    assert_eq!(r[0].literal, Literal::HexNumber("0x20".into()));
    assert_eq!(r[2].literal, Literal::Text(b"any".to_vec()));
}

#[test]
fn call_arguments_tuple() {
    let c = parse("// f((uint256, bytes32), uint256) ->\n// f((uint8), uint8) ->");
    assert_eq!(c[0].signature, "f((uint256,bytes32),uint256)");
    assert_eq!(c[1].signature, "f((uint8),uint8)");
}

#[test]
fn call_arguments_left_right_aligned() {
    let c = parse("// f(bytes32, bytes32): 0x6161, 0x420000EF -> 1");
    assert_eq!(
        c[0].arguments.parameters[0].literal,
        Literal::HexNumber("0x6161".into())
    );
    assert_eq!(
        c[0].arguments.parameters[1].literal,
        Literal::HexNumber("0x420000EF".into())
    );
}

#[test]
fn call_arguments_tuple_of_tuples() {
    let c = parse(
        "// f(((uint256, bytes32), bytes32), uint256)\n// # f(S memory s, uint256 b) #\n// ->",
    );
    assert_eq!(c[0].signature, "f(((uint256,bytes32),bytes32),uint256)");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert_eq!(c[0].arguments.comment, " f(S memory s, uint256 b) ");
}

#[test]
fn call_arguments_recursive_tuples() {
    let c = parse(
        "// f(((((bytes, bytes, bytes), bytes), bytes), bytes), bytes) ->\n\
         // f(((((bytes, bytes, (bytes)), bytes), bytes), (bytes, bytes)), (bytes, bytes)) ->",
    );
    assert_eq!(
        c[0].signature,
        "f(((((bytes,bytes,bytes),bytes),bytes),bytes),bytes)"
    );
    assert_eq!(
        c[1].signature,
        "f(((((bytes,bytes,(bytes)),bytes),bytes),(bytes,bytes)),(bytes,bytes))"
    );
}

#[test]
fn call_arguments_mismatch_allowed() {
    let c = parse("// f(uint256):\n// 1, 2\n// # This only throws at runtime #\n// -> 1");
    assert_eq!(c[0].signature, "f(uint256)");
    assert_eq!(c[0].arguments.parameters.len(), 2);
    assert_eq!(c[0].arguments.comment, " This only throws at runtime ");
    assert_eq!(c[0].expectations.result.len(), 1);
}

#[test]
fn call_multiple_arguments() {
    let c = parse("// test(uint256, uint256):\n// 1,\n// 2\n// -> 1,\n// 1");
    assert_eq!(c[0].signature, "test(uint256,uint256)");
    assert_eq!(c[0].display_mode, DisplayMode::MultiLine);
    assert_eq!(c[0].arguments.parameters.len(), 2);
    assert_eq!(c[0].expectations.result.len(), 2);
}

#[test]
fn call_multiple_arguments_signed() {
    let c = parse("// test(uint256, uint256), 314 wei:\n// 1, -2\n// -> -1, 2");
    assert_eq!(
        c[0].value,
        Some(FunctionValue {
            digits: "314".into(),
            unit: ValueUnit::Wei
        })
    );
    assert_eq!(c[0].arguments.parameters[0].literal, dec(false, "1"));
    assert_eq!(c[0].arguments.parameters[1].literal, dec(true, "2"));
    assert_eq!(
        c[0].arguments.parameters[1].abi_type.kind,
        AbiTypeKind::SignedDec
    );
    assert_eq!(c[0].expectations.result[0].literal, dec(true, "1"));
    assert_eq!(c[0].expectations.result[1].literal, dec(false, "2"));
}

#[test]
fn call_signature_array() {
    let c = parse("// f(uint256[]) ->\n// f(uint256[3]) ->\n// f(uint256[3][][], uint8[9]) ->");
    assert_eq!(c[0].signature, "f(uint256[])");
    assert_eq!(c[1].signature, "f(uint256[3])");
    assert_eq!(c[2].signature, "f(uint256[3][][],uint8[9])");
}

#[test]
fn call_signature_struct_array() {
    let c = parse(
        "// f((uint256)[]) ->\n\
         // f((uint256)[3]) ->\n\
         // f((uint256, uint8)[3]) ->\n\
         // f((uint256)[3][][], (uint8, bool)[9]) ->",
    );
    assert_eq!(c[0].signature, "f((uint256)[])");
    assert_eq!(c[2].signature, "f((uint256,uint8)[3])");
    assert_eq!(c[3].signature, "f((uint256)[3][][],(uint8,bool)[9])");
}

#[test]
fn call_signature_valid_any_types() {
    let c = parse("// f(uint256, uint8, string) -> FAILURE\n// f(invalid, xyz, foo) -> FAILURE");
    assert_eq!(c[0].signature, "f(uint256,uint8,string)");
    assert_eq!(c[1].signature, "f(invalid,xyz,foo)");
    assert!(c[1].expectations.failure);
}

#[test]
fn call_raw_arguments() {
    let c = parse("// f(): 1, -2, -3 ->");
    let p = &c[0].arguments.parameters;
    assert_eq!(p[0].raw_string, "1");
    assert_eq!(p[1].raw_string, "-2");
    assert_eq!(p[2].raw_string, "-3");
    assert_eq!(p[1].literal, dec(true, "2"));
}

#[test]
fn call_builtin_left_aligned() {
    let c = parse("// f(): left(1), left(0x20) -> left(-2), left(true)");
    let a = &c[0].arguments.parameters;
    assert_eq!(a[0].alignment, Alignment::Left);
    assert_eq!(a[0].literal, dec(false, "1"));
    assert_eq!(a[0].raw_string, "left(1)");
    assert_eq!(a[1].alignment, Alignment::Left);
    assert_eq!(a[1].literal, Literal::HexNumber("0x20".into()));
    let e = &c[0].expectations.result;
    assert_eq!(e[0].alignment, Alignment::Left);
    assert_eq!(e[0].literal, dec(true, "2"));
    assert_eq!(e[0].raw_string, "left(-2)");
    assert_eq!(e[1].literal, Literal::Boolean(true));
}

#[test]
fn call_builtin_right_aligned() {
    let c = parse("// f(): right(1), right(0x20) -> right(-2), right(true)");
    assert_eq!(c[0].arguments.parameters[0].alignment, Alignment::Right);
    assert_eq!(c[0].expectations.result[0].alignment, Alignment::Right);
    assert_eq!(c[0].expectations.result[0].literal, dec(true, "2"));
}

#[test]
fn constructor() {
    let c = parse("// constructor()");
    assert_eq!(c[0].signature, "constructor()");
    assert_eq!(c[0].kind, Kind::Constructor);
    assert!(c[0].omits_arrow);
}

#[test]
fn library() {
    let c = parse("// library: L");
    assert_eq!(c[0].signature, "L");
    assert_eq!(c[0].kind, Kind::Library);
    assert!(!c[0].expectations.failure);
}

#[test]
fn low_level_call() {
    let c = parse("// (): hex\"00\" ->");
    assert_eq!(c[0].kind, Kind::LowLevel);
    assert_eq!(c[0].signature, "()");
}

#[test]
fn builtin_bare_identifier() {
    let c = parse_b("// storageEmpty -> 1", &["storageEmpty"]);
    assert_eq!(c[0].signature, "storageEmpty");
    assert_eq!(c[0].kind, Kind::Builtin);
    assert_eq!(c[0].expectations.result[0].literal, dec(false, "1"));
}

#[test]
fn builtin_account_with_colon_arg() {
    let c = parse_b("// account: 0", &["account"]);
    assert_eq!(c[0].signature, "account");
    assert_eq!(c[0].kind, Kind::Builtin);
    assert_eq!(c[0].arguments.parameters[0].literal, dec(false, "0"));
}

#[test]
fn call_effects_single() {
    let c = parse_b(
        "// builtin_returning_call_effect -> 1\n// ~ bla\n// ~ bla bla\n// ~ bla bla bla",
        &["builtin_returning_call_effect"],
    );
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].expectations.result[0].literal, dec(false, "1"));
    assert_eq!(
        c[0].expected_side_effects,
        vec!["bla", "bla bla", "bla bla bla"]
    );
}

#[test]
fn call_effects_distributed_over_calls() {
    let c = parse_b(
        "// builtin_returning_call_effect -> 1\n\
         // ~ bla\n\
         // ~ bla bla\n\
         // builtin_returning_call_effect -> 2\n\
         // ~ bla bla bla\n\
         // builtin_returning_call_effect -> 3",
        &["builtin_returning_call_effect"],
    );
    assert_eq!(c.len(), 3);
    assert_eq!(c[0].expected_side_effects, vec!["bla", "bla bla"]);
    assert_eq!(c[1].expected_side_effects, vec!["bla bla bla"]);
    assert!(c[2].expected_side_effects.is_empty());
}

#[test]
fn call_effects_interior_tildes_preserved() {
    let c = parse_b(
        "// builtin_returning_call_effect -> 1\n// ~ abc ~ def ~ ghi\n// ~ ~ ~",
        &["builtin_returning_call_effect"],
    );
    assert_eq!(c[0].expected_side_effects, vec!["abc ~ def ~ ghi", "~ ~"]);
}

// ---------- REJECT ----------

#[test]
fn reject_scanner_hex_values_invalid1() {
    reject("// f(uint256): \"\\x\" ->");
}

#[test]
fn reject_scanner_hex_values_invalid3() {
    reject("// f(uint256): \"\\xZ\" ->");
}

#[test]
fn reject_scanner_hex_values_invalid4() {
    reject("// f(uint256): \"\\xZZ\" ->");
}

#[test]
fn reject_hex_string_left_align() {
    reject("// f(bytes): left(hex\"4200ef\") ->");
}

#[test]
fn reject_hex_string_right_align() {
    reject("// f(bytes): right(hex\"4200ef\") ->");
}

#[test]
fn reject_newline_invalid() {
    reject("/");
}

#[test]
fn reject_call_invalid() {
    reject("/ f() ->");
}

#[test]
fn reject_signature_invalid_trailing_comma() {
    reject("// f(uint8,) -> FAILURE");
}

#[test]
fn reject_tuple_invalid_unclosed() {
    reject("// f((uint8,) -> FAILURE");
}

#[test]
fn reject_tuple_invalid_empty() {
    reject("// f(uint8, ()) -> FAILURE");
}

#[test]
fn reject_tuple_invalid_parentheses() {
    reject("// f((uint8,() -> FAILURE");
}

#[test]
fn reject_ether_value_expectations_missing() {
    reject("// f(), 0)");
}

#[test]
fn reject_arguments_invalid() {
    reject("// f(uint256): abc -> 1");
}

#[test]
fn reject_arguments_invalid_decimal() {
    reject("// sig(): 0.h3 ->");
}

#[test]
fn reject_ether_value_invalid() {
    reject("// f(uint256), abc : 1 -> 1");
}

#[test]
fn reject_ether_value_invalid_decimal() {
    reject("// sig(): 0.1hd ether ->");
}

#[test]
fn reject_ether_type_invalid() {
    reject("// f(uint256), 2 btc : 1 -> 1");
}

#[test]
fn reject_signed_bool_invalid() {
    reject("// f() -> -true");
}

#[test]
fn reject_signed_failure_invalid() {
    reject("// f() -> -FAILURE");
}

#[test]
fn reject_signed_hex_number_invalid() {
    reject("// f() -> -0x42");
}

#[test]
fn reject_arguments_colon() {
    reject("// h256():\n// -> 1");
}

#[test]
fn reject_arguments_newline_colon() {
    reject("// h256()\n// :\n// -> 1");
}

#[test]
fn reject_arrow_missing() {
    reject("// h256() FAILURE");
}

#[test]
fn reject_unexpected_character() {
    reject("// f() -> ??");
}

#[test]
fn reject_side_effect_with_slash() {
    reject_b(
        "// builtin_returning_call_effect_no_ret # another comment #\n// ~ hello/world",
        &["builtin_returning_call_effect_no_ret"],
    );
}
