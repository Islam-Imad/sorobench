// 2 pass. Inheritance + override: B overrides A.f(), and the runner deploys B
// (the last contract in the file, per solc), so f()->2 and g()->f()->2.
contract A {
    function f() external virtual returns (uint256) {
        return 1;
    }
}


contract B is A {
    function f() public override returns (uint256) {
        return 2;
    }

    function g() public returns (uint256) {
        return f();
    }
}
// ----
// f() -> 2
// g() -> 2
