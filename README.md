# sorobench

Replay the [solc](https://github.com/ethereum/solidity) **semantic test suite**
against [solang](https://github.com/hyperledger-solang/solang)'s **Soroban**
target, using each test's frozen EVM `// ----` values as an oracle.

solang's EVM target emits no bytecode, so the EVM side is never executed — a
test's `// ----` expected values **are** the frozen EVM oracle, and
solang-on-Soroban is the system under test. Every run is *actual (Soroban) vs
frozen-expected (EVM)*: sorobench compiles a `.sol` test with solang → Soroban,
runs it on a `SorobanEnv`, and checks each call's result against `// ----`.

---

## Build

The tool's job is to *run* tests — which compiles them with solang → Soroban — so
the **default build includes the runner** and needs **LLVM 16** (the same toolchain
solang uses). `.cargo/config.toml` points `llvm-sys` at a local LLVM 16 prefix, so
no manual environment setup is required:

```console
$ cargo build                 # builds everything, including `run`
$ cargo run -- run test.sol   # compile + execute a test
```

On a different machine, edit the one path in `.cargo/config.toml` to your LLVM 16
prefix (the directory whose `bin/` holds `llvm-config`).

For fast, LLVM-free iteration on just the front-end (`list-tests` / `parse`), skip
the harness:

```console
$ cargo build --no-default-features
$ cargo run --no-default-features -- parse test.sol
```

---

## Usage

### `run` — execute tests (the main command)

Compile → deploy → invoke → decode both sides → compare, printing **one verdict
per `// ----` call**. Three forms:

```console
# 1. no argument → runs every .sol in ./custom_tests/ (your focus set)
$ cargo run -- run

# 2. one file → verbose, per-call verdicts
$ cargo run -- run custom_tests/demo.sol

# 3. a directory → runs every .sol under it, per-file + a TOTAL line
$ cargo run -- run path/to/dir
```

Example output:

```text
  PASS         inc(uint64)
  PASS         set(uint64)
  PASS         get()
  PASS(revert) needBig(uint64)
  MISMATCH     wrong()  — expected [Int(123)], got Int(999)
  4 pass, 1 fail, 0 skipped/unsupported/nofaithful
```

A `MISMATCH` means the contract returned a value that differs from the `// ----`
oracle. On a **real corpus test** that is a genuine solang-on-Soroban bug; in a
hand-written test it just means your expected value doesn't match your code.

### `run-all` — the whole corpus (the headline report)

Runs **every** solc semantic test against solang-on-Soroban and writes a report.
Each test runs in **its own subprocess**: if solang crashes (an internal error,
LLVM assert, or stack overflow) the crash can't be caught in-process, so running
each test separately keeps one bad test from aborting the batch of 1500. A hung
compile is killed after a per-test timeout.

```console
$ cargo run -- run-all                    # the pinned solc v0.8.22 corpus
$ SOROBENCH_TIMEOUT=30 cargo run -- run-all path/to/dir   # any dir, 30s/test
```

Two artifacts land in `report/`:

- **`results.jsonl`** — one `FileReport` JSON record per test (machine-readable;
  `path`, `report`, `bucket`, per-call `pass`/`fail`/`other`, per-call verdicts).
- **`summary.md`** — the human report: a headline pass-rate, file-level bucket
  table, a per-directory breakdown, and the actionable lists (failures, crashes,
  timeouts, compile-fails with their reasons).

File-level buckets: `PASS_ALL` (every checked call passed), `PASS_SOME` (passed,
some calls skipped), `HAS_FAIL` (a checked mismatch/trap — a real solang bug),
`ONLY_OTHER` (nothing checkable ran), `COMPILE_FAIL`, `UNSUPPORTED`, `NO_BLOCK`,
`FRONTEND_ERROR`, and — synthesized by the driver from a dead child — `CRASH` /
`TIMEOUT`.

### `run-one` — one test as a JSON record

`run-all`'s isolated unit, also usable directly. Runs one file and prints exactly
one `FileReport` line to stdout (the wire format `run-all` reads back):

```console
$ cargo run -- run-one custom_tests/demo.sol
{"path":"custom_tests/demo.sol","report":"ran","bucket":"HAS_FAIL","pass":4,"fail":1,…}
```

### Your own tests: `custom_tests/`

`custom_tests/` is a tracked directory for small, hand-picked cases you want to
focus on — reproduce a suspected gap, or keep a regression close. Drop a `.sol`
file in it and run `sorobench run` (no argument) to execute the whole folder. It
ships with `demo.sol` (a worked example) and `struct_array_return.sol` (a
struct-array return).

### Writing a test

A test file has two parts — Solidity source, then a `// ----` expectation block —
exactly the solc semantic-test format:

```solidity
contract C {
    function inc(uint64 x) public pure returns (uint64) { return x + 1; }
}
// ----
// inc(uint64): 5 -> 6
```

- Everything **above** `// ----` is Solidity — handed verbatim to solang.
- Each `// ----` line is `name(types): args -> expected`.
  - `args` and `-> expected` are optional (a setup call may omit `->`).
  - `-> FAILURE` asserts the call reverts/traps.
  - Values are matched by **meaning**, not width: a Soroban `U64` and an EVM
    `uint256` both compare equal to `6`.

Supported today: scalars (`uintN`/`intN`/`bool`), `bytes`/`bytesN`/`string`,
`enum`, `address` (flagged NoFaithful), and **arrays + structs** (recursively).
Multiple calls run in order and **share state** (one `SorobanEnv`), so
`set(…)` then `get()` works.

### Understanding the verdicts

Per-call:

| Label          | Meaning |
|----------------|---------|
| `PASS`         | actual == expected (decoded to a common form, compared value-wise) |
| `PASS(revert)` | expected `FAILURE` and the call reverted/trapped |
| `MISMATCH`     | returned a value ≠ expected (on corpus tests: a real solang bug) |
| `TRAP`         | expected a value, but the call reverted/trapped |
| `NO-REVERT`    | expected `FAILURE`, but the call returned |
| `NoFaithful`   | result has no faithful Soroban equivalent (e.g. an `address`) |
| `SKIP`         | not run — framework builtin / library / low-level / `,N ether` call |
| `UNSUPPORTED`  | a param/return type the runner doesn't map yet |

Whole-file (the test can't run at all):

| Line | Meaning |
|------|---------|
| `COMPILE-FAILED` | solang couldn't compile the source, or ICE'd — a real bug |
| `UNSUPPORTED` | a whole-file limit (e.g. the constructor needs args) |
| `(no // ---- block)` | the file has no expectations to run |

### `list-tests` and `parse` (corpus inspection)

Inspect the pinned solc corpus without compiling any contracts (add
`--no-default-features` to skip the LLVM build entirely):

```console
$ cargo run -- list-tests | wc -l            # every corpus .sol path
1503

$ cargo run --release -- parse               # parse every // ---- block, coverage report
  files:         1503
  parsed OK:     1476  (5232 calls)
  split errors:  0
  parse errors:  0

$ cargo run --release -- parse path/to/test.sol   # one file: print its parsed calls
```

The corpus root defaults to the pinned solc **v0.8.22** submodule inside solang
(`…/testdata/solidity/test/libsolidity/semanticTests`); override with
`$SOROBENCH_CORPUS` or a positional argument.

---

## For contributors

### Feature flags

`harness` is a **default** feature (the runner is the point). Turn it off with
`--no-default-features` for a fast, LLVM-free build of just the front-end + decoder:

| Feature | Pulls in | Enables | Notes |
|---------|----------|---------|-------|
| `harness` *(default)* | `decoder` + `solang`, `solang-parser` | `run` (compile + execute) | **needs LLVM 16** (provided by `.cargo/config.toml`) |
| `decoder` | `alloy-dyn-abi`, `alloy-primitives`, `soroban-sdk` | the EVM-ABI ↔ `NativeValue` ↔ `Val` decoder | pure Rust, no LLVM |
| *(none)* | std only | `list-tests`, `parse` | fast, env-free |

```console
$ cargo test                                            # everything incl. the runner (LLVM 16)
$ cargo test --no-default-features                      # front-end only, no LLVM
$ cargo test --no-default-features --features decoder   # + decoder, still no LLVM
```

### Layout

- `src/corpus.rs` — enumerate the solc corpus.
- `src/testfile.rs`, `src/expectation/` — split a `.sol` test and parse its `// ----` DSL.
- `src/decoder/` — token→bytes → alloy → `NativeValue` ↔ Soroban `Val` (both sides
  decode to `NativeValue`; comparison happens there, not on raw `Val`s).
- `src/harness/` — `compile` (solang → wasm + `ns`), `env` (`SorobanEnv`), `typemap`
  (`ns` type → decoder type + ABI string), `runner` (the end-to-end `run`), `isolate`
  (subprocess crash-isolation).

### Status

- The front-end parser, the decoder, and the runner all work: you can `run` a
  single file or a directory and get a verdict per `// ----` call.
- Next up: run the whole corpus end-to-end under subprocess isolation, then sort
  each result into pass / real bug / unsupported for a headline number.
