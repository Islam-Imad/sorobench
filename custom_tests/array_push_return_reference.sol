// From solc semanticTests: array/array_push_return_reference.sol (copied verbatim below the header).
//
// What it exercises: the `arr.push()` form that returns a *reference* to the new
// slot, assigned to in place — `storageArray.push() = v;`. It grows the storage
// array, sets the pushed element, and reads it back via `fetch(i)` / `getLength()`,
// with out-of-bounds indexing trapping Panic(0x32).
//
// Result via `cargo run -- run`: GAP (compile-fail, portable)
//   solang: "expression is not assignable"
//
// SOLANG-GAP: solang can't compile `push()`-returning-a-storage-ref as an lvalue.
// The source is portable Solidity (no EVM-only feature), so this is a real solang
// gap, not a platform mismatch. `arr.push(v)` (the arg form) works — see
// array_push_with_arg.sol — only the no-arg `push() = v` reference form is rejected.
contract C {
    uint[] storageArray;
    function test(uint256 v) public {
        storageArray.push() = v;
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
