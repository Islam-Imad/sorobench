// From solc semanticTests: array/array_storage_length_access.sol (copied verbatim below the header).
//
// What it exercises: repeatedly `.push()`-ing empty elements onto a `uint[]` storage
// array in a while-loop until it reaches `len`, then returning `.length`.
//
// WARNING — TOO SLOW TO RUN: the later calls push huge counts (0xFFF = 4095 and
// 0xFFFFF = 1,048,575 elements) one host storage-write at a time, so `cargo run --
// run` on this file does NOT finish within the per-test timeout (TIMED-OUT:
// exceeded 10s). The small cases (0..0xFF) pass, but the big-push lines make the
// file impractical to run as-is — kept as a documented perf/scale case, not a green test.
// The final `0xFFFFF -> FAILURE # Out-of-gas #` line is also EVM-specific: Soroban
// runs under an unlimited test budget, so it would not out-of-gas the same way.
contract C {
    uint[] storageArray;
    function set_get_length(uint256 len) public returns (uint256) {
        while(storageArray.length < len)
            storageArray.push();
        return storageArray.length;
    }
}
// ----
// set_get_length(uint256): 0 -> 0
// set_get_length(uint256): 1 -> 1
// set_get_length(uint256): 10 -> 10
// set_get_length(uint256): 20 -> 20
// set_get_length(uint256): 0xFF -> 0xFF
// gas irOptimized: 96690
// gas legacy: 128571
// gas legacyOptimized: 110143
// set_get_length(uint256): 0xFFF -> 0xFFF
// gas irOptimized: 1209119
// gas legacy: 1689548
// gas legacyOptimized: 1393535
// set_get_length(uint256): 0xFFFFF -> FAILURE # Out-of-gas #
