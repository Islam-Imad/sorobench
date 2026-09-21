// From solc semanticTests: types/mapping_simple.sol (copied verbatim below the header).
//
// What it exercises: a `mapping(uint8 => uint8)` in contract storage — the map is
// written with set(k,v) and read back with get(k). It proves that on Soroban the
// mapping is modelled as a host MapObject and that reads of never-written keys
// return the zero default (get before any set -> 0). uint8 keys/values avoid the
// 20-byte `address` NoFaithful issue, so every call decodes cleanly.
//
// Result via `cargo run -- run`: 15 pass, 0 fail.
contract test {
    mapping(uint8 => uint8) table;
    function get(uint8 k) public returns (uint8 v) {
        return table[k];
    }
    function set(uint8 k, uint8 v) public {
        table[k] = v;
    }
}
// ----
// get(uint8): 0 -> 0
// get(uint8): 0x01 -> 0
// get(uint8): 0xa7 -> 0
// set(uint8,uint8): 0x01, 0xa1 ->
// get(uint8): 0 -> 0
// get(uint8): 0x01 -> 0xa1
// get(uint8): 0xa7 -> 0
// set(uint8,uint8): 0x00, 0xef ->
// get(uint8): 0 -> 0xef
// get(uint8): 0x01 -> 0xa1
// get(uint8): 0xa7 -> 0
// set(uint8,uint8): 0x01, 0x05 ->
// get(uint8): 0 -> 0xef
// get(uint8): 0x01 -> 0x05
// get(uint8): 0xa7 -> 0
