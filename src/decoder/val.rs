use alloy_primitives::U256;
use soroban_sdk::{
    Bytes, Env, IntoVal, Map, String as SString, Symbol, TryFromVal, Val, Vec as SVec,
    I256 as SI256, U256 as SU256,
};

use super::native::NativeValue;
use super::types::SorobanType;

pub fn to_val(env: &Env, native: &NativeValue, ty: &SorobanType) -> Val {
    match ty {
        SorobanType::Bool => bool_of(native).into_val(env),
        SorobanType::U32 => low_u32(native).into_val(env),
        SorobanType::I32 => (low_u32(native) as i32).into_val(env),
        SorobanType::U64 => low_u64(native).into_val(env),
        SorobanType::I64 => (low_u64(native) as i64).into_val(env),
        SorobanType::U128 => low_u128(native).into_val(env),
        SorobanType::I128 => (low_u128(native) as i128).into_val(env),
        SorobanType::U256 => soroban_u256(env, native).into_val(env),
        SorobanType::I256 => soroban_i256(env, native).into_val(env),
        SorobanType::Bytes => Bytes::from_slice(env, bytes_of(native)).into_val(env),
        SorobanType::Str => SString::from_bytes(env, bytes_of(native)).into_val(env),
        SorobanType::Address => panic!("address is NoFaithful — classify, do not to_val"),
        SorobanType::Vec(inner) => {
            let mut v: SVec<Val> = SVec::new(env);
            for item in vec_of(native) {
                v.push_back(to_val(env, item, inner));
            }
            v.into_val(env)
        }
        SorobanType::Struct(fields) => {
            let mut m: Map<Symbol, Val> = Map::new(env);
            for ((name, fty), item) in fields.iter().zip(vec_of(native)) {
                m.set(Symbol::new(env, name), to_val(env, item, fty));
            }
            m.into_val(env)
        }
    }
}

pub fn from_val(env: &Env, val: Val, ty: &SorobanType) -> Result<NativeValue, String> {
    Ok(match ty {
        SorobanType::Bool => {
            NativeValue::Bool(bool::try_from_val(env, &val).map_err(|_| miss("bool"))?)
        }
        SorobanType::U32 => {
            NativeValue::uint_u64(u32::try_from_val(env, &val).map_err(|_| miss("u32"))? as u64)
        }
        SorobanType::I32 => {
            NativeValue::int_i128(i32::try_from_val(env, &val).map_err(|_| miss("i32"))? as i128)
        }
        SorobanType::U64 => {
            NativeValue::uint_u64(u64::try_from_val(env, &val).map_err(|_| miss("u64"))?)
        }
        SorobanType::I64 => {
            NativeValue::int_i128(i64::try_from_val(env, &val).map_err(|_| miss("i64"))? as i128)
        }
        SorobanType::U128 => {
            NativeValue::uint_u128(u128::try_from_val(env, &val).map_err(|_| miss("u128"))?)
        }
        SorobanType::I128 => {
            NativeValue::int_i128(i128::try_from_val(env, &val).map_err(|_| miss("i128"))?)
        }
        SorobanType::U256 => {
            let s = SU256::try_from_val(env, &val).map_err(|_| miss("u256"))?;
            NativeValue::Int(U256::from_be_slice(&bytes_vec(&s.to_be_bytes())))
        }
        SorobanType::I256 => {
            let s = SI256::try_from_val(env, &val).map_err(|_| miss("i256"))?;
            NativeValue::Int(U256::from_be_slice(&bytes_vec(&s.to_be_bytes())))
        }
        SorobanType::Bytes => {
            let b = Bytes::try_from_val(env, &val).map_err(|_| miss("bytes"))?;
            NativeValue::Bytes(bytes_vec(&b))
        }
        SorobanType::Str => {
            let s = SString::try_from_val(env, &val).map_err(|_| miss("string"))?;
            let mut buf = std::vec![0u8; s.len() as usize];
            s.copy_into_slice(&mut buf);
            NativeValue::Str(buf)
        }
        SorobanType::Address => return Err("address is NoFaithful".into()),
        SorobanType::Vec(inner) => {
            let v = SVec::<Val>::try_from_val(env, &val).map_err(|_| miss("vec"))?;
            let mut out = std::vec::Vec::with_capacity(v.len() as usize);
            for i in 0..v.len() {
                out.push(from_val(env, v.get_unchecked(i), inner)?);
            }
            NativeValue::Vec(out)
        }
        SorobanType::Struct(fields) => {
            let m = Map::<Symbol, Val>::try_from_val(env, &val).map_err(|_| miss("struct map"))?;
            let mut out = std::vec::Vec::with_capacity(fields.len());
            for (name, fty) in fields {
                let fv = m
                    .get(Symbol::new(env, name))
                    .ok_or_else(|| format!("struct field `{name}` missing from map"))?;
                out.push(from_val(env, fv, fty)?);
            }
            NativeValue::Tuple(out)
        }
    })
}

fn miss(what: &str) -> String {
    format!("from_val: expected {what}")
}

fn bytes_vec(b: &Bytes) -> std::vec::Vec<u8> {
    b.iter().collect()
}

fn bool_of(n: &NativeValue) -> bool {
    match n {
        NativeValue::Bool(b) => *b,
        _ => panic!("expected NativeValue::Bool, got {n:?}"),
    }
}

fn word_of(n: &NativeValue) -> U256 {
    match n {
        NativeValue::Int(w) => *w,
        _ => panic!("expected NativeValue::Int, got {n:?}"),
    }
}

fn low_u32(n: &NativeValue) -> u32 {
    word_of(n).as_limbs()[0] as u32
}
fn low_u64(n: &NativeValue) -> u64 {
    word_of(n).as_limbs()[0]
}
fn low_u128(n: &NativeValue) -> u128 {
    let w = word_of(n);
    let l = w.as_limbs();
    (l[0] as u128) | ((l[1] as u128) << 64)
}

fn bytes_of(n: &NativeValue) -> &[u8] {
    match n {
        NativeValue::Bytes(b) | NativeValue::Str(b) => b,
        _ => panic!("expected NativeValue::Bytes/Str, got {n:?}"),
    }
}

fn vec_of(n: &NativeValue) -> &[NativeValue] {
    match n {
        NativeValue::Vec(v) | NativeValue::Tuple(v) => v,
        _ => panic!("expected NativeValue::Vec/Tuple, got {n:?}"),
    }
}

fn soroban_u256(env: &Env, n: &NativeValue) -> SU256 {
    let be = word_of(n).to_be_bytes::<32>();
    SU256::from_be_bytes(env, &Bytes::from_array(env, &be))
}

fn soroban_i256(env: &Env, n: &NativeValue) -> SI256 {
    let be = word_of(n).to_be_bytes::<32>();
    SI256::from_be_bytes(env, &Bytes::from_array(env, &be))
}
