//! The decoder bridges two value representations through one common NativeValue
//! so they can be compared:
//! - expected (the // ---- tokens): bytes_utils turns them into an ABI byte
//!   buffer, then abi decodes that into a NativeValue.
//! - actual (a Soroban Val): val::from_val decodes it into a NativeValue.
//! - arguments: val::to_val encodes a NativeValue into a Val.
//!
//! Comparison is done on NativeValues, never on raw Vals, because two Soroban
//! Vals that wrap host objects compare by handle rather than by content.
//! The tests check the round-trip from_val(to_val(x)) == x, and that a decoded
//! expected value equals a decoded actual value.

pub mod abi;
pub mod bytes_utils;
pub mod native;
pub mod nofaithful;
pub mod types;
pub mod val;

#[cfg(test)]
mod tests;

pub use native::NativeValue;
pub use nofaithful::NoFaithful;
pub use types::SorobanType;
