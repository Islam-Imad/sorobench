use alloy_primitives::U256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeValue {
    Bool(bool),
    Int(U256),
    Bytes(Vec<u8>),
    Str(Vec<u8>),
    Address(Vec<u8>),
    Vec(Vec<NativeValue>),
    Tuple(Vec<NativeValue>),
}

impl NativeValue {
    pub fn uint_u64(v: u64) -> NativeValue {
        NativeValue::Int(U256::from_limbs([v, 0, 0, 0]))
    }

    pub fn uint_u128(v: u128) -> NativeValue {
        NativeValue::Int(u128_word(v))
    }

    pub fn int_i128(v: i128) -> NativeValue {
        let mag = u128_word(v.unsigned_abs());
        let word = if v < 0 {
            U256::ZERO.wrapping_sub(mag)
        } else {
            mag
        };
        NativeValue::Int(word)
    }

    pub fn int_be(word: [u8; 32]) -> NativeValue {
        NativeValue::Int(U256::from_be_slice(&word))
    }
}

fn u128_word(v: u128) -> U256 {
    U256::from_limbs([v as u64, (v >> 64) as u64, 0, 0])
}
