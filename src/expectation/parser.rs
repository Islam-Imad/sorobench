//! Recursive-descent parser for the // ---- expectation DSL, a port of solc's
//! TestFileParser.
//!
//! This records the structure of each call plus each parameter's detected
//! AbiType and decoded Literal. It does not turn tokens into ABI bytes
//! (alignment, two's complement, etc.); that is done later by the decoder.

use std::collections::BTreeMap;

use super::ast::*;
use super::scanner::{Scanner, Token};
use super::{Builtins, PResult, ParseError};

const GAS_RUN_TYPES: [&str; 4] = ["ir", "irOptimized", "legacy", "legacyOptimized"];

pub struct Parser<'b> {
    scanner: Scanner,
    line_number: usize,
    builtins: &'b Builtins,
}

impl<'b> Parser<'b> {
    pub fn new(source: &str, builtins: &'b Builtins) -> Parser<'b> {
        Parser {
            scanner: Scanner::new(source),
            line_number: 0,
            builtins,
        }
    }

    // Parse the whole block into a list of calls (mirror of parseFunctionCalls).
    // line_offset seeds error line numbers.
    pub fn parse_function_calls(&mut self, line_offset: usize) -> PResult<Vec<FunctionCall>> {
        let mut calls: Vec<FunctionCall> = Vec::new();
        if self.scanner.current_token() == Token::Eos {
            return Ok(calls);
        }
        // Initial token is Unknown; prime the scanner.
        self.scanner.scan_next_token()?;
        while self.scanner.current_token() != Token::Eos {
            if self.scanner.current_token() == Token::Whitespace {
                // The scanner never yields Whitespace (it is skipped internally);
                // guard anyway to avoid a spin.
                self.scanner.scan_next_token()?;
                continue;
            }
            // Newline handling sits *outside* the error-prefix wrapper in the C++.
            if calls.is_empty() {
                self.expect(Token::Newline, true)?;
            } else if self.accept(Token::Newline, true)? {
                self.line_number += 1;
            }
            self.parse_entry(&mut calls).map_err(|e| {
                ParseError::new(format!(
                    "Line {}: {}",
                    line_offset + self.line_number,
                    e.message
                ))
            })?;
        }
        Ok(calls)
    }

    // One gas line or one function call (the try-wrapped body of the loop).
    fn parse_entry(&mut self, calls: &mut Vec<FunctionCall>) -> PResult<()> {
        if self.accept(Token::Gas, true)? {
            if calls.is_empty() {
                return Err(ParseError::new(
                    "Expected function call before gas usage filter.",
                ));
            }
            let run_type = self.scanner.current_literal_string();
            if GAS_RUN_TYPES.contains(&run_type.as_str()) {
                self.scanner.scan_next_token()?;
                self.expect(Token::Colon, true)?;
                let call = calls.last_mut().unwrap();
                if call.expectations.gas_used.contains_key(&run_type) {
                    return Err(ParseError::new("Gas usage expectation set multiple times."));
                }
                let n = self.parse_decimal_number()?;
                call.expectations.gas_used.insert(run_type, n);
            } else {
                return Err(ParseError::new(
                    "Expected \"ir\", \"irOptimized\", \"legacy\", or \"legacyOptimized\".",
                ));
            }
            return Ok(());
        }

        let mut call = FunctionCall::new();
        if self.accept(Token::Library, true)? {
            self.expect(Token::Colon, true)?;
            let library_name;
            if self.accept(Token::String, false)? {
                call.library_file = self.scanner.current_literal_string();
                self.expect(Token::String, true)?;
                self.expect(Token::Colon, true)?;
                library_name = self.scanner.current_literal_string();
                self.expect(Token::Identifier, true)?;
            } else if self.accept(Token::Colon, true)? {
                library_name = self.scanner.current_literal_string();
                self.expect(Token::Identifier, true)?;
            } else {
                library_name = self.scanner.current_literal_string();
                self.expect(Token::Identifier, true)?;
            }
            call.signature = library_name;
            call.kind = Kind::Library;
            call.expectations.failure = false;
        } else {
            let (signature, low_level) = self.parse_function_signature()?;
            call.signature = signature;
            if low_level {
                call.kind = Kind::LowLevel;
            } else if self.is_builtin_function(&call.signature) {
                call.kind = Kind::Builtin;
            }

            if self.accept(Token::Comma, true)? {
                call.value = Some(self.parse_function_call_value()?);
            }
            if self.accept(Token::Colon, true)? {
                call.arguments = self.parse_function_call_arguments()?;
            }
            if self.accept(Token::Newline, true)? {
                call.display_mode = DisplayMode::MultiLine;
                self.line_number += 1;
            }
            call.arguments.comment = self.parse_comment()?;
            if self.accept(Token::Newline, true)? {
                call.display_mode = DisplayMode::MultiLine;
                self.line_number += 1;
            }
            if self.accept(Token::Arrow, true)? {
                call.omits_arrow = false;
                call.expectations = self.parse_function_call_expectations()?;
                if self.accept(Token::Newline, true)? {
                    self.line_number += 1;
                }
            } else {
                call.expectations.failure = false;
                call.display_mode = DisplayMode::SingleLine;
            }
            call.expectations.comment = self.parse_comment()?;
            if call.signature == "constructor()" {
                call.kind = Kind::Constructor;
            }
        }

        self.accept(Token::Newline, true)?;
        call.expected_side_effects = self.parse_function_call_side_effects()?;
        calls.push(call);
        Ok(())
    }

    // token helpers (mirror of accept/expect)

    fn accept(&mut self, token: Token, do_expect: bool) -> PResult<bool> {
        if self.scanner.current_token() != token {
            return Ok(false);
        }
        if do_expect {
            self.expect(token, true)?;
        }
        Ok(true)
    }

    fn expect(&mut self, token: Token, advance: bool) -> PResult<()> {
        let cur = self.scanner.current_token();
        if cur != token || cur == Token::Invalid {
            return Err(ParseError::new(format!(
                "Unexpected {}: \"{}\". Expected \"{}\".",
                cur.format(),
                self.scanner.current_literal_string(),
                token.format()
            )));
        }
        if advance {
            self.scanner.scan_next_token()?;
        }
        Ok(())
    }

    // productions

    fn parse_function_signature(&mut self) -> PResult<(String, bool)> {
        let mut signature = String::new();
        let mut has_name = false;
        if self.accept(Token::Identifier, false)? {
            has_name = true;
            signature = self.scanner.current_literal_string();
            self.expect(Token::Identifier, true)?;
        }

        if self.is_builtin_function(&signature) && self.scanner.current_token() != Token::LParen {
            return Ok((signature, false));
        }

        signature.push('(');
        self.expect(Token::LParen, true)?;

        let mut parameters = String::new();
        if !self.accept(Token::RParen, false)? {
            parameters = self.parse_identifier_or_tuple()?;
        }
        while self.accept(Token::Comma, false)? {
            parameters.push(',');
            self.expect(Token::Comma, true)?;
            parameters += &self.parse_identifier_or_tuple()?;
        }
        if self.accept(Token::Arrow, true)? {
            return Err(ParseError::new(format!(
                "Invalid signature detected: {signature}"
            )));
        }
        if !has_name && !parameters.is_empty() {
            return Err(ParseError::new(format!(
                "Signatures without a name cannot have parameters: {signature}"
            )));
        }
        signature += &parameters;
        self.expect(Token::RParen, true)?;
        signature.push(')');
        Ok((signature, !has_name))
    }

    fn parse_function_call_value(&mut self) -> PResult<FunctionValue> {
        let digits = self.parse_decimal_number()?;
        let tok = self.scanner.current_token();
        let unit = match tok {
            Token::Wei => ValueUnit::Wei,
            Token::Ether => ValueUnit::Ether,
            _ => {
                return Err(ParseError::new(
                    "Invalid value unit provided. Coins can be wei or ether.",
                ))
            }
        };
        self.scanner.scan_next_token()?;
        Ok(FunctionValue { digits, unit })
    }

    fn parse_function_call_arguments(&mut self) -> PResult<FunctionCallArgs> {
        let mut args = FunctionCallArgs::default();
        let param = self.parse_parameter()?;
        if param.abi_type.kind == AbiTypeKind::None {
            return Err(ParseError::new("No argument provided."));
        }
        args.parameters.push(param);
        while self.accept(Token::Comma, true)? {
            let p = self.parse_parameter()?;
            args.parameters.push(p);
        }
        Ok(args)
    }

    fn parse_function_call_expectations(&mut self) -> PResult<FunctionCallExpectations> {
        let mut expectations = FunctionCallExpectations::default();
        let param = self.parse_parameter()?;
        if param.abi_type.kind == AbiTypeKind::None {
            expectations.failure = false;
            return Ok(expectations);
        }
        expectations.result.push(param);
        while self.accept(Token::Comma, true)? {
            let p = self.parse_parameter()?;
            expectations.result.push(p);
        }
        // A leading FAILURE marks the whole call as expecting a revert
        if expectations.result[0].abi_type.kind != AbiTypeKind::Failure {
            expectations.failure = false;
        }
        Ok(expectations)
    }

    fn parse_parameter(&mut self) -> PResult<Parameter> {
        let mut param = Parameter {
            raw_string: String::new(),
            abi_type: AbiType::none(),
            alignment: Alignment::None,
            newline: false,
            literal: Literal::None,
        };
        if self.accept(Token::Newline, true)? {
            param.newline = true;
            self.line_number += 1;
        }

        let mut is_signed = false;
        if self.accept(Token::Left, true)? {
            param.raw_string.push_str("left(");
            self.expect(Token::LParen, true)?;
            param.alignment = Alignment::Left;
        }
        if self.accept(Token::Right, true)? {
            param.raw_string.push_str("right(");
            self.expect(Token::LParen, true)?;
            param.alignment = Alignment::Right;
        }
        if self.accept(Token::Sub, true)? {
            param.raw_string.push('-');
            is_signed = true;
        }

        if self.accept(Token::Boolean, false)? {
            if is_signed {
                return Err(ParseError::new("Invalid boolean literal."));
            }
            param.abi_type = AbiType {
                kind: AbiTypeKind::Boolean,
                align: Align::Right,
                size: 32,
                fractional_digits: 0,
            };
            let parsed = self.parse_boolean()?;
            param.raw_string.push_str(&parsed);
            param.literal = Literal::Boolean(parsed == "true");
        } else if self.accept(Token::HexNumber, false)? {
            if is_signed {
                return Err(ParseError::new("Invalid hex number literal."));
            }
            param.abi_type = AbiType {
                kind: AbiTypeKind::Hex,
                align: Align::Right,
                size: 32,
                fractional_digits: 0,
            };
            let parsed = self.parse_hex_number()?;
            param.raw_string.push_str(&parsed);
            param.literal = Literal::HexNumber(parsed);
        } else if self.accept(Token::Hex, true)? {
            if is_signed {
                return Err(ParseError::new("Invalid hex string literal."));
            }
            if param.alignment != Alignment::None {
                return Err(ParseError::new(
                    "Hex string literals cannot be aligned or padded.",
                ));
            }
            let digits = self.parse_string()?;
            let digits_str = String::from_utf8_lossy(&digits).into_owned();
            param.raw_string.push_str(&format!("hex\"{digits_str}\""));
            let bytes = hex_decode(&digits_str);
            param.abi_type = AbiType {
                kind: AbiTypeKind::HexString,
                align: Align::None,
                size: bytes.len(),
                fractional_digits: 0,
            };
            param.literal = Literal::HexString(bytes);
        } else if self.accept(Token::String, false)? {
            if is_signed {
                return Err(ParseError::new("Invalid string literal."));
            }
            if param.alignment != Alignment::None {
                return Err(ParseError::new(
                    "String literals cannot be aligned or padded.",
                ));
            }
            let parsed = self.parse_string()?;
            param.abi_type = AbiType {
                kind: AbiTypeKind::String,
                align: Align::Left,
                size: parsed.len(),
                fractional_digits: 0,
            };
            param
                .raw_string
                .push_str(&format!("\"{}\"", String::from_utf8_lossy(&parsed)));
            param.literal = Literal::Text(parsed);
        } else if self.accept(Token::Number, false)? {
            let kind = if is_signed {
                AbiTypeKind::SignedDec
            } else {
                AbiTypeKind::UnsignedDec
            };
            param.abi_type = AbiType {
                kind,
                align: Align::Right,
                size: 32,
                fractional_digits: 0,
            };
            let parsed = self.parse_decimal_number()?;
            param.raw_string.push_str(&parsed);
            if parsed.contains('.') {
                param.abi_type.kind = if is_signed {
                    AbiTypeKind::SignedFixedPoint
                } else {
                    AbiTypeKind::UnsignedFixedPoint
                };
                // The fractional-digit count and ABI bytes are computed later by the decoder.
            }
            param.literal = Literal::Decimal {
                negative: is_signed,
                digits: parsed,
            };
        } else if self.accept(Token::Failure, true)? {
            if is_signed {
                return Err(ParseError::new("Invalid failure literal."));
            }
            param.abi_type = AbiType {
                kind: AbiTypeKind::Failure,
                align: Align::Right,
                size: 0,
                fractional_digits: 0,
            };
            param.literal = Literal::Failure;
        }

        if param.alignment != Alignment::None {
            self.expect(Token::RParen, true)?;
            param.raw_string.push(')');
        }
        Ok(param)
    }

    fn parse_identifier_or_tuple(&mut self) -> PResult<String> {
        let mut s = String::new();
        if self.accept(Token::Identifier, false)? {
            s = self.scanner.current_literal_string();
            self.expect(Token::Identifier, true)?;
            self.parse_array_dimensions(&mut s)?;
            return Ok(s);
        }
        self.expect(Token::LParen, true)?;
        s.push('(');
        s += &self.parse_identifier_or_tuple()?;
        while self.accept(Token::Comma, false)? {
            s.push(',');
            self.expect(Token::Comma, true)?;
            s += &self.parse_identifier_or_tuple()?;
        }
        self.expect(Token::RParen, true)?;
        s.push(')');
        self.parse_array_dimensions(&mut s)?;
        Ok(s)
    }

    fn parse_array_dimensions(&mut self, s: &mut String) -> PResult<()> {
        while self.accept(Token::LBrack, false)? {
            s.push('[');
            self.expect(Token::LBrack, true)?;
            if self.accept(Token::Number, false)? {
                *s += &self.parse_decimal_number()?;
            }
            s.push(']');
            self.expect(Token::RBrack, true)?;
        }
        Ok(())
    }

    fn parse_boolean(&mut self) -> PResult<String> {
        let literal = self.scanner.current_literal_string();
        self.expect(Token::Boolean, true)?;
        Ok(literal)
    }

    fn parse_comment(&mut self) -> PResult<String> {
        let comment = self.scanner.current_literal_string();
        if self.accept(Token::Comment, true)? {
            Ok(comment)
        } else {
            Ok(String::new())
        }
    }

    fn parse_decimal_number(&mut self) -> PResult<String> {
        let literal = self.scanner.current_literal_string();
        self.expect(Token::Number, true)?;
        Ok(literal)
    }

    fn parse_hex_number(&mut self) -> PResult<String> {
        let literal = self.scanner.current_literal_string();
        self.expect(Token::HexNumber, true)?;
        Ok(literal)
    }

    fn parse_string(&mut self) -> PResult<Vec<u8>> {
        let literal = self.scanner.current_literal().to_vec();
        self.expect(Token::String, true)?;
        Ok(literal)
    }

    fn parse_function_call_side_effects(&mut self) -> PResult<Vec<String>> {
        let mut result = Vec::new();
        while self.accept(Token::Tilde, false)? {
            result.push(self.scanner.current_literal_string());
            self.scanner.scan_next_token()?;
            if self.scanner.current_token() == Token::Newline {
                self.scanner.scan_next_token()?;
            }
        }
        Ok(result)
    }

    fn is_builtin_function(&self, signature: &str) -> bool {
        self.builtins.contains(signature)
    }
}

// Hex-decode a hex"..." digit string. Best-effort here; the decoder's
// bytes_utils is authoritative.
fn hex_decode(digits: &str) -> Vec<u8> {
    let d = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
        .unwrap_or(digits);
    let padded = if d.len() % 2 == 1 {
        format!("0{d}")
    } else {
        d.to_string()
    };
    let bytes = padded.as_bytes();
    (0..bytes.len())
        .step_by(2)
        .filter_map(|i| {
            let hi = (bytes[i] as char).to_digit(16)?;
            let lo = (bytes[i + 1] as char).to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}

// Silence "unused" for the map type until later consumers land.
#[allow(dead_code)]
type _GasMap = BTreeMap<String, String>;
