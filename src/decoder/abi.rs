//! Decode an ABI byte buffer into a NativeValue using alloy-dyn-abi. The ABI
//! type string comes from the compiled function's signature and is passed in.

use alloy_dyn_abi::{DynSolType, DynSolValue};

use super::native::NativeValue;

pub fn abi_decode_params(buffer: &[u8], abi_type: &str) -> Result<NativeValue, String> {
    let ty = DynSolType::parse(abi_type).map_err(|e| e.to_string())?;
    let val = ty.abi_decode_params(buffer).map_err(|e| e.to_string())?;
    Ok(from_dyn(&val))
}

pub fn from_dyn(v: &DynSolValue) -> NativeValue {
    match v {
        DynSolValue::Bool(b) => NativeValue::Bool(*b),
        DynSolValue::Int(i, _) => NativeValue::Int(i.into_raw()),
        DynSolValue::Uint(u, _) => NativeValue::Int(*u),
        DynSolValue::FixedBytes(word, n) => NativeValue::Bytes(word.as_slice()[..*n].to_vec()),
        DynSolValue::Address(a) => NativeValue::Address(a.as_slice().to_vec()),
        DynSolValue::Bytes(b) => NativeValue::Bytes(b.clone()),
        DynSolValue::String(s) => NativeValue::Str(s.clone().into_bytes()),
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) => {
            NativeValue::Vec(items.iter().map(from_dyn).collect())
        }
        DynSolValue::Tuple(items) => NativeValue::Tuple(items.iter().map(from_dyn).collect()),
        DynSolValue::Function(_) => NativeValue::Bytes(Vec::new()),
    }
}
