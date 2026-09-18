//! AST for the // ---- expectation DSL, modelled on solc's SoltestTypes.h.
//!
//! The parser records structure and type classification only; it does not
//! compute the ABI bytes for each value. Each Parameter carries its detected
//! AbiType, parsed Alignment, and a decoded Literal; the raw byte encoding is
//! computed later by the decoder.

// Detected ABI type of a parameter literal (like solc's ABIType::Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiTypeKind {
    None,
    Failure,
    Boolean,
    UnsignedDec,
    SignedDec,
    Hex,
    HexString,
    String,
    UnsignedFixedPoint,
    SignedFixedPoint,
}

// How a value's bytes are aligned in its 32-byte word (like ABIType::Align).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    None,
}

// Type info attached to a parsed parameter (like solc's ABIType). size and
// fractional_digits are best-effort here: the exact values for numbers and
// fixed-point are computed later by the decoder. size is precise for String
// and HexString (the decoded byte length).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiType {
    pub kind: AbiTypeKind,
    pub align: Align,
    pub size: usize,
    pub fractional_digits: usize,
}

impl AbiType {
    pub fn none() -> Self {
        AbiType {
            kind: AbiTypeKind::None,
            align: Align::None,
            size: 0,
            fractional_digits: 0,
        }
    }
}

// The explicit left(...)/right(...) alignment wrapper (like Parameter::Alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    None,
    Boolean(bool),                              // true or false
    Decimal { negative: bool, digits: String }, // - a.b
    HexNumber(String),                          // 0x... hex number, including the 0x prefix.
    HexString(Vec<u8>),                         // Decoded bytes of a hex"..." string.
    Text(Vec<u8>),                              // Decoded bytes of a quoted string.
    Failure,
}

// A single parameter in an argument or expectation list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    // Reconstructed source text, e.g. -5, "any", hex"4200ef", left(0x20).
    // Lossy for binary string content (the exact bytes live in literal).
    pub raw_string: String,
    pub abi_type: AbiType,
    pub alignment: Alignment,
    // Whether a // newline preceded this parameter (isoltest's format.newline).
    pub newline: bool,
    pub literal: Literal,
}

// Denomination a FunctionValue was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueUnit {
    Wei,
    Ether,
}

// The optional ", N ether|wei" value modifier. Keeps the raw decimal digits
// and unit; the wei conversion (x 10^18 for ether) is not done here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionValue {
    pub digits: String,
    pub unit: ValueUnit,
}

// Classification of a call (like solc's FunctionCall::Kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Regular,
    Constructor,
    // Empty-name signature (): a low-level call with unstructured calldata.
    LowLevel,
    Library,
    // A framework builtin (storageEmpty, balance, account, isoltest_*).
    Builtin,
}

// Single-line vs multi-line display mode (like solc's FunctionCall::DisplayMode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionCallArgs {
    pub parameters: Vec<Parameter>,
    pub comment: String,
}

// Expected result of a call (like solc's FunctionCallExpectations).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallExpectations {
    pub result: Vec<Parameter>,
    // Whether a REVERT/EVM failure is expected. Defaults to true and is cleared
    // once a non-FAILURE result (or no ->) is seen.
    pub failure: bool,
    pub comment: String,
    // "gas <runType>: N" lines attached to the call, by run type.
    pub gas_used: std::collections::BTreeMap<String, String>,
}

impl Default for FunctionCallExpectations {
    fn default() -> Self {
        FunctionCallExpectations {
            result: Vec::new(),
            failure: true,
            comment: String::new(),
            gas_used: std::collections::BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub signature: String, // e.g. f(uint256,uint256);
    // None unless a ", N ether|wei" modifier was given.
    pub value: Option<FunctionValue>,
    pub arguments: FunctionCallArgs,
    pub expectations: FunctionCallExpectations,
    pub display_mode: DisplayMode,
    pub kind: Kind,
    // true when no -> was declared ("short-handed").
    pub omits_arrow: bool,
    // "~ ..." side-effect lines following the call.
    pub expected_side_effects: Vec<String>,
    // Library file (library: "file":Name); empty unless a library call.
    pub library_file: String,
}

impl FunctionCall {
    pub fn new() -> Self {
        FunctionCall {
            signature: String::new(),
            value: None,
            arguments: FunctionCallArgs::default(),
            expectations: FunctionCallExpectations::default(),
            display_mode: DisplayMode::SingleLine,
            kind: Kind::Regular,
            omits_arrow: true,
            expected_side_effects: Vec::new(),
            library_file: String::new(),
        }
    }
}

impl Default for FunctionCall {
    fn default() -> Self {
        Self::new()
    }
}
