# sorobench

Run the [solc](https://github.com/ethereum/solidity) **semantic test suite**
against [solang](https://github.com/hyperledger-solang/solang)'s **Soroban**
target, using each test's saved EVM `// ----` values as the answer key.

solang's EVM target produces no bytecode, so the EVM side is never run here. A
test's `// ----` expected values are the saved EVM answer key, and
solang-on-Soroban is the thing being tested. Every run compares *actual (Soroban)
vs expected (EVM)*: sorobench compiles a `.sol` test with solang to Soroban, runs
it on a `SorobanEnv`, and checks each call's result against `// ----`.

The solc suite is the largest and most tested set of Solidity behavior that
exists (about 1500 tests). sorobench turns each one into a single result about
solang's Soroban backend, so "how much Solidity does solang-on-Soroban actually
get right?" becomes a number you can measure and track.

---

## How it works

Each test goes through five steps, giving one result per `// ----` call:

```
 parse  ->  compile  ->  invoke  ->  decode  ->  compare
(// ----   (solang     (Soroban   (both       (native
  DSL)      -> wasm+ns)  Env)       sides)      values)
```

1. **Parse** the test file: Solidity source above `// ----`, and the expectation
   lines below it (`name(types): args -> expected`). This is a direct port of
   solc's own `TestFileParser` / `TestCaseReader`.
2. **Compile** the source with `solang::compile(…, Target::Soroban)`. This
   returns the wasm and the resolved `Namespace` (`ns`). The `ns` holds every
   function's parameter and return **types**, which the decoder needs.
3. **Invoke** each call on a `SorobanEnv` (a thin `soroban_sdk::Env` wrapper).
   State stays between calls in one `Env`, so `set(…)` then `get()` works.
4. **Decode** both sides into a shared `NativeValue` (see below).
5. **Compare** the native values and print `PASS` / `MISMATCH` / `TRAP` / etc.

### The decoder: why alloy-dyn-abi

This is the core of the tool, and the reason for its most important dependency.

The `// ----` expected values are **not plain literals**. They are EVM ABI byte
dumps. `f() -> 0x20, 1, 13` is three 32-byte ABI words (an offset, a length, and
a value), not "three numbers". To know that those words mean *an array of one
struct holding 13*, you need the return **type**, which is exactly what solang's
`ns` provides. So decoding has to be driven by the type. Writing a correct EVM ABI
decoder by hand (offsets, nesting, sign-extension, `bytesN` cleanup, and so on)
would be a large and error-prone job.

Instead sorobench uses **[Alloy](https://github.com/alloy-rs)**, the standard Rust
toolkit for Ethereum-style chains, and specifically its **`alloy-dyn-abi`** crate.
`alloy-dyn-abi` is a *runtime* ABI encoder/decoder: it works with Solidity types
whose shape is only known while running (a type string like `((uint256)[])`),
which is exactly our case. The type comes from `ns` at run time, not from Rust
generics at compile time. Because the expected values are written in EVM ABI
format, we need a decoder that turns those bytes back into normal Rust values we
can read, and then maps those to Soroban `Val`s.

The decode has three steps:

```
 // ---- tokens  ->  ABI byte buffer  ->  DynSolValue  ->  NativeValue  <->  Val
  (ours: a port      (type-specific       (alloy-dyn-abi)   (ours: the        (Soroban
   of solc's         padding/align)                          shared form)      host side)
   BytesUtils.cpp)
```

- **tokens -> ABI buffer** (ours): a port of solc's `BytesUtils.cpp`. Numbers and
  bools are right-aligned and zero-padded to 32 bytes, strings are left-aligned,
  `hex"…"` stays raw, and so on.
- **buffer + type -> `DynSolValue`** (`alloy-dyn-abi`): `DynSolType::parse(t)
  .abi_decode_params(buf)`. All the ABI-walking work lives here.
- **`DynSolValue` <-> `NativeValue` <-> `Val`** (ours): the new adapter.

**Why one shared `NativeValue`, and why compare there.** A Soroban result is a
host `Val`. Two `Val`s that wrap host objects compare by *handle*, not by content,
so you cannot compare `Val`s directly. Both sides are decoded down to one shared
`NativeValue` and compared there instead. Integers collapse to a single 256-bit
number, so the compare is **by value and ignores width and sign**: a Soroban
`U64` returned for a Solidity `uint32` still equals the expected `6`.

Two checks keep a decoder bug apart from a real solang bug:

- **round-trip:** `from_val(to_val(x)) == x` (no solang involved).
- **oracle check:** `abi_decode(expected, T) == from_val(actual)`. This *is* the
  pass/fail test.

### Other notable decisions

- **`ns` is the type source, not the DSL.** The words `0x20, 1, 13` mean nothing
  on their own; the return type from `ns` tells us what they are. This is why the
  runner needs a full solang compile, not just the front-end.
- **Structs are driven by type (Finding B).** At the ABI boundary solang turns a
  struct into a host `Map` keyed by field-name `Symbol`, while alloy decodes it as
  an ordered `Tuple`. So `from_val` / `to_val` read and write struct fields by
  `Symbol` key in field order, not by position.
- **`NoFaithful` is labelled, not failed.** Some values have no true Soroban
  match: a 20-byte EVM `address` (Soroban addresses are 32-byte), a `uint256` that
  depends on exact 2^256 wraparound, or a `bytes32` used as a keccak identity.
  These are marked `NoFaithful` instead of being counted as bugs.
- **Crashes are isolated in a subprocess.** A solang internal error can be an
  uncatchable `SIGABRT` or a stack overflow, so `run-all` runs each test in its
  own child process. One bad test cannot kill the batch of 1500.

---

## Current results

From the last full corpus run (`report/summary.md`, solc **v0.8.22**, 1503 tests,
60s per test), against solang at
[`e6289eb`](https://github.com/Islam-Imad/solang/commit/e6289eb708d5cfbe07782dadb9c41c23ba6facfa):

**Headline:** of the **500** files that compiled and ran, **394 (78.8%)** had no
failed check; **106** showed a likely solang-on-Soroban bug (mismatch or trap).
Across those files, call by call: **1423 pass**, **276 fail**, 242 other (skipped
/ nofaithful / unsupported).

File-level buckets:

| bucket | files | % | meaning |
|---|---:|---:|---|
| `PASS_ALL` | 349 | 23.2% | every checked call passed |
| `PASS_SOME` | 45 | 3.0% | passed, some calls skipped |
| `HAS_FAIL` | 106 | 7.1% | a checked mismatch/trap: a real bug |
| `ONLY_OTHER` | 36 | 2.4% | nothing checkable ran |
| `COMPILE_FAIL` | 890 | 59.2% | solang could not compile it, or errored |
| `CRASH` | 45 | 3.0% | uncatchable abort (caught by isolation) |
| `NO_BLOCK` | 27 | 1.8% | no `// ----` expectations |
| `UNSUPPORTED` | 0 | 0.0% | a whole-file limit |
| `TIMEOUT` | 5 | 0.3% | went over the per-test timeout |

The large `COMPILE_FAIL` group comes mostly from a few repeated solang gaps: above
all *"Soroban external functions can return at most one value"* (no multi-return
support) and internal errors mid-compile. `summary.md` lists every failure, crash,
timeout, and compile-fail (with its reason) so gaps can be looked at directly.

> **Note.** These raw buckets are the v1 view: "did it compile, run, and match?"
> They do **not** yet separate *EVM-only tests that cannot have a Soroban meaning*
> (assembly, `selfdestruct`, `ecrecover`, and so on) from real solang gaps. That
> sorting step, putting each test into A (not applicable) / B (bridgeable, on the
> roadmap) / C (a real pass or gap) for an honest headline, is the next phase.

Regenerate the report any time with `cargo run -- run-all`.

---

## Build

The tool's job is to *run* tests, which means compiling them with solang to
Soroban, so the **default build includes the runner** and needs **LLVM 16** (the
same toolchain solang uses). `.cargo/config.toml` points `llvm-sys` at a local
LLVM 16 prefix, so no manual setup is needed:

```console
$ cargo build                 # builds everything, including `run`
$ cargo run -- run test.sol   # compile + execute a test
```

On another machine, edit the one path in `.cargo/config.toml` to your LLVM 16
prefix (the directory whose `bin/` holds `llvm-config`).

For fast, LLVM-free work on just the front-end (`list-tests` / `parse`), skip the
harness:

```console
$ cargo build --no-default-features
$ cargo run --no-default-features -- parse test.sol
```

---

## Usage

### `run`: execute tests (the main command)

Compile, deploy, invoke, decode both sides, compare, printing **one result per
`// ----` call**. Three forms:

```console
# 1. no argument -> runs every .sol in ./custom_tests/ (your focus set)
$ cargo run -- run

# 2. one file -> per-call results
$ cargo run -- run custom_tests/demo.sol

# 3. a directory -> runs every .sol under it, per-file plus a TOTAL line
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

A `MISMATCH` means the contract returned a value different from the `// ----`
answer key. On a **real corpus test** that is a real solang-on-Soroban bug; in a
hand-written test it just means your expected value does not match your code.

### `run-all`: the whole corpus (the headline report)

Runs **every** solc semantic test against solang-on-Soroban and writes a report.
Each test runs in **its own subprocess**: if solang crashes (an internal error,
an LLVM assert, or a stack overflow) the crash cannot be caught in-process, so
running each test on its own keeps one bad test from killing the batch of 1500. A
stuck compile is killed after the per-test timeout.

```console
$ cargo run -- run-all                    # the pinned solc v0.8.22 corpus
$ SOROBENCH_TIMEOUT=30 cargo run -- run-all path/to/dir   # any dir, 30s/test
```

Two files land in `report/`:

- **`results.jsonl`**: one `FileReport` JSON record per test (machine-readable;
  `path`, `report`, `bucket`, per-call `pass` / `fail` / `other`, per-call
  results).
- **`summary.md`**: the human report. A headline pass-rate, the file-level bucket
  table, a per-directory breakdown, and the lists you act on (failures, crashes,
  timeouts, compile-fails with their reasons).

File-level buckets: `PASS_ALL` (every checked call passed), `PASS_SOME` (passed,
some calls skipped), `HAS_FAIL` (a checked mismatch/trap: a real solang bug),
`ONLY_OTHER` (nothing checkable ran), `COMPILE_FAIL`, `UNSUPPORTED`, `NO_BLOCK`,
`FRONTEND_ERROR`, and, added by the driver when a child dies, `CRASH` / `TIMEOUT`.

### `run-one`: one test as a JSON record

`run-all`'s single unit, also usable on its own. Runs one file and prints exactly
one `FileReport` line to stdout (the format `run-all` reads back):

```console
$ cargo run -- run-one custom_tests/demo.sol
{"path":"custom_tests/demo.sol","report":"ran","bucket":"HAS_FAIL","pass":4,"fail":1,…}
```

### Your own tests: `custom_tests/`

`custom_tests/` is a tracked folder for small, hand-picked cases you want to focus
on: reproduce a suspected gap, or keep a regression close by. Drop a `.sol` file
in it and run `sorobench run` (no argument) to run the whole folder. It ships with
`demo.sol` (a worked example) and `struct_array_return.sol` (a struct-array
return).

### Writing a test

A test file has two parts: Solidity source, then a `// ----` expectation block.
This is exactly the solc semantic-test format:

```solidity
contract C {
    function inc(uint64 x) public pure returns (uint64) { return x + 1; }
}
// ----
// inc(uint64): 5 -> 6
```

- Everything **above** `// ----` is Solidity, handed as-is to solang.
- Each `// ----` line is `name(types): args -> expected`.
  - `args` and `-> expected` are optional (a setup call may leave out `->`).
  - `-> FAILURE` says the call should revert or trap.
  - Values are matched by **meaning**, not width: a Soroban `U64` and an EVM
    `uint256` both compare equal to `6`.

Supported today: scalars (`uintN` / `intN` / `bool`), `bytes` / `bytesN` /
`string`, `enum`, `address` (marked NoFaithful), and **arrays and structs**
(including nested ones). Multiple calls run in order and **share state** (one
`SorobanEnv`), so `set(…)` then `get()` works.

### Reading the results

Per-call:

| Label          | Meaning |
|----------------|---------|
| `PASS`         | actual == expected (both decoded to the shared form, compared by value) |
| `PASS(revert)` | expected `FAILURE` and the call reverted or trapped |
| `MISMATCH`     | returned a value that is not the expected one (on corpus tests: a real solang bug) |
| `TRAP`         | expected a value, but the call reverted or trapped |
| `NO-REVERT`    | expected `FAILURE`, but the call returned |
| `NoFaithful`   | result has no true Soroban match (e.g. an `address`) |
| `SKIP`         | not run: framework builtin / library / low-level / `,N ether` call |
| `UNSUPPORTED`  | a param or return type the runner does not map yet |

Whole-file (the test cannot run at all):

| Line | Meaning |
|------|---------|
| `COMPILE-FAILED` | solang could not compile the source, or errored: a real bug |
| `UNSUPPORTED` | a whole-file limit (e.g. the constructor needs args) |
| `(no // ---- block)` | the file has no expectations to run |

### `list-tests` and `parse` (corpus inspection)

Look at the pinned solc corpus without compiling any contracts (add
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
(`…/testdata/solidity/test/libsolidity/semanticTests`); override it with
`$SOROBENCH_CORPUS` or a positional argument.

---

## For contributors

### Feature flags

`harness` is a **default** feature (the runner is the whole point). Turn it off
with `--no-default-features` for a fast, LLVM-free build of just the front-end and
decoder:

| Feature | Pulls in | Enables | Notes |
|---------|----------|---------|-------|
| `harness` *(default)* | `decoder` + `solang`, `solang-parser` | `run` (compile + execute) | **needs LLVM 16** (set up by `.cargo/config.toml`) |
| `decoder` | `alloy-dyn-abi`, `alloy-primitives`, `soroban-sdk` | the EVM-ABI <-> `NativeValue` <-> `Val` decoder | pure Rust, no LLVM |
| *(none)* | std only | `list-tests`, `parse` | fast, no env setup |

```console
$ cargo test                                            # everything incl. the runner (LLVM 16)
$ cargo test --no-default-features                      # front-end only, no LLVM
$ cargo test --no-default-features --features decoder   # + decoder, still no LLVM
```

### Layout

- `src/corpus.rs`: list the solc corpus.
- `src/testfile.rs`, `src/expectation/`: split a `.sol` test and parse its
  `// ----` lines.
- `src/decoder/`: tokens -> bytes -> alloy -> `NativeValue` <-> Soroban `Val`
  (both sides decode to `NativeValue`; the compare happens there, not on raw
  `Val`s). `bytes_utils.rs` (tokens -> ABI buffer), `abi.rs` (alloy decode),
  `native.rs` (the shared form), `val.rs` (the `Val` adapter), `nofaithful.rs`,
  `types.rs`.
- `src/harness/`: `compile` (solang -> wasm + `ns`), `env` (`SorobanEnv`),
  `typemap` (`ns` type -> decoder type + ABI string), `runner` (the end-to-end
  `run`), `isolate` (subprocess crash isolation).

### Dependencies (why each)

- **`solang` + `solang-parser`**: the compiler used as a library.
  `compile(…, Target::Soroban)` is the harness, and `ns` is the decoder's type
  source. Both pinned to the **same** solang rev (types must match: a `Level` or
  `pt` from two different revs will not line up).
- **`alloy-dyn-abi` + `alloy-primitives`**: the runtime EVM ABI codec that turns
  the `// ----` byte dumps into typed values (see [How it works](#how-it-works)).
- **`soroban-sdk`**: the `Val` side of the decoder and the `Env` the runner
  invokes on.

### Status

- The front-end parser, the decoder, and the runner all work end to end: you can
  `run` a single file or a directory, or `run-all` the whole corpus under
  subprocess isolation and get the report above.
- Next up (v2): the A/B/C sorting step. Separate EVM-only tests and
  bridgeable-but-unbuilt features from real solang gaps, for an honest headline
  pass-rate.
