#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SorobanType {
    Bool,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    U256,
    I256,
    Bytes,
    Str,
    Address,
    Vec(Box<SorobanType>),
    Struct(std::vec::Vec<(std::string::String, SorobanType)>),
}
