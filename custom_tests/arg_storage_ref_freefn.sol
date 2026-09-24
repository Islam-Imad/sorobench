// Storage-ref PARAMETER on a free function: `fun(uint[] calldata, uint[] storage _y)`.
// The public `f` pushes onto storage `data`, then passes it by storage reference
// (alongside a calldata array) into the free function `fun`.
// From solc semanticTests/freeFunctions/storage_calldata_refs.sol.
//
// RESULT via `cargo run -- run`: GAP (compile-fail, portable)
//   solang: "Soroban external functions can return at most one value"
//
// SOLANG-GAP: this never reaches the storage-ref-free-function path we wanted to
// probe — it is blocked earlier by the multi-value-return limitation. `f` returns
// `(uint, uint)` and `fun` returns `(uint, uint[] calldata)`; solang rejects any
// n>=2 return on the Soroban target at compile time (no return packing). So the
// free-fn storage-ref behaviour stays untested until multi-value returns are
// supported (or the test is rewritten to a single return).
contract C {
    uint[] data;
    function f(uint x, uint[] calldata input) public returns (uint, uint) {
        data.push(x);
        (uint a, uint[] calldata b) = fun(input, data);
        return (a, b[1]);

    }
}

function fun(uint[] calldata _x, uint[] storage _y) view  returns (uint, uint[] calldata) {
	return (_y[0], _x);
}
// ----
// f(uint256,uint256[]): 7, 0x40, 3, 8, 9, 10 -> 7, 9
