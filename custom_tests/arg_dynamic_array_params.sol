// from solc semanticTests: array/slices/array_calldata_assignment.sol
// 3 params (two dynamic uint256[] + a scalar) hand-encoded as 8 ABI arg words:
// the arg-word count is NOT the arity. Regression for name-based resolution +
// decode-based parameter matching.
contract C {
    function f(uint256[] calldata x, uint256[] calldata y, uint256 i) external returns (uint256) {
        x = y;
        return x[i];
    }
}
// ----
// f(uint256[],uint256[],uint256): 0x60, 0xA0, 1, 1, 0, 2, 1, 2 -> 2
