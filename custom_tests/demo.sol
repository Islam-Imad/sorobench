contract C {
    uint64 stored;

    function inc(uint64 x) public pure returns (uint64) {
        return x + 1;
    }

    function set(uint64 x) public {
        stored = x;
    }

    function get() public view returns (uint64) {
        return stored;
    }

    function needBig(uint64 x) public pure returns (uint64) {
        require(x > 100);
        return x;
    }

    function wrong() public pure returns (uint64) {
        return 999;
    }
}
// ----
// inc(uint64): 5 -> 6
// set(uint64): 42 ->
// get() -> 42
// needBig(uint64): 5 -> FAILURE
// wrong() -> 999
