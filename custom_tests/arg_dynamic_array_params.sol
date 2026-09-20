// PASS. 3 params (two uint256[] + a uint256) passed as 8 ABI arg words.
// Shows the runner resolves f by name, not by arg-word count.
contract C {
    function f(uint256[] calldata x, uint256[] calldata y, uint256 i) external returns (uint256) {
        x = y;
        return x[i];
    }
}
// ----
// f(uint256[],uint256[],uint256): 0x60, 0xA0, 1, 1, 0, 2, 1, 2 -> 2
