// solc semanticTests/array/pop/byte_array_pop_empty_exception.sol (verbatim).
// Pops an empty `bytes` storage array -> Panic(0x31) underflow -> traps.
// Result: 1 pass (PASS(revert)).
contract c {
    uint256 a;
    uint256 b;
    uint256 c;
    bytes data;

    function test() public returns (bool) {
        data.pop();
        return true;
    }
}
// ----
// test() -> FAILURE, hex"4e487b71", 0x31
