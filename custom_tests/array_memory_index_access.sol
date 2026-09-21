// From solc semanticTests: array/array_memory_index_access.sol (copied verbatim below the header).
//
// What it exercises: dynamic MEMORY arrays. `new uint[](len)` allocates, a for-loop
// fills each slot with `i + 1` (a summing/accumulator-style write pattern), and a
// second loop `require`s each value round-tripped — so it stresses per-element read
// and write in a loop, plus `require`. `index(len)` returns whether `array.length`
// matched. `accessIndex` returns one element, and the trailing lines show that an
// out-of-bounds index (11, 10, or -1 cast to a huge uint) traps with Panic(0x32),
// matched as PASS(revert). Note: the `// gas ...` lines are EVM-only and the runner
// strips them.
//
// Result via `cargo run -- run`: 9 pass, 0 fail.
contract C {
	function index(uint256 len) public returns (bool)
	{
		uint[] memory array = new uint[](len);

		for (uint256 i = 0; i < len; i++)
			array[i] = i + 1;

		for (uint256 i = 0; i < len; i++)
			require(array[i] == i + 1, "Unexpected value in array!");

		return array.length == len;
	}
	function accessIndex(uint256 len, int256 idx) public returns (uint256)
	{
		uint[] memory array = new uint[](len);

		for (uint256 i = 0; i < len; i++)
			array[i] = i + 1;

		return array[uint256(idx)];
	}
}
// ----
// index(uint256): 0 -> true
// index(uint256): 10 -> true
// index(uint256): 20 -> true
// index(uint256): 0xFF -> true
// gas irOptimized: 108291
// gas legacy: 181523
// gas legacyOptimized: 117443
// accessIndex(uint256,int256): 10, 1 -> 2
// accessIndex(uint256,int256): 10, 0 -> 1
// accessIndex(uint256,int256): 10, 11 -> FAILURE, hex"4e487b71", 0x32
// accessIndex(uint256,int256): 10, 10 -> FAILURE, hex"4e487b71", 0x32
// accessIndex(uint256,int256): 10, -1 -> FAILURE, hex"4e487b71", 0x32
