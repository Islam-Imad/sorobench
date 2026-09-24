# custom_tests

Curated `.sol` tests we want to focus on — small, hand-picked cases in the solc
semantic-test format (Solidity source + a `// ----` expectation block), separate
from the full pinned corpus.

Use it to pin down a specific behaviour, reproduce a suspected solang-on-Soroban
gap in isolation, or keep a regression close at hand while iterating.

## Run a single test

Point `run` at one `.sol` file to check just that file. Commands are run from the
repo root; `run` is part of the default `harness` feature, so no extra flags are
needed (it does require the LLVM16 toolchain to build):

```console
$ cargo run -- run custom_tests/demo.sol
  PASS         inc(uint64)
  PASS         set(uint64)
  PASS         get()
  PASS(revert) needBig(uint64)
  PASS         wrong()
  5 pass, 0 fail, 0 skipped/unsupported/nofaithful
```

Each `// ----` call gets one line: a verdict label, the call signature, and (for
non-passes) a short reason. The file above is `demo.sol` — it exercises a pure
return, state (`set`/`get`), an expected revert (`FAILURE`), and a value return.

To try your own case, drop a `.sol` file in this directory in the same format
(source, then a `// ----` block of `call(types): args -> expected` lines) and run
it by path.

## Run every test here

With no argument, `run` defaults to this directory and runs all `.sol` files,
printing a per-file section plus a combined total:

```console
$ cargo run -- run                    # all of custom_tests/
$ cargo run -- run custom_tests       # same thing, explicit
```

You can also point it at any other directory of `.sol` tests.

## Verdict labels

| Label          | Meaning                                                        |
|----------------|---------------------------------------------------------------|
| `PASS`         | actual value equals the expected `// ----` value              |
| `PASS(revert)` | expected `FAILURE` and the call trapped                       |
| `MISMATCH`     | the call returned a value, but not the expected one           |
| `TRAP`         | expected a value, but the call trapped                        |
| `NO-REVERT`    | expected `FAILURE`, but the call returned a value             |
| `NoFaithful`   | no faithful Soroban equivalent (e.g. a 20-byte `address`)     |
| `SKIP`         | not run (value call, builtin, library, or constructor line)   |
| `UNSUPPORTED`  | a type/feature the runner doesn't handle yet                  |

A whole file can also fail before any call runs — you'll see `GAP (compile-fail,
portable)` (solang couldn't compile portable source: a real gap), `FILTERED
(EVM-only)` (the source uses an EVM-only feature Soroban can't express),
`CRASHED`, `TIMED-OUT`, `UNSUPPORTED`, `FRONTEND-ERROR` (the `// ----` block
didn't parse), or `(no // ---- block)`.

## What a run does

Each file is parsed, compiled with solang → Soroban, deployed on a `SorobanEnv`
(a parameterized constructor is deployed from its `constructor(): args` line), and
each `// ----` call is invoked. Both the expected value and the actual `Val` are
decoded to the canonical `NativeValue` and compared there, not on raw `Val`s.

## Format

Same as the solc corpus: region 1 is the Solidity source (handed to solang),
region 3 is the `// ----` block of `call(types): args -> expected` lines. See the
grammar in the spec §2. `demo.sol` is a worked example covering pass / void /
state / revert / value; `struct_array_return.sol` and `nested_dynamic_array.sol`
probe composite encode/decode paths.
