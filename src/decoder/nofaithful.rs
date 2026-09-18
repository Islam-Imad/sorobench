//! Detects values that have no faithful Soroban equivalent — they are neither a
//! pass nor a fail, so we label them instead of forcing a verdict.

use super::native::NativeValue;
use super::types::SorobanType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoFaithful {
    /// A 20-byte EVM `address` result — Soroban addresses are 32-byte and do not
    /// correspond, except via the indexed `account(N)` mechanism.
    Address20,
    /// A `uint256` result hinging on the exact 2²⁵⁶ wrap boundary.
    Uint256Wrap,
    /// A `bytes32` used as a keccak identity (keccak is EVM-only).
    Bytes32KeccakIdentity,
}

/// Flag a decoded expected value that has no faithful Soroban equivalent.
///
/// This only handles the case that can be decided from the value alone (a 20-byte
/// `address`). The `Uint256Wrap` / `Bytes32KeccakIdentity` cases depend on how the
/// value is *used*, not just its shape, so the caller surfaces those when known;
/// here we classify only by (type, decoded value).
pub fn classify(ty: &SorobanType, native: &NativeValue) -> Option<NoFaithful> {
    match (ty, native) {
        (SorobanType::Address, NativeValue::Address(bytes)) if bytes.len() == 20 => {
            Some(NoFaithful::Address20)
        }
        // A bare `address` type is treated as the 20≠32 boundary regardless of
        // the decoded length, since the two address spaces do not correspond.
        (SorobanType::Address, _) => Some(NoFaithful::Address20),
        _ => None,
    }
}
