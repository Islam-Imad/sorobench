// From solc semanticTests: array/array_push_with_arg.sol (copied verbatim below the header).
//
// What it exercises: pushing values onto a `uint[]` storage array with `.push(v)`,
// reading `.length` back, and indexing into it with `fetch(i)`. It proves the push
// grows the array (getLength goes 0 -> 1 -> 2), values round-trip (fetch(0) -> 42),
// and an out-of-bounds index traps — the `FAILURE, hex"4e487b71", 0x32` lines are
// Solidity's Panic(0x32) "array index out of bounds", which the runner matches as a
// trap (PASS(revert)); the panic data words are dropped, only the trap is checked.
//
// Result via `cargo run -- run`: 10 pass, 0 fail.
contract C {
    uint[] storageArray;
    function test(uint256 v) public {
        storageArray.push(v);
    }
    function getLength() public view returns (uint256) {
        return storageArray.length;
    }
    function fetch(uint256 a) public view returns (uint256) {
        return storageArray[a];
    }
}
// ----
// getLength() -> 0
// test(uint256): 42 ->
// getLength() -> 1
// fetch(uint256): 0 -> 42
// fetch(uint256): 1 -> FAILURE, hex"4e487b71", 0x32
// test(uint256): 23 ->
// getLength() -> 2
// fetch(uint256): 0 -> 42
// fetch(uint256): 1 -> 23
// fetch(uint256): 2 -> FAILURE, hex"4e487b71", 0x32
