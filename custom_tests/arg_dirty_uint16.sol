// 2 PASS, 2 FAIL. Not a solang bug: a cost of the Soroban target. solang widens
// uint16 to the host U32, so wraparound/overflow happen at 32 bits, not 16.
//   65534+0        -> PASS       (fits in 16 bits)
//   65536+0        -> MISMATCH   (EVM truncates to 0; Soroban keeps 65536)
//   65535+0        -> PASS       (fits in 16 bits)
//   65535+1        -> NO-REVERT  (EVM overflow-panics; Soroban has room in U32)
pragma abicoder v1;
contract C {
    // Input is still not checked - this needs ABIEncoderV2!
    function f(uint16 a, uint16 b) public returns (uint16) {
        return a + b;
    }
}
// ====
// ABIEncoderV1Only: true
// compileViaYul: false
// ----
// f(uint16,uint16): 65534, 0 -> 0xfffe
// f(uint16,uint16): 65536, 0 -> 0x00
// f(uint16,uint16): 65535, 0 -> 0xffff
// f(uint16,uint16): 65535, 1 -> FAILURE, hex"4e487b71", 0x11
