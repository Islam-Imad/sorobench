use crate::expectation::ast::{Alignment, Literal, Parameter};
use alloy_primitives::U256;

/// Materialize one parameter to its ABI bytes.
pub fn materialize(param: &Parameter) -> Vec<u8> {
    match &param.literal {
        Literal::None | Literal::Failure => Vec::new(),
        Literal::Boolean(b) => apply_align(param.alignment, vec![u8::from(*b)]),
        Literal::HexNumber(s) => apply_align(param.alignment, convert_hex_number(s)),
        Literal::Decimal { negative, digits } => {
            if digits.contains('.') {
                convert_fixed_point(*negative, digits)
            } else {
                let signed = if *negative {
                    format!("-{digits}")
                } else {
                    digits.clone()
                };
                apply_align(param.alignment, convert_number(&signed))
            }
        }
        Literal::Text(bytes) => apply_align(Alignment::Left, bytes.clone()),
        Literal::HexString(bytes) => bytes.clone(),
    }
}

/// Concatenate a parameter list into one flat ABI buffer.
pub fn encode_params(params: &[Parameter]) -> Vec<u8> {
    params.iter().flat_map(materialize).collect()
}

fn apply_align(alignment: Alignment, bytes: Vec<u8>) -> Vec<u8> {
    assert!(
        bytes.len() <= 32,
        "token is {} bytes, exceeds the 32-byte ABI word (solc alignLeft/alignRight assert <= 32)",
        bytes.len()
    );
    match alignment {
        Alignment::Left => {
            let mut v = bytes;
            v.resize(32, 0);
            v
        }
        _ => {
            let mut v = vec![0u8; 32 - bytes.len()];
            v.extend_from_slice(&bytes);
            v
        }
    }
}

/// `convertNumber`: `toCompactBigEndian(u256{literal})` — compact big-endian of
/// the 256-bit value (two's-complement for a leading `-`). `0` → empty bytes.
fn convert_number(literal: &str) -> Vec<u8> {
    let word = parse_word(literal);
    compact_big_endian(word)
}

/// `convertFixedPoint`: value integer (dot removed, leading zeros stripped),
/// two's-complement if negative, as a **full** 32-byte big-endian word.
fn convert_fixed_point(negative: bool, digits: &str) -> Vec<u8> {
    let value_integer: String = digits.chars().filter(|c| *c != '.').collect();
    let trimmed = value_integer.trim_start_matches('0');
    let mag = U256::from_str_radix(if trimmed.is_empty() { "0" } else { trimmed }, 10)
        .unwrap_or(U256::ZERO);
    let word = if negative {
        U256::ZERO.wrapping_sub(mag)
    } else {
        mag
    };
    word.to_be_bytes::<32>().to_vec()
}

/// `convertHexNumber`: `fromHex` — decode hex digits (front-pad an odd count).
fn convert_hex_number(literal: &str) -> Vec<u8> {
    let d = literal
        .strip_prefix("0x")
        .or_else(|| literal.strip_prefix("0X"))
        .unwrap_or(literal);
    let padded = if d.len() % 2 == 1 {
        format!("0{d}")
    } else {
        d.to_string()
    };
    let b = padded.as_bytes();
    (0..b.len())
        .step_by(2)
        .filter_map(|i| {
            let hi = (b[i] as char).to_digit(16)?;
            let lo = (b[i + 1] as char).to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}

fn parse_word(literal: &str) -> U256 {
    if let Some(mag) = literal.strip_prefix('-') {
        let m = U256::from_str_radix(mag, 10).unwrap_or(U256::ZERO);
        U256::ZERO.wrapping_sub(m) // 2^256 - m
    } else {
        U256::from_str_radix(literal, 10).unwrap_or(U256::ZERO)
    }
}

/// Compact big-endian: 32-byte word with leading zero bytes stripped (`0` → empty).
fn compact_big_endian(word: U256) -> Vec<u8> {
    let be = word.to_be_bytes::<32>();
    let start = be.iter().position(|&b| b != 0).unwrap_or(be.len());
    be[start..].to_vec()
}
