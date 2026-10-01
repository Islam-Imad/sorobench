# sorobench — explained results

1503 tests from the solc semantic test suite were compiled with Solang for Soroban and run. **386 of 1235 (31.3%) pass.** 241 tests are excluded because they rely on EVM-only features, and 27 have nothing to check.

Every failure below is matched against sorobench's explanation dictionary, which is grounded in Solang's own Soroban documentation. A failure nothing explains is listed under **review**; it is not assumed to be a bug.

## What the failures are

| category | files | meaning |
|---|---:|---|
| **bug** | 213 | Violates Solang's documented behaviour: compiler crashes and wrong results |
| **review** | 78 | Not explained yet: needs a human to check it against the docs |
| **soroban-gap** | 5 | Missing on Soroban, not documented, works on Solang's other targets |
| **solang-gap** | 174 | Rejected by Solang on every target: a general Solang limitation |
| **documented-unsupported** | 243 | Solang's docs list the feature as not supported on Soroban |
| **documented-difference** | 60 | Solang's docs describe this behaviour as intended on Soroban |
| **evm-only** | 262 | Relies on an EVM concept that Soroban does not have |
| **tool** | 18 | A sorobench or test-environment limitation, not Solang |

A file with more than one kind of failure is counted in each of its categories.

## Issues

| # | issue | category | files |
|---:|---|---|---:|
| 1 | [Unsupported type crashes the Soroban encoder/decoder](#panic-unsupported-abi-type) | bug | 87 |
| 2 | [LLVM value-kind confusion (integer where a pointer is expected)](#llvm-value-kind) | bug | 25 |
| 3 | [LLVM assertion: call with the wrong argument types](#llvm-assert-call) | bug | 22 |
| 4 | [Function modifiers crash Soroban dispatch](#modifier-dispatch) | bug | 19 |
| 5 | [Unsupported builtin crashes Soroban codegen](#panic-unsupported-builtin) | bug | 17 |
| 6 | [`try`/`catch` runs Polkadot-only code](#try-catch-polkadot-path) | bug | 10 |
| 7 | [Other compiler crashes](#crash-other) | bug | 10 |
| 8 | [Stack overflow on recursive struct types](#stack-overflow) | bug | 6 |
| 9 | [Other LLVM assertion failures](#llvm-assert-other) | bug | 5 |
| 10 | [Crash in constant folding](#constant-folding) | bug | 5 |
| 11 | [Shifting by the full bit width gives the wrong result](#shift-by-width) | bug | 4 |
| 12 | [Overloaded function names exceed Soroban's symbol length](#symbol-too-long) | bug | 3 |
| 13 | [Timed out (over 10 s)](#timeout) | review | 22 |
| 14 | [Probably caused by integer rounding (needs confirmation)](#int-width-probable) | review | 18 |
| 15 | [Hashes differ (input built with `abi.encodePacked`)](#hash-of-encode-packed) | review | 9 |
| 16 | [Out-of-range enum values are accepted](#enum-range-not-checked) | review | 8 |
| 17 | [Contract fails while being deployed](#deployment-failed) | review | 3 |
| 18 | [Contract exceeds the Soroban budget](#budget-exceeded) | review | 1 |
| 19 | [Rejected on Soroban only (not yet explained)](#soroban-only-rejection) | soroban-gap | 5 |
| 20 | [Rejected by Solang on every target](#solang-language-gap) | solang-gap | 174 |
| 21 | [Multiple return values from public functions](#multi-return) | documented-unsupported | 201 |
| 22 | [Creating contracts with `new Contract()`](#new-contract) | documented-unsupported | 17 |
| 23 | [Native value transfer (`{value: …}`, `.balance`, `.transfer`)](#value-transfer-uint512) | documented-unsupported | 12 |
| 24 | [Public getters for struct state variables](#struct-public-getter) | documented-unsupported | 9 |
| 25 | [Structs as event parameters](#struct-in-event) | documented-unsupported | 4 |
| 26 | [Conversions between small integers and `bytesN`](#int-width-bytes-conversion) | documented-difference | 25 |
| 27 | [Small integer types are computed at a wider width](#int-width-overflow) | documented-difference | 23 |
| 28 | [`abi.encode` returns Soroban handles, not ABI bytes](#abi-encode-handles) | documented-difference | 7 |
| 29 | [`abi.decode` of bytes from outside the contract](#abi-decode-external-bytes) | documented-difference | 5 |
| 30 | [EVM-only test that also crashes the compiler](#evm-only-also-crashed) | evm-only | 155 |
| 31 | [EVM-only feature](#evm-only) | evm-only | 86 |
| 32 | [Malformed or raw EVM calldata](#evm-calldata-shape) | evm-only | 9 |
| 33 | [Function selectors (`msg.sig`, `interfaceId`)](#evm-selector-concepts) | evm-only | 4 |
| 34 | [Ethereum address literals](#eth-address-literal) | evm-only | 3 |
| 35 | [Fallback and receive functions](#evm-fallback-receive) | evm-only | 3 |
| 36 | [Reading contract bytecode (`address.code`)](#address-code) | evm-only | 2 |
| 37 | [Tests that import files from outside the test](#external-source-files) | tool | 15 |
| 38 | [Block number / timestamp start at 0 in sorobench](#test-env-block) | tool | 2 |
| 39 | [Not runnable by sorobench](#tool-unsupported) | tool | 1 |

<a id="panic-unsupported-abi-type"></a>
### Unsupported type crashes the Soroban encoder/decoder

**Category:** bug · **Files:** 87 · **Rule:** `panic-unsupported-abi-type`

**What it means.** The test passes a type across the contract boundary that Solang cannot encode for Soroban (function types, user-defined value types, contract types, …). Solang knows it is unsupported, but raises a panic instead of a compile error. Solang's docs state that unsupported constructs should be rejected with a diagnostic, not crash.

**Suggested fix.** Turn the panic in `codegen/targets/soroban/encoding.rs` into a sema diagnostic naming the type and location.

**Docs.** docs/targets/soroban_language_compatibility.rst, intro (reject rather than miscompile); support matrix, 'Enums and other complex user-defined types'.

<details><summary>Files</summary>

- `abiEncoderV1/abi_encode.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV1/abi_encode_empty_string.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV1/abi_encode_rational.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV2/abi_encode_rational_v2.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV2/abi_encode_v2.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV2/abi_encode_v2_in_function_inherited_in_v1_contract.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `abiEncoderV2/calldata_array_static_index_access.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint256[3]' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiEncoderV2/struct/validation_function_type_inside_struct.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `abiencodedecode/abi_encode_call_special_args.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiencodedecode/abi_encode_with_selector.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `abiencodedecode/abi_encode_with_selectorv2.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `array/copying/calldata_1d_array_into_2d_memory_array_element.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `array/copying/copying_bytes_multiassign.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract receiver' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `builtinFunctions/keccak256_packed_complex_types.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external returns (bytes32,bytes32,bytes32)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/ta…
- `constructor/constructor_arguments_internal.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract Helper' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `constructor/constructor_function_argument.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external returns (uint256)' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `constructor/constructor_function_complex.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `constructor/store_function_in_constructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint256) internal returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sor…
- `constructor/store_function_in_constructor_packed.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint32) internal returns (uint32)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sorob…
- `constructor/store_internal_unused_function_in_constructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `constructor/store_internal_unused_library_function_in_constructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `conversions/function_type_array_to_storage.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `deployedCodeExclusion/library_function_deployed.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal pure returns (bytes)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban…
- `deployedCodeExclusion/static_base_function_deployed.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal pure returns (bytes)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban…
- `events/event_indexed_function.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `events/event_indexed_function2.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `fallback/call_forward_bytes.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract receiver' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionCall/calling_nonexisting_contract_throws.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract D' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `functionCall/calling_uninitialized_function.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionCall/creation_function_call_with_args.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionCall/creation_function_call_with_salt.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionCall/external_call_to_nonexisting.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract I' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `functionCall/external_call_to_nonexisting_debugstrings.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract I' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `functionTypes/call_to_zero_initialized_function_type_ir.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint256) external returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sor…
- `functionTypes/call_to_zero_initialized_function_type_legacy.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint256) external returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sor…
- `functionTypes/external_functions_with_calldata_args_assigned_to_function_pointers_with_memory_type.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionTypes/function_delete_storage.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `functionTypes/mapping_of_functions.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionTypes/pass_function_types_externally.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint256) external returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sor…
- `functionTypes/selector_expression_side_effect.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `functionTypes/uninitialized_internal_storage_function_call.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `immutable/internal_function_pointer.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal view returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/sorob…
- `isoltestTesting/balance_other_contract.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract Other' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `libraries/external_call_with_function_pointer_parameter.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `libraries/internal_library_function_attached_to_external_function_type.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `libraries/library_references_preserve.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `operators/userDefined/attaching_and_defining_operator_with_same_function.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Int' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/checked_operators.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype U8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/multiple_operator_definitions_different_types_different_functions_separate_directives.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype SmallInt' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/multiple_operator_definitions_same_type_same_function_same_directive.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Int' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/operator_definition_shadowing_builtin_keccak256.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Int' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/operator_evaluation_order.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Bool' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `operators/userDefined/recursive_operator.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Uint' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `operators/userDefined/unchecked_operators.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype U8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `revertStrings/bubble.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `specialFunctions/abi_encode_with_signature_from_string.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `storage/packed_functions.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal returns (uint256)' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/en…
- `types/external_function_to_address.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `types/mapping/user_defined_types_mapping_storage.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype B' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `types/mapping_contract_key.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `types/mapping_contract_key_getter.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `types/mapping_contract_key_library.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract A' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `uninitializedFunctionPointer/invalidInConstructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `uninitializedFunctionPointer/invalidStoredInConstructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `uninitializedFunctionPointer/store2.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `uninitializedFunctionPointer/storeInConstructor.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/abicodec.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype C.MyInt' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `userDefinedValueType/constant.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype T' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/conversion.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyUInt8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/conversion_abicoderv1.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyUInt8' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/erc20.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype UFixed18' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `userDefinedValueType/fixedpoint.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype UFixed256x18' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14…
- `userDefinedValueType/mapping_key.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyInt' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/multisource.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyInt' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/multisource_module.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyInt' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `userDefinedValueType/ownable.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype Ownable.Owner' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:1…
- `userDefinedValueType/wrap_unwrap_via_contract_name.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype C.T' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `userDefinedValueType/zero_cost_abstraction_comparison_userdefined.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype MyInt' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `using/using_global_for_global.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype global' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `using/using_global_invisible.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'usertype T' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `various/destructuring_assignment.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'uint256 storage' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `various/external_types_in_calls.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C1' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `various/write_storage_external.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'contract C' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `viaYul/conversion/function_cast.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function(uint256) external returns (uint256)' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/sor…
- `viaYul/function_address.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)
- `viaYul/function_pointers.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function_selector' is not supported by the Soroban encoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:658:14)
- `viaYul/function_selector.sol` — compiler crashed: solang panicked mid-compile (ICE): type 'function() external' is not supported by the Soroban decoder for target soroban (at solang/src/codegen/targets/soroban/encoding.rs:255:14)

</details>

<a id="llvm-value-kind"></a>
### LLVM value-kind confusion (integer where a pointer is expected)

**Category:** bug · **Files:** 25 · **Rule:** `llvm-value-kind`

**What it means.** Solang's code generator hands LLVM an integer constant where a pointer is required (e.g. in `Binary::vector_bytes`, triggered by `bytes32 x = ""` and dynamic return types). This is an internal compiler error.

**Suggested fix.** Check the value kinds in emit for byte-array/string conversions on Soroban.

**Docs.** Internal error.

<details><summary>Files</summary>

- `abiEncoderV1/return_dynamic_types_cross_call_simple.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -123001739…
- `abiEncoderV2/calldata_array_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 2076556223…
- `abiEncoderV2/calldata_array_dynamic_index_access.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 2076556223…
- `abiEncoderV2/calldata_array_dynamic_static_short_reencode.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 896778812"…
- `abiEncoderV2/calldata_array_multi_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -103314749…
- `abiEncoderV2/calldata_array_static.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1572939921…
- `abiEncoderV2/calldata_array_struct_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -184234820…
- `abiEncoderV2/calldata_array_two_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -800344695…
- `abiEncoderV2/calldata_array_two_static.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 519426714"…
- `abiEncoderV2/calldata_struct_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1501966044…
- `abiEncoderV2/calldata_struct_simple.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1069563627…
- `array/evm_exceptions_out_of_band_access.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 981279420"…
- `array/memory.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1852907602…
- `calldata/calldata_bytes_external.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 794233379"…
- `calldata/calldata_internal_library.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -732474120…
- `functionCall/external_call.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -467655094…
- `functionCall/external_call_dynamic_returndata.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 2137741580…
- `inheritance/inherited_function_calldata_memory.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 2076556223…
- `libraries/library_staticcall_delegatecall.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -501769330…
- `revertStrings/calldata_array_dynamic_static_short_reencode.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 896778812"…
- `revertStrings/called_contract_has_code.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 638722032"…
- `types/mapping_enum_key_getter_v1.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1724265095…
- `types/mapping_enum_key_getter_v2.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 1724265095…
- `using/library_on_interface.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 638722032"…
- `various/literal_empty_string.sol` — compiler crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, is_const: true, is_null: false, is_undef: false, llvm_value: "i32 -109677199…

</details>

<a id="llvm-assert-call"></a>
### LLVM assertion: call with the wrong argument types

**Category:** bug · **Files:** 22 · **Rule:** `llvm-assert-call`

**What it means.** Solang emits an LLVM call whose arguments do not match the called function's signature; LLVM aborts (SIGABRT). This is an internal compiler error.

**Suggested fix.** Minimise one of these tests and compare the emitted call with the callee's declaration.

**Docs.** Internal error.

<details><summary>Files</summary>

- `array/array_storage_push_empty_length_address.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:636: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `constructor_with_params_diamond_inheritance.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionCall/bare_call_no_returndatacopy.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:636: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionCall/call_attached_library_function_on_function.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionCall/call_internal_function_via_expression.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionCall/member_accessors.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:636: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionTypes/function_type_library_internal.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `functionTypes/pass_function_types_internally.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `inheritance/dataLocation/external_public_calldata.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `inheritance/inherited_function_through_dispatch.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `inheritance/super_overload.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `interfaceID/homer.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `interfaceID/homer_interfaceId.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `libraries/internal_library_function_attached_to_internal_function_type.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `libraries/internal_library_function_attached_to_internal_function_type_named_selector.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `libraries/internal_library_function_pointer.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `libraries/using_for_overload.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `modifiers/function_modifier_library.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `using/library_through_module.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `viaYul/comparison.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:636: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `viaYul/delete.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…
- `virtualFunctions/internal_virtual_function_calls_through_dispatch.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Instructions.cpp:631: void llvm::CallInst::init(llvm::FunctionType*, llvm::Value*, llvm::ArrayRef<llvm::Value*>, llvm::ArrayRef<llvm::Ope…

</details>

<a id="modifier-dispatch"></a>
### Function modifiers crash Soroban dispatch

**Category:** bug · **Files:** 19 · **Rule:** `modifier-dispatch`

**What it means.** Almost every test here declares a function modifier. Compiling it for Soroban hits `unreachable!()` in the Soroban dispatch code. Modifiers are not documented as unsupported, so this is a compiler bug.

**Suggested fix.** Handle modifier-wrapped functions in `codegen/targets/soroban/dispatch.rs`.

**Docs.** Modifiers are not mentioned in the Soroban docs.

<details><summary>Files</summary>

- `arithmetics/checked_modifier_called_by_unchecked.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/break_in_modifier.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/continue_in_modifier.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_empty.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_local_variables.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_loop.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_loop_viair.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_multi_invocation.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_multi_invocation_viair.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_multi_with_return.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_multiple_times.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_multiple_times_local_vars.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/function_modifier_overriding.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/modifer_recursive.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/modifier_init_return.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/return_does_not_skip_modifier.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/return_in_modifier.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `modifiers/stacked_return_with_modifiers.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)
- `various/multi_modifiers.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/targets/soroban/dispatch.rs:38:26)

</details>

<a id="panic-unsupported-builtin"></a>
### Unsupported builtin crashes Soroban codegen

**Category:** bug · **Files:** 17 · **Rule:** `panic-unsupported-builtin`

**What it means.** The test uses a builtin (or storage operation) that Solang does not implement for Soroban. Instead of a compile error, the compiler panics in `emit/soroban/target.rs`.

**Suggested fix.** Reject the builtin during semantic analysis for the Soroban target.

**Docs.** docs/targets/soroban_language_compatibility.rst, intro (reject rather than miscompile).

<details><summary>Files</summary>

- `abiEncoderV2/calldata_array.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `abiEncoderV2/calldata_array_dynamic_static_dynamic.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `array/bytes_length_member.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `array/copying/bytes_inside_mappings.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `array/copying/copy_removes_bytes_data.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `array/delete/delete_removes_bytes_data.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `events/event_constructor.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `events/event_emit_via_interface.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `events/event_really_lots_of_data.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `isoltestTesting/account.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `libraries/library_call_in_homestead.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `state/msg_data.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `state/msg_sender.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `structs/copy_from_mapping.sol` — compiler crashed: solang panicked mid-compile (ICE): storage subscript is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `various/create_calldata.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `various/erc20.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)
- `various/store_bytes.sol` — compiler crashed: solang panicked mid-compile (ICE): this Soroban builtin is not supported for target soroban (at solang/src/emit/soroban/target.rs:27:5)

</details>

<a id="try-catch-polkadot-path"></a>
### `try`/`catch` runs Polkadot-only code

**Category:** bug · **Files:** 10 · **Rule:** `try-catch-polkadot-path`

**What it means.** Compiling `try`/`catch` for Soroban ends up in the Polkadot target's code generator, which panics with `not implemented`. The Soroban backend should either implement try/catch or reject it.

**Suggested fix.** Route try/catch to a Soroban implementation (host `try_call`) or emit a diagnostic.

**Docs.** try/catch is not mentioned in the Soroban docs.

<details><summary>Files</summary>

- `immutable/getter_call_in_constructor.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `salted_create/salted_create.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/assert.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/create.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/nested.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/simple.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/simple_notuple.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/structuredAndLowLevel.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/super_trivial.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)
- `tryCatch/trivial.sol` — compiler crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/codegen/targets/polkadot/try_catch.rs:38:9)

</details>

<a id="crash-other"></a>
### Other compiler crashes

**Category:** bug · **Files:** 10 · **Rule:** `crash-other`

**What it means.** Solang crashed (panic, out-of-bounds index, `unwrap` on `None`, …) instead of compiling or rejecting the test. A compiler must never crash on any input.

**Suggested fix.** Minimise each and file individually.

**Docs.** Internal error.

<details><summary>Files</summary>

- `array/copying/nested_array_element_calldata_to_storage.sol` — compiler crashed: solang panicked mid-compile (ICE): not an array (at solang/src/sema/types.rs:1441:18)
- `array/copying/nested_array_element_memory_to_storage.sol` — compiler crashed: solang panicked mid-compile (ICE): not an array (at solang/src/sema/types.rs:1441:18)
- `constructor/functions_called_by_constructor_through_dispatch.sol` — compiler crashed: solang panicked mid-compile (ICE): called `Option::unwrap()` on a `None` value (at solang/src/emit/instructions.rs:720:18)
- `functionCall/call_function_returning_nothing_via_pointer.sol` — compiler crashed: solang panicked mid-compile (ICE): called `Option::unwrap()` on a `None` value (at solang/src/emit/instructions.rs:720:18)
- `functionCall/inheritance/call_unimplemented_base.sol` — compiler crashed: solang panicked mid-compile (ICE): index out of bounds: the len is 0 but the index is 0 (at solang/src/emit/cfg.rs:231:29)
- `functionCall/send_zero_ether.sol` — compiler crashed: solang panicked mid-compile (ICE): Builtin should not be in the cfg (at solang/src/codegen/mod.rs:2003:18)
- `metaTypes/name_other_contract.sol` — compiler crashed: solang panicked mid-compile (ICE): index out of bounds: the len is 0 but the index is 0 (at solang/src/emit/instructions.rs:569:36)
- `modifiers/function_modifier_library_inheritance.sol` — compiler crashed: solang panicked mid-compile (ICE): index out of bounds: the len is 0 but the index is 0 (at solang/src/emit/cfg.rs:231:29)
- `storage/mappings_array2d_pop_delete.sol` — compiler crashed: solang panicked mid-compile (ICE): called `Option::unwrap()` on a `None` value (at solang/src/codegen/targets/soroban/arrays.rs:56:29)
- `storage/mappings_array_pop_delete.sol` — compiler crashed: solang panicked mid-compile (ICE): called `Option::unwrap()` on a `None` value (at solang/src/codegen/targets/soroban/arrays.rs:56:29)

</details>

<a id="stack-overflow"></a>
### Stack overflow on recursive struct types

**Category:** bug · **Files:** 6 · **Rule:** `stack-overflow`

**What it means.** The compiler recurses without end, typically on a struct that refers to itself through an array or mapping.

**Suggested fix.** Guard the recursive type walk in the Soroban encoding/layout code.

**Docs.** Internal error.

<details><summary>Files</summary>

- `structs/array_of_recursive_struct.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack
- `structs/conversion/recursive_storage_memory.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack
- `structs/conversion/recursive_storage_memory_complex.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack
- `structs/recursive_structs.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack
- `structs/struct_reference.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack
- `structs/structs.sol` — compiler crashed: killed by signal 6: thread 'main' has overflowed its stack

</details>

<a id="llvm-assert-other"></a>
### Other LLVM assertion failures

**Category:** bug · **Files:** 5 · **Rule:** `llvm-assert-other`

**What it means.** LLVM aborts on invalid IR produced by Solang (type mismatch in replace-all-uses, inliner assertion, …). Internal compiler error.

**Suggested fix.** Minimise and file individually.

**Docs.** Internal error.

<details><summary>Files</summary>

- `arithmetics/addmod_mulmod.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/Transforms/Utils/InlineFunction.cpp:2823: llvm::InlineResult llvm::InlineFunction(llvm::CallBase&, llvm::InlineFunctionInfo&, bool, llvm::AA…
- `arithmetics/addmod_mulmod_zero.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/Transforms/Utils/InlineFunction.cpp:2823: llvm::InlineResult llvm::InlineFunction(llvm::CallBase&, llvm::InlineFunctionInfo&, bool, llvm::AA…
- `array/delete/memory_arrays_delete.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Value.cpp:502: void llvm::Value::doRAUW(llvm::Value*, llvm::Value::ReplaceMetadataUses): Assertion `New && "Value::replaceAllUsesWith(<nu…
- `functionTypes/function_delete_stack.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Value.cpp:502: void llvm::Value::doRAUW(llvm::Value*, llvm::Value::ReplaceMetadataUses): Assertion `New && "Value::replaceAllUsesWith(<nu…
- `variables/delete_local.sol` — compiler crashed: killed by signal 6: sorobench: llvm/lib/IR/Value.cpp:502: void llvm::Value::doRAUW(llvm::Value*, llvm::Value::ReplaceMetadataUses): Assertion `New && "Value::replaceAllUsesWith(<nu…

</details>

<a id="constant-folding"></a>
### Crash in constant folding

**Category:** bug · **Files:** 5 · **Rule:** `constant-folding`

**What it means.** The optimiser's constant folding pass meets an expression it does not expect (`unreachable!` or `Poison`). Internal compiler error, not Soroban-specific code.

**Suggested fix.** Minimise and file; check which codegen path produces the unexpected expression.

**Docs.** Internal error.

<details><summary>Files</summary>

- `constructor/order_of_evaluation.sol` — compiler crashed: solang panicked mid-compile (ICE): expr should not be in cfg: Poison (at solang/src/codegen/optimize/constant_folding.rs:671:14)
- `constructor_inheritance_init_order_3_legacy.sol` — compiler crashed: solang panicked mid-compile (ICE): expr should not be in cfg: Poison (at solang/src/codegen/optimize/constant_folding.rs:671:14)
- `constructor_inheritance_init_order_3_viaIR.sol` — compiler crashed: solang panicked mid-compile (ICE): expr should not be in cfg: Poison (at solang/src/codegen/optimize/constant_folding.rs:671:14)
- `enums/enum_explicit_overflow.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimize/constant_folding.rs:730:14)
- `enums/enum_explicit_overflow_homestead.sol` — compiler crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimize/constant_folding.rs:730:14)

</details>

<a id="shift-by-width"></a>
### Shifting by the full bit width gives the wrong result

**Category:** bug · **Files:** 4 · **Rule:** `shift-by-width`

**What it means.** In Solidity, `x << 256` on a `uint256` is 0, and large right shifts of negative numbers give -1. On Soroban the result is the unshifted value (`0x4266 << 256` returns `0x4266`), which suggests the shift amount is taken modulo the width, as LLVM does for out-of-range shifts. No integer rounding is involved (`uint256`).

**Suggested fix.** Clamp the shift amount before emitting LLVM `shl`/`lshr`/`ashr` (shift amount ≥ width must yield 0, or -1 for negative `>>`).

**Docs.** Solidity semantics; not a documented Soroban difference.

<details><summary>Files</summary>

- `operators/shifts/shift_left.sol` — wrong value `f(uint256,uint256)`: expected [Int(0)], got Int(16998)
- `operators/shifts/shift_left_assignment.sol` — wrong value `f(uint256,uint256)`: expected [Int(0)], got Int(16998)
- `operators/shifts/shift_left_uint32.sol` — wrong value `f(uint32,uint32)`: expected [Int(0)], got Int(16998)
- `operators/shifts/shift_underflow_negative_rvalue.sol` — wrong value `f(int256,uint256)`: expected [Int(0)], got Int(57896044618658097711785492504343953926634992332820282019728792003956564819968)

</details>

<a id="symbol-too-long"></a>
### Overloaded function names exceed Soroban's symbol length

**Category:** bug · **Files:** 3 · **Rule:** `symbol-too-long`

**What it means.** Solang gives overloaded functions a mangled export name (e.g. `f_uint64ArrayArray_uint64_uint64`). Soroban limits function names to 32 characters, and Solang panics when the mangled name is longer.

**Suggested fix.** Use a shorter, hashed mangling for long names, or report a clean error.

**Docs.** Internal error.

<details><summary>Files</summary>

- `array/calldata_array_two_dimensional.sol` — compiler crashed: solang panicked mid-compile (ICE): function name "test_uint256ArrayArray2_uint256_uint256" exceeds limit (at solang/src/emit/soroban/mod.rs:401:41)
- `array/calldata_array_two_dimensional_1.sol` — compiler crashed: solang panicked mid-compile (ICE): function name "test_uint256ArrayArray_uint256_uint256" exceeds limit (at solang/src/emit/soroban/mod.rs:401:41)
- `calldata/calldata_array_access.sol` — compiler crashed: solang panicked mid-compile (ICE): function name "f_uint256ArrayArray_uint256_uint256" exceeds limit (at solang/src/emit/soroban/mod.rs:401:41)

</details>

<a id="timeout"></a>
### Timed out (over 10 s)

**Category:** review · **Files:** 22 · **Rule:** `timeout`

**What it means.** Compiling or running the test took longer than 10 seconds. It may be a compiler hang or simply a slow test.

**Suggested fix.** Re-run alone with a longer timeout to tell a hang from slowness.

<details><summary>Files</summary>

- `abiEncoderV2/calldata_dynamic_array_to_memory.sol` — timed out: exceeded 10s
- `abiEncoderV2/calldata_three_dimensional_dynamic_array_index_access.sol` — timed out: exceeded 10s
- `array/array_storage_length_access.sol` — timed out: exceeded 10s
- `array/array_storage_push_empty.sol` — timed out: exceeded 10s
- `array/array_storage_push_pop.sol` — timed out: exceeded 10s
- `array/copying/array_copy_storage_to_memory_nested.sol` — timed out: exceeded 10s
- `array/copying/array_elements_to_mapping.sol` — timed out: exceeded 10s
- `array/copying/array_nested_storage_to_memory.sol` — timed out: exceeded 10s
- `array/copying/array_to_mapping.sol` — timed out: exceeded 10s
- `array/copying/elements_of_nested_array_of_structs_calldata_to_storage.sol` — timed out: exceeded 10s
- `array/copying/elements_of_nested_array_of_structs_memory_to_storage.sol` — timed out: exceeded 10s
- `array/copying/nested_array_element_storage_to_memory.sol` — timed out: exceeded 10s
- `array/copying/nested_array_of_structs_calldata_to_storage.sol` — timed out: exceeded 10s
- `array/copying/nested_array_of_structs_memory_to_storage.sol` — timed out: exceeded 10s
- `array/copying/nested_array_of_structs_with_nested_array_from_storage_to_memory.sol` — timed out: exceeded 10s
- `array/fixed_arrays_in_storage.sol` — timed out: exceeded 10s
- `events/event_dynamic_nested_array_storage_v2.sol` — timed out: exceeded 10s
- `literals/denominations_in_array_sizes.sol` — timed out: exceeded 10s
- `structs/copy_struct_array_from_storage.sol` — timed out: exceeded 10s
- `structs/copy_substructures_from_mapping.sol` — timed out: exceeded 10s
- `structs/copy_substructures_to_mapping.sol` — timed out: exceeded 10s
- `structs/copy_to_mapping.sol` — timed out: exceeded 10s

</details>

<a id="int-width-probable"></a>
### Probably caused by integer rounding (needs confirmation)

**Category:** review · **Files:** 18 · **Rule:** `int-width-probable`

**What it means.** The test declares small integer types that Solang rounds up on Soroban, which is the likely cause. The failing call's own parameters are not small integers, so the link is not certain.

**Suggested fix.** Confirm by rewriting the test with 32/64/128/256-bit types.

**Docs.** docs/targets/soroban_language_compatibility.rst, 'Integer Widths'.

<details><summary>Files</summary>

- `array/push/array_push_nested_from_memory.sol` — failed at runtime `f()`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `cleanup/exp_cleanup.sol` — wrong value `f()`: expected [Int(1)], got Int(0)
- `cleanup/exp_cleanup_direct.sol` — wrong value `f()`: expected [Int(1)], got Int(0)
- `integer/uint.sol` — failed at runtime `uintMaxA()`: Error(Context, InvalidAction); log: runtime_error: require condition failed in test.sol:122:3-10
- `literals/ternary_operator_with_literal_types_overflow.sol` — should have reverted `g()`: returned Int(318) instead of reverting
- `operators/shifts/shift_cleanup.sol` — wrong value `f()`: expected [Int(0)], got Int(256)
- `operators/shifts/shift_cleanup_garbled.sol` — wrong value `f()`: expected [Int(0)], got Int(255)
- `revertStrings/calldata_too_short_v1.sol` — should have reverted `d(bytes)`: returned Int(0) instead of reverting
- `structs/struct_delete_storage.sol` — failed at runtime `f()`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `types/convert_fixed_bytes_to_uint_smaller_size.sol` — wrong value `bytesToUint(bytes4)`: expected [Int(25444)], got Int(1633837924)
- `types/mapping/copy_struct_to_array_stored_in_mapping.sol` — failed at runtime `from_storage_to_static_array()`: contract deployment failed: called `Result::unwrap()` on an `Err` value: HostError: Error(Context, InvalidAction)
- `types/packing_signed_types.sol` — wrong value `run()`: expected [Int(115792089237316195423570985008687907853269984665640564039457584007913129639930)], got Int(250)
- `viaYul/cleanup/comparison.sol` — wrong value `eq()`: expected [Bool(true)], got Bool(false)
- `viaYul/conversion/explicit_cast_local_assignment.sol` — wrong value `f(uint256)`: expected [Int(120)], got Int(305419896)
- `viaYul/conversion/implicit_cast_assignment.sol` — wrong value `f()`: expected [Int(120)], got Int(305419896)
- `viaYul/conversion/implicit_cast_function_call.sol` — wrong value `g()`: expected [Int(120)], got Int(305419896)
- `viaYul/conversion/implicit_cast_local_assignment.sol` — wrong value `f()`: expected [Int(120)], got Int(305419896)
- `viaYul/return_and_convert.sol` — wrong value `f()`: expected [Int(255)], got Int(65535)

</details>

<a id="hash-of-encode-packed"></a>
### Hashes differ (input built with `abi.encodePacked`)

**Category:** review · **Files:** 9 · **Rule:** `hash-of-encode-packed`

**What it means.** Solang's docs list `keccak256` and `sha256` as supported on Soroban, yet the hash differs from the EVM. All these tests hash the output of `abi.encodePacked(...)`. If `encodePacked` behaves like `abi.encode` on Soroban (host handles instead of bytes), the hash input differs and the hash is correct; if not, the hash itself is wrong. Strong bug candidate until one of the two is confirmed.

**Suggested fix.** Hash a plain literal (e.g. `keccak256("foo")`) on Soroban and compare with the known EVM value, to tell a hashing bug from an encoding difference.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Hash and cryptographic builtins': Supported (since v0.4.0).

<details><summary>Files</summary>

- `builtinFunctions/iterated_keccak256_with_bytes.sol` — wrong value `foo()`: expected [Bytes([179, 56, 238, 252, 226, 6, 249, 245, 123, 131, 170, 115, 141, 238, 205, 83, 38, 220, 75, 114, 221, 129, 238, 106, 124, 98, 26, 111, 172, 183, 1…
- `builtinFunctions/keccak256.sol` — wrong value `f(int256)`: expected [Bytes([138, 53, 172, 251, 193, 95, 248, 26, 57, 174, 125, 52, 79, 215, 9, 242, 142, 134, 0, 180, 170, 140, 101, 198, 182, 75, 254, 127, 227, 107, 209,…
- `builtinFunctions/keccak256_multiple_arguments.sol` — wrong value `foo(uint256,uint256,uint256)`: expected [Bytes([188, 116, 10, 152, 170, 229, 146, 62, 143, 4, 201, 170, 121, 140, 158, 232, 47, 105, 227, 25, 153, 118, 153, 242, 120, 44, 64, 130, 141, 185, 2…
- `builtinFunctions/keccak256_multiple_arguments_with_numeric_literals.sol` — wrong value `foo(uint256,uint16)`: expected [Bytes([136, 172, 212, 95, 117, 144, 126, 124, 86, 3, 24, 188, 26, 82, 73, 133, 10, 9, 153, 196, 137, 103, 23, 177, 22, 125, 5, 209, 22, 230, 219, 173]…
- `builtinFunctions/keccak256_multiple_arguments_with_string_literals.sol` — wrong value `bar(uint256,uint16)`: expected [Bytes([105, 144, 243, 100, 118, 220, 65, 43, 28, 75, 170, 72, 226, 217, 244, 170, 75, 179, 19, 246, 31, 218, 54, 124, 143, 219, 187, 34, 50, 220, 97, …
- `builtinFunctions/keccak256_packed.sol` — wrong value `f(int256)`: expected [Bytes([210, 112, 40, 91, 153, 102, 254, 252, 113, 85, 97, 239, 205, 9, 213, 182, 168, 222, 177, 85, 150, 247, 197, 60, 180, 161, 187, 115, 170, 85, 17…
- `builtinFunctions/sha256.sol` — wrong value `f(int256)`: expected [Bytes([227, 137, 144, 208, 199, 252, 0, 152, 128, 169, 192, 124, 35, 132, 46, 136, 108, 107, 189, 201, 100, 206, 107, 221, 88, 23, 173, 53, 115, 53, 2…
- `builtinFunctions/sha256_packed.sol` — wrong value `f(int256)`: expected [Bytes([128, 78, 13, 112, 3, 207, 215, 15, 201, 37, 220, 16, 49, 116, 217, 248, 152, 235, 177, 66, 236, 194, 162, 134, 218, 26, 189, 34, 172, 44, 227, …
- `cleanup/cleanup_bytes_types_shortening_newCodeGen.sol` — wrong value `f()`: expected [Bytes([255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])], got Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0,…

</details>

<a id="enum-range-not-checked"></a>
### Out-of-range enum values are accepted

**Category:** review · **Files:** 8 · **Rule:** `enum-range-not-checked`

**What it means.** The test passes a number that is not a valid value of the enum (e.g. 5 for an enum with 3 members) and expects the call to revert, as Solidity does. On Soroban the call succeeds. Solang's docs list enums as supported, so accepting invalid values is a strong bug candidate. It could also be a consequence of enums crossing the Soroban boundary as plain integers.

**Suggested fix.** Range-check enum arguments when decoding them in the Soroban dispatcher.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Enums and other complex user-defined types': enumerations supported (since v0.4.0).

<details><summary>Files</summary>

- `abiEncoderV2/enums.sol` — should have reverted `f(uint8)`: returned Int(2) instead of reverting
- `revertStrings/enum_v1.sol` — should have reverted `f(uint8[])`: returned (void) instead of reverting
- `revertStrings/enum_v2.sol` — should have reverted `f(uint8[])`: returned (void) instead of reverting
- `types/mapping_enum_key_library_v1.sol` — should have reverted `get(uint8)`: returned Int(0) instead of reverting
- `types/mapping_enum_key_library_v2.sol` — should have reverted `get(uint8)`: returned Int(0) instead of reverting
- `types/mapping_enum_key_v1.sol` — should have reverted `get(uint8)`: returned Int(0) instead of reverting
- `types/mapping_enum_key_v2.sol` — should have reverted `get(uint8)`: returned Int(0) instead of reverting
- `viaYul/mapping_enum_key_getter.sol` — should have reverted `table(uint8)`: returned Int(0) instead of reverting

</details>

<a id="deployment-failed"></a>
### Contract fails while being deployed

**Category:** review · **Files:** 3 · **Rule:** `deployment-failed`

**What it means.** The contract's constructor (or state initialisation) fails on the Soroban host, so no call can run. On the EVM the same contract deploys normally. Likely a Solang bug in constructor or initializer code.

**Suggested fix.** Minimise the constructor/initialisers and check the host error.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Contract model' (constructors supported).

<details><summary>Files</summary>

- `array/dynamic_array_cleanup.sol` — failed at runtime `fill()`: contract deployment failed: called `Result::unwrap()` on an `Err` value: HostError: Error(Context, InvalidAction)
- `storage/accessors_mapping_for_array.sol` — failed at runtime `data(uint256,uint256)`: contract deployment failed: called `Result::unwrap()` on an `Err` value: HostError: Error(Context, InvalidAction)
- `structs/struct_copy_via_local.sol` — failed at runtime `test()`: contract deployment failed: called `Result::unwrap()` on an `Err` value: HostError: Error(Context, InvalidAction)

</details>

<a id="budget-exceeded"></a>
### Contract exceeds the Soroban budget

**Category:** review · **Files:** 1 · **Rule:** `budget-exceeded`

**What it means.** The call used more CPU or memory than Soroban allows, even with sorobench's unlimited budget setting. This usually means a runaway loop or unbounded allocation in the generated code.

**Suggested fix.** Minimise and check for a loop that does not terminate.

<details><summary>Files</summary>

- `interfaceID/interfaceId_events.sol` — failed at runtime `hello_world()`: host panicked: HostError: Error(Budget, ExceededLimit)

</details>

<a id="soroban-only-rejection"></a>
### Rejected on Soroban only (not yet explained)

**Category:** soroban-gap · **Files:** 5 · **Rule:** `soroban-only-rejection`

**What it means.** Solang rejects this test for Soroban, but accepts the same code for its Polkadot target, so the limitation is specific to the Soroban backend. No dictionary rule explains it yet.

**Suggested fix.** Check the error against the Soroban docs and add a specific rule.

**Docs.** Check docs/targets/soroban_support_matrix.rst.

<details><summary>Files</summary>

- `arithmetics/signed_mod.sol` — rejected: value 57896044618658097711785492504343953926634992332820282019728792003956564819968 does not fit into type int256.; value 5789604461865809771178549250434395392663499233282028201972…
- `operators/shifts/shift_left_larger_type.sol` — rejected: left shift by 254 is not possible
- `structs/struct_referencing.sol` — rejected: Variable 's' is undefined; Variable 's' is undefined
- `viaYul/local_variable_without_init.sol` — rejected: Variable 'x' is undefined
- `viaYul/loops/return.sol` — rejected: Variable 'a' is undefined; Variable 'a' is undefined

</details>

<a id="solang-language-gap"></a>
### Rejected by Solang on every target

**Category:** solang-gap · **Files:** 174 · **Rule:** `solang-language-gap`

**What it means.** Solang rejects this code for Polkadot as well, so this is a general Solang language limitation (for example calldata slices, signed `**`, some function-type and array conversions, `super`, library `using for` calls), not something specific to Soroban.

**Suggested fix.** Report to Solang as a language feature, independent of the Soroban backend.

**Docs.** Not a Soroban topic.

<details><summary>Files</summary>

- `abiEncoderV1/abi_encode_calldata_slice.sol` — rejected: slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice…
- `abiEncoderV1/decode_slice.sol` — rejected: slice not supported yet
- `abiEncoderV2/abi_encode_calldata_slice.sol` — rejected: slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice…
- `abiEncoderV2/calldata_array_function_types.sol` — rejected: conversion from function() external returns (uint256)[] to function() external returns (uint256)[] not possible; conversion from function() external returns (uint256)[] to function…
- `abiEncoderV2/calldata_array_static_dynamic_static.sol` — rejected: conversion from uint32[1][] to uint32 not possible; conversion from uint32[1][] to uint32 not possible; conversion from uint256[2][] to uint256 not possible; conversion from uint25…
- `abiEncoderV2/struct/struct_function.sol` — rejected: method 'f' does not exist
- `abiencodedecode/abi_encode_call.sol` — rejected: 'length' not found; new cannot allocate type 'usertype C.UnsignedNumber'
- `abiencodedecode/abi_encode_call_is_consistent.sol` — rejected: variable cannot be declared external; 'fPointer' not found; first argument should be function, got 'function(uint256,string) external'; first argument should be function, got 'func…
- `abiencodedecode/abi_encode_call_memory.sol` — rejected: conversion from function() external to function() external not possible
- `abiencodedecode/contract_array.sol` — rejected: new cannot construct array of 'contract C'
- `abiencodedecode/contract_array_v2.sol` — rejected: new cannot construct array of 'contract C'
- `array/array_function_pointers.sol` — rejected: conversion from function() internal returns (uint256)[] to function() internal returns (uint256)[] not possible; conversion from function() internal returns (uint256)[][] to functi…
- `array/array_push_return_reference.sol` — rejected: expression is not assignable
- `array/bytes_to_fixed_bytes_simple.sol` — rejected: slice not supported yet
- `array/bytes_to_fixed_bytes_too_long.sol` — rejected: slice not supported yet
- `array/calldata_array_as_argument_internal_function.sol` — rejected: slice not supported yet
- `array/calldata_slice_access.sol` — rejected: slice not supported yet; slice not supported yet
- `array/concat/bytes_concat_different_types.sol` — rejected: slice not supported yet
- `array/copying/array_copy_calldata_storage.sol` — rejected: conversion from uint256[9] to uint256[] not possible
- `array/copying/array_copy_different_packing.sol` — rejected: conversion from bytes8[] to bytes10[] not possible
- `array/copying/array_copy_memory_to_storage.sol` — rejected: conversion from uint32[3] to uint32[] not possible
- `array/copying/array_copy_nested_array.sol` — rejected: conversion from uint256[2][] to uint256[4][] not possible
- `array/copying/array_copy_storage_storage_different_base.sol` — rejected: conversion from uint64[] to uint256[] not possible
- `array/copying/array_copy_storage_storage_different_base_nested.sol` — rejected: conversion from uint64[5][2] to uint128[6][3] not possible
- `array/copying/array_copy_storage_storage_static_dynamic.sol` — rejected: conversion from uint256[9] to uint256[] not possible
- `array/copying/array_copy_storage_storage_static_static.sol` — rejected: conversion from uint256[20] to uint256[40] not possible
- `array/copying/array_copy_target_simple.sol` — rejected: number of 8 bytes cannot be converted to type 'bytes17'
- `array/copying/array_copy_target_simple_2.sol` — rejected: number of 8 bytes cannot be converted to type 'bytes32'
- `array/copying/array_nested_memory_to_storage.sol` — rejected: conversion from uint256[2][] to uint256[4][] not possible; conversion from uint256[2][2] to uint256[2][3] not possible; conversion from uint256[2][3] to uint256[][] not possible
- `array/copying/array_of_function_external_storage_to_storage_dynamic.sol` — rejected: conversion from function() external[] to function() external[] not possible; conversion from function() external[] to function() external[] not possible; conversion from function()…
- `array/copying/array_of_function_external_storage_to_storage_dynamic_different_mutability.sol` — rejected: conversion from function() external[] to function() external[] not possible; conversion from function() external[] to function() external[] not possible; conversion from function()…
- `array/copying/calldata_to_storage_different_base.sol` — rejected: conversion from bytes8[] to bytes10[] not possible
- `array/copying/cleanup_during_multi_element_per_slot_copy.sol` — rejected: expected 'uint32[] storage', found integer
- `array/copying/copy_function_internal_storage_array.sol` — rejected: conversion from function() internal returns (uint256)[] to function() internal returns (uint256)[] not possible
- `array/copying/copy_internal_function_array_to_storage.sol` — rejected: conversion from function() internal returns (uint256)[20] to function() internal returns (uint256)[20] not possible
- `array/copying/function_type_array_to_storage.sol` — rejected: conversion from function() external[] to function() external[] not possible; conversion from function() external[] to function() external[] not possible; conversion from function()…
- `array/copying/memory_to_storage_different_base.sol` — rejected: conversion from bytes4[] to bytes10[] not possible
- `array/copying/nested_array_element_storage_to_storage.sol` — rejected: conversion from uint32[2][] to uint32[][] not possible
- `array/function_array_cross_calls.sol` — rejected: conversion from function() external returns (function() external returns (uint256))[] to function() external returns (function() external returns (uint256))[] not possible
- `array/function_memory_array.sol` — rejected: conversion from function(uint256) internal returns (uint256)[] to function(uint256) internal returns (uint256)[] not possible
- `array/indexAccess/inline_array_index_access_strings.sol` — rejected: conversion from bytes3 to string not possible
- `array/inline_array_return.sol` — rejected: conversion from uint32[5] to uint32[] not possible
- `array/pop/array_pop_isolated.sol` — rejected: 'pop' not found
- `array/pop/byte_array_pop_isolated.sol` — rejected: 'pop' not found
- `array/push/push_no_args_1d.sol` — rejected: expression is not assignable
- `array/push/push_no_args_2d.sol` — rejected: expression is not assignable
- `array/slices/array_slice_calldata_as_argument_of_external_calls.sol` — rejected: slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet
- `array/slices/array_slice_calldata_to_calldata.sol` — rejected: slice not supported yet; slice not supported yet
- `array/slices/array_slice_calldata_to_memory.sol` — rejected: slice not supported yet; slice not supported yet; slice not supported yet
- `array/slices/array_slice_calldata_to_storage.sol` — rejected: slice not supported yet
- `c99_scoping_activation.sol` — rejected: x is already declared; x is already declared; x is already declared; x is already declared
- `calldata/calldata_array_index_range_access.sol` — rejected: slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice not supported yet; slice…
- `calldata/calldata_attached_to_dynamic_array_or_slice.sol` — rejected: slice not supported yet
- `cleanup/exp_cleanup_nonzero_base.sol` — rejected: x is already declared
- `constantEvaluator/negative_fractional_mod.sol` — rejected: expression of type rational not allowed
- `constants/constants_at_file_level_referencing.sol` — rejected: 'a' not found; 'a' not found; 'b' not found; unknown function or type 'fre'; 'b' not found
- `constants/function_unreferenced.sol` — rejected: contract 'B' does not have a member called 'g'
- `constants/same_constants_different_files.sol` — rejected: 'M' is an import
- `constructor/function_usage_in_constructor_arguments.sol` — rejected: missing arguments to base contract 'BaseBase' constructor; unknown function or type 'g'; missing arguments to base contract 'BaseBase' constructor
- `deployedCodeExclusion/module_function_deployed.sol` — rejected: 'M' is an import
- `deployedCodeExclusion/super_function_deployed.sol` — rejected: 'super' not found
- `deployedCodeExclusion/virtual_function_deployed.sol` — rejected: 'super' not found
- `errors/error_selector.sol` — rejected: contract 'L' does not have a member called 'E'; contract 'L' does not have a member called 'E'; contract 'S' does not have a member called 'E'; contract 'T' does not have a member …
- `events/event_shadowing_file_level.sol` — rejected: multiple definitions of event
- `exponentiation/literal_base.sol` — rejected: exponation (**) is not allowed with signed types
- `expressions/conditional_expression_functions.sol` — rejected: expression of type function() internal returns (uint256) not allowed
- `expressions/exp_operator_const_signed.sol` — rejected: exponation (**) is not allowed with signed types
- `expressions/module_from_ternary_expression.sol` — rejected: 'M' is an import
- `expressions/tuple_from_ternary_expression.sol` — rejected: lists only permitted in destructure statements
- `expressions/uncalled_address_transfer_send.sol` — rejected: fallback function must not be declare payable, use 'receive() external payable' instead
- `fallback/fallback_argument.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values
- `fallback/fallback_argument_to_storage.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values
- `fallback/fallback_or_receive.sol` — rejected: fallback function must not be declare payable, use 'receive() external payable' instead
- `fallback/fallback_override.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values; fallback function cannot have parameters; fallback function cannot have return values
- `fallback/fallback_override2.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values; '' does not override anything
- `fallback/fallback_override_multi.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values; fallback function cannot have parameters; fallback function cannot have return values; '' doe…
- `fallback/fallback_return_data.sol` — rejected: fallback function cannot have parameters; fallback function cannot have return values
- `functionCall/call_attached_library_function_on_storage_variable.sol` — rejected: expression not expected here
- `functionCall/call_function_returning_function.sol` — rejected: return type 'function internal' not allowed in public or external functions; return type 'function internal' not allowed in public or external functions; return type 'function inte…
- `functionCall/call_internal_function_with_multislot_arguments_via_pointer.sol` — rejected: function arguments do not match in conversion from 'function(function() external returns (uint256),function() external returns (uint256)) internal returns (function() external retu…
- `functionCall/conditional_with_arguments.sol` — rejected: expression of type function(int256,int256) internal pure returns (int256) not allowed
- `functionCall/creation_function_call_no_args.sol` — rejected: expression found where type expected
- `functionCall/inheritance/super_skip_unimplemented_in_abstract_contract.sol` — rejected: function 'f' should specify 'override'
- `functionTypes/comparison_operators_for_external_functions.sol` — rejected: expression of type function() external not allowed; expression of type function() external not allowed
- `functionTypes/function_external_delete_storage.sol` — rejected: accessor function cannot be called via an internal function call
- `functionTypes/same_function_in_construction_and_runtime_equality_check.sol` — rejected: expression of type function(uint256) internal returns (uint256) not allowed
- `functionTypes/selector_ternary.sol` — rejected: expression of type function() external not allowed
- `functionTypes/selector_ternary_function_pointer_from_function_call.sol` — rejected: expression of type function() external not allowed
- `functionTypes/stack_height_check_on_adding_gas_variable_to_function.sol` — rejected: unexpect block encountered
- `functionTypes/store_function.sol` — rejected: function arguments do not match in conversion from 'function(function(uint256) external returns (uint256)) internal returns (uint256)' to 'function(function(uint256) external retur…
- `functionTypes/struct_with_external_function.sol` — rejected: method 'x' does not exist
- `functionTypes/struct_with_functions.sol` — rejected: method 'x' does not exist
- `functionTypes/ternary_contract_internal_function.sol` — rejected: expression of type function() internal pure returns (uint256) not allowed
- `functionTypes/ternary_contract_library_internal_function.sol` — rejected: expression of type function() internal pure returns (uint256) not allowed
- `functionTypes/ternary_contract_public_function.sol` — rejected: expression of type function() internal pure returns (uint256) not allowed
- `getters/struct_with_bytes.sol` — rejected: mapping in a struct variable cannot be public
- `getters/struct_with_bytes_simple.sol` — rejected: mapping in a struct variable cannot be public
- `immutable/multiple_initializations.sol` — rejected: 'm' not found; 'm' not found; missing arguments to base contract 'A' constructor; need instance of contract 'A' to get variable value 'x'; missing arguments to contract 'A' constru…
- `inheritance/base_access_to_function_type_variables.sol` — rejected: need instance of contract 'C' to get variable value 'x'; unknown function or type 'x'
- `inheritance/pass_dynamic_arguments_to_the_base_base.sol` — rejected: missing arguments to contract 'Base' constructor
- `inheritance/super_in_constructor.sol` — rejected: function 'f' override list does not contain 'A'
- `inheritance/super_in_constructor_assignment.sol` — rejected: 'super' not found; 'super' not found; 'super' not found; function 'f' override list does not contain 'A'
- `integer/basic.sol` — rejected: exponation (**) is not allowed with signed types
- `integer/int.sol` — rejected: exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed…
- `interfaceID/lisa.sol` — rejected: function 'supportsInterface' of abstract contract 'ERC165MappingImplementation' is overloaded
- `interfaceID/lisa_interfaceId.sol` — rejected: function 'supportsInterface' of abstract contract 'ERC165MappingImplementation' is overloaded
- `libraries/internal_library_function_attached_to_string_accepting_storage.sol` — rejected: function declared 'pure' but this expression reads from state
- `libraries/library_enum_as_an_expression.sol` — rejected: contract 'Arst' does not have a member called 'Foo'
- `libraries/library_stray_values.sol` — rejected: 'Lib' is a contract
- `libraries/library_struct_as_an_expression.sol` — rejected: contract 'Arst' does not have a member called 'Foo'
- `libraries/using_for_by_name.sol` — rejected: method 'mul' does not exist
- `modifiers/access_through_contract_name.sol` — rejected: unknown modifier 'A.m' on function
- `modifiers/access_through_module_name.sol` — rejected: unknown modifier 'M.M.C.m' on function
- `modifiers/evaluation_order.sol` — rejected: 'm2' not found; 'm1' not found; 'm3' not found
- `modifiers/function_modifier_calling_functions_in_creation_context.sol` — rejected: 'mod1' not found
- `modifiers/function_modifier_for_constructor.sol` — rejected: 'mod1' not found
- `modifiers/function_modifier_return_reference.sol` — rejected: 'x' not found; 'y' not found
- `modifiers/function_return_parameter.sol` — rejected: 'r' not found
- `modifiers/function_return_parameter_complex.sol` — rejected: 'r1' not found; 'r3' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found; 'r' not found
- `modifiers/modifier_in_constructor_ice.sol` — rejected: 'm1' not found
- `modifiers/modifiers_in_construction_context.sol` — rejected: 'm1' not found; 'm1' not found; 'm2' not found
- `multiSource/circular_import.sol` — rejected: overloaded function with this signature already exist; unknown function or type 'f'; import 's2.sol' does not export 'f'
- `multiSource/circular_import_2.sol` — rejected: overloaded function with this signature already exist; unknown function or type 'h'; import 's2.sol' does not export 'f'; import 's2.sol' does not export 'g'; unknown function or t…
- `multiSource/circular_reimport.sol` — rejected: unknown function or type 'f'; overloaded function with this signature already exist; import 's1.sol' does not export 'f'
- `multiSource/circular_reimport_2.sol` — rejected: unknown function or type 'f'; overloaded function with this signature already exist; import 's2.sol' does not export 'f'; import 's2.sol' does not export 'g'; unknown function or t…
- `multiSource/free_different_interger_types.sol` — rejected: g is already defined as a function; conversion from bool to uint32 not possible
- `multiSource/imported_free_function_via_alias.sol` — rejected: overloaded function with this signature already exist; unknown function or type 'f'
- `multiSource/imported_free_function_via_alias_direct_call.sol` — rejected: overloaded function with this signature already exist; unknown function or type 'f'
- `operators/userDefined/fixed_point_udvt_with_operators.sol` — rejected: exponation (**) is not allowed with signed types
- `revertStrings/array_slices.sol` — rejected: slice not supported yet
- `revertStrings/library_non_view_call.sol` — rejected: 'L' is a contract
- `specialFunctions/abi_functions_member_access.sol` — rejected: 'abi' not found
- `storage/struct_accessor.sol` — rejected: mapping in a struct variable cannot be public
- `strings/concat/string_concat_different_types.sol` — rejected: slice not supported yet
- `structs/function_type_copy.sol` — rejected: conversion from function() external[] to function() external[] not possible; conversion from function() external[] to function() external[] not possible
- `structs/lone_struct_array_type.sol` — rejected: expected expression before ']' token
- `structs/multislot_struct_allocation.sol` — rejected: method 'x' does not exist
- `structs/struct_memory_to_storage_function_ptr.sol` — rejected: method 'f' does not exist
- `structs/struct_storage_to_memory_function_ptr.sol` — rejected: method 'f' does not exist
- `tryCatch/lowLevel.sol` — rejected: b is already declared; conversion from bool to uint256 not possible
- `tryCatch/panic.sol` — rejected: b is already declared; conversion from bool to uint256 not possible; b is already declared; conversion from bool to uint256 not possible
- `tryCatch/return_function.sol` — rejected: type 'function() external' does not match return value of function 'function() external'
- `tryCatch/structured.sol` — rejected: b is already declared; conversion from bool to uint256 not possible
- `tryCatch/try_catch_library_call.sol` — rejected: try only supports external calls or constructor calls; try only supports external calls or constructor calls
- `types/array_mapping_abstract_constructor_param.sol` — rejected: parameter of type 'storage' not allowed public or external functions; default constructor does not take arguments
- `types/mapping_abstract_constructor_param.sol` — rejected: parameter of type 'storage' not allowed public or external functions; default constructor does not take arguments
- `types/nested_tuples.sol` — rejected: lists only permitted in destructure statements; lists only permitted in destructure statements; lists only permitted in destructure statements; lists only permitted in destructure …
- `types/struct_mapping_abstract_constructor_param.sol` — rejected: parameter of type 'storage' not allowed public or external functions; default constructor does not take arguments
- `types/tuple_assign_multi_slot_grow.sol` — rejected: lists only permitted in destructure statements
- `userDefinedValueType/calldata.sol` — rejected: data location 'calldata' can only be specified for array, struct or mapping; data location 'memory' can only be specified for array, struct or mapping; data location 'memory' only …
- `userDefinedValueType/calldata_to_storage.sol` — rejected: data location 'calldata' can only be specified for array, struct or mapping; data location 'memory' can only be specified for array, struct or mapping; data location 'calldata' can…
- `userDefinedValueType/in_parenthesis.sol` — rejected: 'MyInt' is an user type
- `userDefinedValueType/memory_to_storage.sol` — rejected: data location 'memory' can only be specified for array, struct or mapping; data location 'memory' can only be specified for array, struct or mapping; data location 'memory' can onl…
- `userDefinedValueType/wrap_unwrap.sol` — rejected: 'MyAddress' is an user type
- `using/imported_functions.sol` — rejected: method 'f' does not exist
- `using/private_library_function.sol` — rejected: cannot call private library function
- `various/codehash.sol` — rejected: 'codehash' not found; 'codehash' not found; 'codehash' not found
- `various/crazy_elementary_typenames_on_stack.sol` — rejected: type not expected
- `various/flipping_sign_tests.sol` — rejected: exponation (**) is not allowed with signed types
- `various/skip_dynamic_types_for_static_arrays_with_dynamic_elements.sol` — rejected: conversion from bool[] to bool not possible
- `various/skip_dynamic_types_for_structs.sol` — rejected: destructuring assignment has 3 elements on the left and 4 on the right
- `various/state_variable_local_variable_mixture.sol` — rejected: need instance of contract 'A' to get variable value 'y'
- `various/state_variable_under_contract_name.sol` — rejected: need instance of contract 'Scope' to get variable value 'stateVar'
- `various/super.sol` — rejected: function 'f' override list does not contain 'A'
- `various/super_alone.sol` — rejected: 'super' not found
- `various/super_parentheses.sol` — rejected: 'super' not found; 'super' not found; function 'f' override list does not contain 'A'; 'super' not found
- `viaYul/conversion/explicit_cast_function_call.sol` — rejected: number of 4 bytes cannot be converted to type 'bytes32'
- `viaYul/conversion/explicit_string_bytes_calldata_cast.sol` — rejected: slice not supported yet
- `viaYul/copy_struct_invalid_ir_bug.sol` — rejected: method 'el' does not exist
- `viaYul/exp_literals.sol` — rejected: exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed…
- `viaYul/exp_literals_success.sol` — rejected: exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types; exponation (**) is not allowed…
- `viaYul/exp_neg.sol` — rejected: exponation (**) is not allowed with signed types
- `viaYul/exp_neg_overflow.sol` — rejected: exponation (**) is not allowed with signed types; exponation (**) is not allowed with signed types
- `virtualFunctions/virtual_function_usage_in_constructor_arguments.sol` — rejected: missing arguments to base contract 'BaseBase' constructor; unknown function or type 'g'; missing arguments to base contract 'BaseBase' constructor

</details>

<a id="multi-return"></a>
### Multiple return values from public functions

**Category:** documented-unsupported · **Files:** 201 · **Rule:** `multi-return`

**What it means.** Solidity functions can return several values at once (`returns (uint, bool)`). On Soroban a contract function returns a single value, and Solang's docs state that external functions return at most one value. The same code compiles for Solang's other targets. This is the single largest reason tests are rejected.

**Suggested fix.** Return the values packed into one Soroban value (a `Vec` or struct-like `Map`) and unpack them on the calling side.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Contract model': external functions return at most one value (documented since v0.4.0).

<details><summary>Files</summary>

- `abiEncoderV1/abi_decode_fixed_arrays.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV1/abi_encode_decode_simple.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV1/byte_arrays.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV1/calldata_bytes_bytes32_arrays.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV1/dynamic_arrays.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV1/memory_params_in_external_function.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV1/struct/struct_storage_ptr.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/abi_encode_empty_string_v2.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/abi_encode_v2_in_modifier_used_in_v1_contract.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban
- `abiEncoderV2/abi_encoder_v2_head_overflow_with_static_array_cleanup_bug.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/byte_arrays.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV2/calldata_array_dynamic_static_in_library.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/calldata_overlapped_dynamic_arrays.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/calldata_struct_member_offset.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/calldata_with_garbage.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV2/dynamic_arrays.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/dynamic_nested_arrays.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV2/memory_dynamic_array_and_calldata_static_array.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/memory_params_in_external_function.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `abiEncoderV2/struct/mediocre2_struct.sol` — rejected: Soroban external functions can return at most one value
- `abiEncoderV2/struct/mediocre_struct.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_decode_calldata.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_decode_simple.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_decode_simple_storage.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_encode_empty_string_v1.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_encode_with_signature.sol` — rejected: Soroban external functions can return at most one value
- `abiencodedecode/abi_encode_with_signaturev2.sol` — rejected: Soroban external functions can return at most one value
- `array/array_memory_allocation/array_static_return_param_zeroed_memory_index_access.sol` — rejected: Soroban external functions can return at most one value
- `array/calldata_array.sol` — rejected: Soroban external functions can return at most one value
- `array/calldata_array_of_struct.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_copy_including_array.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_copy_storage_storage_dyn_dyn.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_copy_storage_storage_dynamic_dynamic.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_copy_storage_storage_struct.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_copy_storage_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_nested_calldata_to_memory.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `array/copying/array_nested_calldata_to_storage.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `array/copying/array_of_struct_calldata_to_memory.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `array/copying/array_of_struct_calldata_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_of_struct_memory_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_of_structs_containing_arrays_calldata_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_of_structs_containing_arrays_calldata_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_of_structs_containing_arrays_memory_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/array_storage_multi_items_per_slot.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/calldata_array_of_struct_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/calldata_array_static_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/calldata_bytes_array_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/calldata_dynamic_array_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/copy_byte_array_in_struct_to_storage.sol` — rejected: Variable 'x' is undefined; Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return a…
- `array/copying/storage_memory_nested.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/storage_memory_nested_from_pointer.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/storage_memory_nested_struct.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/storage_memory_packed.sol` — rejected: Soroban external functions can return at most one value
- `array/copying/storage_memory_packed_dyn.sol` — rejected: Soroban external functions can return at most one value
- `array/create_memory_array.sol` — rejected: Soroban external functions can return at most one value
- `array/dynamic_arrays_in_storage.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `array/external_array_args.sol` — rejected: Soroban external functions can return at most one value
- `array/fixed_arrays_as_return_type.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban
- `array/fixed_bytes_length_access.sol` — rejected: Soroban external functions can return at most one value
- `array/indexAccess/arrays_complex_memory_index_access.sol` — rejected: Soroban external functions can return at most one value
- `array/indexAccess/bytes_index_access_memory.sol` — rejected: Soroban external functions can return at most one value
- `array/indexAccess/bytes_memory_index_access.sol` — rejected: Soroban external functions can return at most one value
- `array/inline_array_storage_to_memory_conversion_ints.sol` — rejected: Soroban external functions can return at most one value
- `array/inline_array_storage_to_memory_conversion_strings.sol` — rejected: Soroban external functions can return at most one value
- `array/pop/array_pop.sol` — rejected: Soroban external functions can return at most one value
- `array/pop/array_pop_array_transition.sol` — rejected: Soroban external functions can return at most one value
- `array/pop/array_pop_uint16_transition.sol` — rejected: Soroban external functions can return at most one value
- `array/pop/array_pop_uint24_transition.sol` — rejected: Soroban external functions can return at most one value
- `array/pop/byte_array_pop.sol` — rejected: Soroban external functions can return at most one value
- `array/push/array_push.sol` — rejected: Soroban external functions can return at most one value
- `array/push/array_push_packed_array.sol` — rejected: Soroban external functions can return at most one value
- `array/push/array_push_struct.sol` — rejected: Soroban external functions can return at most one value
- `array/push/array_push_struct_from_calldata.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_array_dynamic_bytes.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `calldata/calldata_array_length.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_array_three_dimensional.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_attached_to_bytes.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_attached_to_static_array.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_attached_to_struct.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_internal_multi_array.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `calldata/calldata_internal_multi_fixed_array.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `calldata/calldata_memory_mixed.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `calldata/calldata_string_array.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_struct.sol` — rejected: Soroban external functions can return at most one value
- `calldata/calldata_struct_internal.sol` — rejected: Soroban external functions can return at most one value
- `calldata/copy_from_calldata_removes_bytes_data.sol` — rejected: Variable 'emptyData' is undefined; Soroban external functions can return at most one value
- `cleanup/cleanup_in_compound_assign.sol` — rejected: Soroban external functions can return at most one value
- `constantEvaluator/rounding.sol` — rejected: Soroban external functions can return at most one value
- `constants/constant_string_at_file_level.sol` — rejected: Soroban external functions can return at most one value
- `constants/consteval_array_length.sol` — rejected: Soroban external functions can return at most one value
- `constructor/arrays_in_constructors.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban
- `constructor/bytes_in_constructors_packer.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban
- `enums/enum_with_256_members.sol` — rejected: Soroban external functions can return at most one value
- `events/event_selector.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `events/event_selector_file_level.sol` — rejected: Soroban external functions can return at most one value
- `expressions/conditional_expression_tuples.sol` — rejected: Soroban external functions can return at most one value
- `expressions/conditional_expression_with_return_values.sol` — rejected: Soroban external functions can return at most one value
- `freeFunctions/import.sol` — rejected: Soroban external functions can return at most one value
- `freeFunctions/libraries_from_free.sol` — rejected: Soroban external functions can return at most one value
- `freeFunctions/overloads.sol` — rejected: Soroban external functions can return at most one value
- `freeFunctions/storage_calldata_refs.sol` — rejected: Soroban external functions can return at most one value
- `functionCall/external_function.sol` — rejected: Soroban external functions can return at most one value
- `functionCall/file_level_call_via_module.sol` — rejected: Soroban external functions can return at most one value
- `functionCall/mapping_array_internal_argument.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `functionCall/mapping_internal_argument.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `functionCall/mapping_internal_return.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `functionCall/multiple_return_values.sol` — rejected: Soroban external functions can return at most one value
- `functionSelector/function_selector_via_contract_name.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `functionTypes/address_member.sol` — rejected: Soroban external functions can return at most one value
- `functionTypes/selector_1.sol` — rejected: Soroban external functions can return at most one value
- `functionTypes/selector_2.sol` — rejected: Soroban external functions can return at most one value
- `immutable/increment_decrement.sol` — rejected: Soroban external functions can return at most one value
- `immutable/inheritance.sol` — rejected: Soroban external functions can return at most one value
- `immutable/multi_creation.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban; contract construction is not supported for target soroban; contr…
- `immutable/stub.sol` — rejected: Soroban external functions can return at most one value
- `immutable/uninitialized.sol` — rejected: Soroban external functions can return at most one value
- `immutable/use_scratch.sol` — rejected: Soroban external functions can return at most one value
- `inheritance/access_base_storage.sol` — rejected: Soroban external functions can return at most one value
- `interface_inheritance_conversions.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; contract construction is not supported for target soroban; contrac…
- `libraries/attached_internal_library_function_accepting_calldata.sol` — rejected: Soroban external functions can return at most one value
- `libraries/attached_internal_library_function_returning_calldata.sol` — rejected: Soroban external functions can return at most one value
- `libraries/attached_public_library_function_accepting_calldata.sol.sol` — rejected: Soroban external functions can return at most one value
- `libraries/attached_public_library_function_returning_calldata.sol` — rejected: Soroban external functions can return at most one value
- `libraries/internal_types_in_library.sol` — rejected: Soroban external functions can return at most one value
- `libraries/mapping_returns_in_library_named.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `libraries/using_for_storage_structs.sol` — rejected: Soroban external functions can return at most one value
- `libraries/using_library_mappings_public.sol` — rejected: Soroban external functions can return at most one value
- `libraries/using_library_mappings_return.sol` — rejected: Soroban external functions can return at most one value
- `libraries/using_library_structs.sol` — rejected: Soroban external functions can return at most one value
- `literals/escape.sol` — rejected: Soroban external functions can return at most one value
- `multiSource/import_overloaded_function.sol` — rejected: Soroban external functions can return at most one value
- `smoke/alignment.sol` — rejected: Soroban external functions can return at most one value; contract construction is not supported for target soroban
- `smoke/arrays.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `smoke/bytes_and_strings.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `specialFunctions/keccak256_optimized.sol` — rejected: Soroban external functions can return at most one value
- `storage/packed_storage_overflow.sol` — rejected: Soroban external functions can return at most one value
- `storage/packed_storage_signed.sol` — rejected: Soroban external functions can return at most one value
- `strings/empty_storage_string.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `strings/empty_string_input.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `structs/calldata/calldata_nested_structs.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `structs/calldata/calldata_struct.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_and_ints.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_array_member.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_array_member_dynamic.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_as_argument_of_lib_function.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_as_memory_argument.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_struct_member.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_struct_member_dynamic.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_to_memory_tuple_assignment.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_with_array_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_with_bytes_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_with_nested_array_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_struct_with_nested_array_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/calldata_structs.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/dynamic_nested.sol` — rejected: Soroban external functions can return at most one value
- `structs/calldata/dynamically_encoded.sol` — rejected: Soroban external functions can return at most one value
- `structs/global.sol` — rejected: Soroban external functions can return at most one value
- `structs/memory_structs_as_function_args.sol` — rejected: Soroban external functions can return at most one value
- `structs/memory_structs_nested.sol` — rejected: Soroban external functions can return at most one value
- `structs/memory_structs_nested_load.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `structs/memory_structs_read_write.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `structs/msg_data_to_struct_member_copy.sol` — rejected: Soroban external functions can return at most one value
- `structs/struct_assign_reference_to_struct.sol` — rejected: Soroban external functions can return at most one value
- `structs/struct_constructor_nested.sol` — rejected: Soroban external functions can return at most one value
- `structs/struct_copy.sol` — rejected: Soroban external functions can return at most one value
- `structs/struct_memory_to_storage.sol` — rejected: Soroban external functions can return at most one value
- `structs/struct_storage_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `types/assign_calldata_value_type.sol` — rejected: Soroban external functions can return at most one value
- `types/strings.sol` — rejected: Soroban external functions can return at most one value
- `userDefinedValueType/simple.sol` — rejected: Soroban external functions can return at most one value
- `using/module_renamed.sol` — rejected: Soroban external functions can return at most one value
- `using/using_global_all_the_types.sol` — rejected: Soroban external functions can return at most one value
- `using/using_global_library.sol` — rejected: Soroban external functions can return at most one value
- `variables/delete_locals.sol` — rejected: Soroban external functions can return at most one value
- `variables/mapping_local_assignment.sol` — rejected: Soroban external functions can return at most one value
- `variables/mapping_local_compound_assignment.sol` — rejected: Soroban external functions can return at most one value
- `variables/mapping_local_tuple_assignment.sol` — rejected: Soroban external functions can return at most one value
- `variables/public_state_overridding_dynamic_struct.sol` — rejected: Soroban external functions can return at most one value
- `variables/public_state_overridding_mapping_to_dynamic_struct.sol` — rejected: Soroban external functions can return at most one value
- `various/inline_member_init.sol` — rejected: Soroban external functions can return at most one value
- `various/multi_variable_declaration.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `various/nested_calldata_struct.sol` — rejected: Soroban external functions can return at most one value
- `various/nested_calldata_struct_to_memory.sol` — rejected: Soroban external functions can return at most one value
- `various/skip_dynamic_types.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `various/string_tuples.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `various/tuples.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/assign_tuple_from_function_call.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `viaYul/comparison_functions.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `viaYul/conditional/conditional_tuple.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/conditional/conditional_with_assignment.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/conditional/conditional_with_variables.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/define_tuple_from_function_call.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `viaYul/if.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `viaYul/local_tuple_assignment.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban e…
- `viaYul/return_storage_pointers.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/short_circuit.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value
- `viaYul/simple_assignment.sol` — rejected: Soroban external functions can return at most one value
- `viaYul/struct_member_access.sol` — rejected: Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban external functions can return at most one value; Soroban e…
- `viaYul/tuple_evaluation_order.sol` — rejected: Soroban external functions can return at most one value

</details>

<a id="new-contract"></a>
### Creating contracts with `new Contract()`

**Category:** documented-unsupported · **Files:** 17 · **Rule:** `new-contract`

**What it means.** The test deploys another contract with `new C()`. Solang does not support this syntax on Soroban; contracts must be deployed from an uploaded Wasm hash instead.

**Suggested fix.** Use the `deployContract(wasm_hash, salt, ...args)` builtin. Supporting `new C()` would require embedding and uploading the child contract's Wasm.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Deploying and upgrading contracts': `new Contract()` not yet supported.

<details><summary>Files</summary>

- `array/reusing_memory.sol` — rejected: contract construction is not supported for target soroban
- `constructor/evm_exceptions_in_constructor_call_fail.sol` — rejected: contract construction is not supported for target soroban
- `constructor/no_callvalue_check.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban; contract construction is not supported for target soroban
- `freeFunctions/new_operator.sol` — rejected: contract construction is not supported for target soroban
- `functionCall/external_call_at_construction_time.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban
- `functionCall/failed_create.sol` — rejected: contract construction is not supported for target soroban
- `inheritance/address_overload_resolution.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban
- `inheritance/inherited_function_calldata_calldata_interface.sol` — rejected: contract construction is not supported for target soroban
- `inheritance/inherited_function_calldata_memory_interface.sol` — rejected: contract construction is not supported for target soroban
- `inheritance/member_notation_ctor.sol` — rejected: contract construction is not supported for target soroban
- `libraries/internal_library_function_attached_to_contract.sol` — rejected: contract construction is not supported for target soroban
- `libraries/internal_library_function_attached_to_interface.sol` — rejected: contract construction is not supported for target soroban
- `salted_create/prediction_example.sol` — rejected: contract construction is not supported for target soroban
- `various/contract_binary_dependencies.sol` — rejected: contract construction is not supported for target soroban
- `various/many_subassemblies.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban; contract construction is not supported for target soroban; con…
- `various/staticcall_for_view_and_pure.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban; contract construction is not supported for target soroban
- `various/staticcall_for_view_and_pure_pre_byzantium.sol` — rejected: contract construction is not supported for target soroban; contract construction is not supported for target soroban; contract construction is not supported for target soroban

</details>

<a id="value-transfer-uint512"></a>
### Native value transfer (`{value: …}`, `.balance`, `.transfer`)

**Category:** documented-unsupported · **Files:** 12 · **Rule:** `value-transfer-uint512`

**What it means.** These tests send or read native currency (`{value: x}`, `{gas: x}`, `address.balance`, `.transfer`). Soroban has no native value attached to calls, and Solang documents payable-style flows as unsupported. The error message about `uint512` is an unhelpful symptom of that.

**Suggested fix.** Reject value/gas call options and `balance`/`transfer` on Soroban with a direct message ("native value transfer is not supported on Soroban") instead of the uint512 truncation error.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Native value transfer and payable-style flows': Unsupported.

<details><summary>Files</summary>

- `functionCall/call_options_overload.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `functionCall/external_call_value.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `functionCall/gas_and_value_basic.sol` — rejected: implicit conversion would truncate from uint512 to uint256; implicit conversion would truncate from uint512 to uint256
- `functionCall/gas_and_value_brace_syntax.sol` — rejected: implicit conversion would truncate from uint512 to uint256; implicit conversion would truncate from uint512 to uint256
- `inheritance/value_for_constructor.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `revertStrings/transfer.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `salted_create/salted_create_with_value.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `smoke/constructor.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `various/balance.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `various/senders_balance.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `various/value_complex.sol` — rejected: implicit conversion would truncate from uint512 to uint256
- `various/value_insane.sol` — rejected: implicit conversion would truncate from uint512 to uint256

</details>

<a id="struct-public-getter"></a>
### Public getters for struct state variables

**Category:** documented-unsupported · **Files:** 9 · **Rule:** `struct-public-getter`

**What it means.** A `public` state variable of struct type needs an automatically generated getter that returns the struct. Solang's docs state that structs are not yet supported as public state-variable accessor return values on Soroban (user-written functions returning structs do work).

**Suggested fix.** Generate the getter the same way as a user-written function returning the struct.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Structs': not yet supported as `public` accessor return values (documented since v0.4.0).

<details><summary>Files</summary>

- `array/string_allocation_bug.sol` — rejected: type 'struct Sample.s' is not supported as a Soroban public variable accessor return value
- `array/strings_in_struct.sol` — rejected: type 'struct buggystruct.Buggy' is not supported as a Soroban public variable accessor return value
- `getters/array_mapping_struct.sol` — rejected: type 'struct C.Y' is not supported as a Soroban public variable accessor return value; type 'struct C.Y' is not supported as a Soroban public variable accessor return value
- `getters/mapping_array_struct.sol` — rejected: type 'struct C.Y' is not supported as a Soroban public variable accessor return value; type 'struct C.Y' is not supported as a Soroban public variable accessor return value
- `getters/mapping_to_struct.sol` — rejected: type 'struct C.S' is not supported as a Soroban public variable accessor return value
- `storage/array_accessor.sol` — rejected: type 'struct test.st' is not supported as a Soroban public variable accessor return value
- `structs/struct_named_constructor.sol` — rejected: type 'struct C.S' is not supported as a Soroban public variable accessor return value
- `various/negative_stack_height.sol` — rejected: type 'struct C.Invoice' is not supported as a Soroban public variable accessor return value
- `various/swap_in_storage_overwrite.sol` — rejected: type 'struct c.S' is not supported as a Soroban public variable accessor return value; type 'struct c.S' is not supported as a Soroban public variable accessor return value

</details>

<a id="struct-in-event"></a>
### Structs as event parameters

**Category:** documented-unsupported · **Files:** 4 · **Rule:** `struct-in-event`

**What it means.** The test emits an event that has a struct field. Solang documents structs in `event` declarations as not yet supported on Soroban.

**Suggested fix.** Encode the struct as a Soroban `Map` in the event data.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Structs'.

<details><summary>Files</summary>

- `events/event_signature_in_library.sol` — rejected: type 'struct L.S' is not supported as a Soroban event parameter; type 'struct L.S' is not supported as a Soroban event parameter
- `events/event_struct_memory_v2.sol` — rejected: type 'struct C.S' is not supported as a Soroban event parameter
- `events/event_struct_storage_v2.sol` — rejected: type 'struct C.S' is not supported as a Soroban event parameter
- `structs/event.sol` — rejected: type 'struct Item' is not supported as a Soroban event parameter

</details>

<a id="int-width-bytes-conversion"></a>
### Conversions between small integers and `bytesN`

**Category:** documented-difference · **Files:** 25 · **Rule:** `int-width-bytes-conversion`

**What it means.** On Soroban, Solang rounds integer types up to 32/64/128/256 bits (`uint8` becomes `uint32`). A conversion such as `bytes1(uint8 x)` is only legal between types of the same size, so after rounding it becomes `bytes1(uint32 x)` and is rejected. The compiler's rounding warning on this test is the cause.

**Suggested fix.** Keep the declared width for conversions (convert before rounding), or explain in the rounding warning that `bytesN` conversions will change.

**Docs.** docs/targets/soroban_language_compatibility.rst, 'Integer Widths'.

<details><summary>Files</summary>

- `abiEncoderV1/return_dynamic_types_cross_call_advanced.sol` — rejected: number of 32 bytes cannot be converted to type 'bytes20'
- `abiEncoderV2/struct/struct_simple.sol` — rejected: conversion to uint32 from bytes2 not allowed
- `abiencodedecode/abi_encode_call_uint_bytes.sol` — rejected: slice not supported yet; conversion to bytes2 from uint32 not allowed; conversion to uint32 from bytes2 not allowed
- `array/byte_array_transitional_2.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/copying/array_copy_storage_storage_static_simple.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/copying/array_copy_target_leftover.sol` — rejected: conversion to uint32 from bytes2 not allowed
- `array/copying/array_copy_target_leftover2.sol` — rejected: conversion to bytes10 from uint128 not allowed
- `array/copying/bytes_storage_to_storage.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/delete/bytes_delete_element.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/indexAccess/bytes_index_access.sol` — rejected: implicit conversion to uint256 from bytes1 not allowed; expression of type bytes1 not allowed
- `array/indexAccess/fixed_bytes_index_access.sol` — rejected: conversion to uint32 from bytes1 not allowed
- `array/pop/byte_array_pop_long_storage_empty.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/push/byte_array_push_transition.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `array/push/push_no_args_bytes.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `calldata/calldata_internal_function_pointer.sol` — rejected: number of 4 bytes cannot be converted to type 'bytes1'
- `cleanup/cleanup_bytes_types_v1.sol` — rejected: conversion to bytes3 from uint32 not allowed
- `cleanup/cleanup_bytes_types_v2.sol` — rejected: conversion to bytes3 from uint32 not allowed
- `events/event_indexed_string.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `getters/value_types.sol` — rejected: number of 4 bytes cannot be converted to type 'bytes1'
- `libraries/internal_library_function_attached_to_fixed_bytes.sol` — rejected: conversion to uint32 from bytes2 not allowed
- `operators/userDefined/all_possible_user_defined_value_types_with_operators.sol` — rejected: conversion to bytes20 from address not allowed; conversion to bytes20 from address not allowed; conversion to bytes20 from address not allowed; conversion to bytes20 from address n…
- `types/convert_fixed_bytes_to_uint_same_min_size.sol` — rejected: conversion to uint32 from bytes1 not allowed
- `types/convert_uint_to_fixed_bytes_same_min_size.sol` — rejected: conversion to bytes1 from uint32 not allowed
- `types/convert_uint_to_fixed_bytes_smaller_size.sol` — rejected: conversion to bytes2 from uint32 not allowed
- `viaYul/storage/packed_storage.sol` — rejected: conversion to bytes1 from uint32 not allowed

</details>

<a id="int-width-overflow"></a>
### Small integer types are computed at a wider width

**Category:** documented-difference · **Files:** 23 · **Rule:** `int-width-overflow`

**What it means.** On Soroban, Solang rounds integer types up to 32/64/128/256 bits and warns about it. Arithmetic, overflow checks, shifts and truncation therefore follow the wider type: `uint8` 200 + 100 returns 300 instead of reverting. This is documented, but the warning does not mention that overflow protection changes.

**Suggested fix.** Keep the declared width inside the contract (range-check inputs, check overflow at the declared width) or state the overflow consequence in the warning.

**Docs.** docs/targets/soroban_language_compatibility.rst, 'Integer Widths'.

<details><summary>Files</summary>

- `arithmetics/checked_add_v1.sol` — wrong value `f(uint16,uint16)`: expected [Int(0)], got Int(65536)
- `arithmetics/checked_add_v2.sol` — should have reverted `f(uint16,uint16)`: returned Int(65536) instead of reverting
- `arithmetics/checked_called_by_unchecked.sol` — should have reverted `f(uint16,uint16,uint16)`: returned Int(115970) instead of reverting
- `arithmetics/unchecked_called_by_checked.sol` — wrong value `f(uint16)`: expected [Int(511)], got Int(66047)
- `operators/shifts/shift_left_uint8.sol` — wrong value `f(uint8,uint8)`: expected [Int(0)], got Int(26112)
- `operators/shifts/shift_overflow.sol` — wrong value `leftU(uint8,uint8)`: expected [Int(0)], got Int(65280)
- `operators/shifts/shift_right_garbled_signed_v1.sol` — wrong value `g(int8,uint8)`: expected [Int(115792089237316195423570985008687907853269984665640564039457584007913129639934)], got Int(30)
- `operators/shifts/shift_right_garbled_signed_v2.sol` — should have reverted `f(int8,uint8)`: returned Int(115792089237316195423570985008687907853269984665640564039457584007913129639934) instead of reverting
- `operators/shifts/shift_right_garbled_v1.sol` — wrong value `f(uint8,uint8)`: expected [Int(15)], got Int(268435455)
- `operators/shifts/shift_right_garbled_v2.sol` — wrong value `f(uint8,uint8)`: expected [Int(15)], got Int(268435455)
- `operators/shifts/shift_right_negative_lvalue_signextend_int16_v1.sol` — wrong value `f(int16,uint16)`: expected [Int(115792089237316195423570985008687907853269984665640564039457584007913129639833)], got Int(65433)
- `operators/shifts/shift_right_negative_lvalue_signextend_int16_v2.sol` — should have reverted `f(int16,uint16)`: returned Int(65433) instead of reverting
- `operators/shifts/shift_right_negative_lvalue_signextend_int8_v1.sol` — wrong value `f(int8,uint8)`: expected [Int(115792089237316195423570985008687907853269984665640564039457584007913129639833)], got Int(153)
- `operators/shifts/shift_right_negative_lvalue_signextend_int8_v2.sol` — should have reverted `f(int8,uint8)`: returned Int(153) instead of reverting
- `types/mapping/copy_struct_to_array_stored_in_mapping.sol` — failed at runtime `from_calldata_to_static_array((uint8))`: contract deployment failed: called `Result::unwrap()` on an `Err` value: HostError: Error(Context, InvalidAction)
- `viaYul/detect_add_overflow.sol` — should have reverted `g(uint8,uint8)`: returned Int(256) instead of reverting
- `viaYul/detect_add_overflow_signed.sol` — should have reverted `g(int8,int8)`: returned Int(128) instead of reverting
- `viaYul/detect_div_overflow.sol` — should have reverted `g(int8,int8)`: returned Int(128) instead of reverting
- `viaYul/detect_mul_overflow.sol` — should have reverted `g(uint8,uint8)`: returned Int(256) instead of reverting
- `viaYul/detect_mul_overflow_signed.sol` — should have reverted `g(int8,int8)`: returned Int(128) instead of reverting
- `viaYul/detect_sub_overflow_signed.sol` — should have reverted `g(int8,int8)`: returned Int(128) instead of reverting
- `viaYul/dirty_calldata_struct.sol` — wrong value `f((uint16[]))`: expected [Bool(true)], got Bool(false)
- `viaYul/exp_overflow.sol` — should have reverted `f(uint8,uint8)`: returned Int(256) instead of reverting

</details>

<a id="abi-encode-handles"></a>
### `abi.encode` returns Soroban handles, not ABI bytes

**Category:** documented-difference · **Files:** 7 · **Rule:** `abi-encode-handles`

**What it means.** On Soroban, Solang's `abi.encode` produces 8 bytes per value: the value's live host-object handle, not the EVM ABI encoding. Solang documents that these bytes are only valid inside the same invocation. A test that returns or compares the encoded bytes therefore cannot match the EVM result.

**Suggested fix.** If `abi.encode` output must leave the contract, encode real bytes (e.g. with the host's `serialize_to_bytes`) or reject returning it.

**Docs.** docs/targets/soroban_examples_coverage.rst, 'eth_abi' (handles valid only within one invocation).

<details><summary>Files</summary>

- `abiEncoderV1/memory_dynamic_array_and_calldata_bytes.sol` — wrong value `f(uint256[],bytes)`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `abiEncoderV2/calldata_nested_array_reencode.sol` — wrong value `f(uint256[][])`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `abiEncoderV2/calldata_overlapped_nested_dynamic_arrays.sol` — wrong value `f_encode(uint256[][])`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `abiEncoderV2/calldata_struct_array_reencode.sol` — wrong value `f((uint256[]))`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `abiEncoderV2/memory_dynamic_array_and_calldata_bytes.sol` — wrong value `f(uint256[],bytes)`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `abiEncoderV2/storage_array_encoding.sol` — wrong value `h(uint256[2][])`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…
- `calldata/calldata_bytes_to_memory_encode.sol` — wrong value `f(bytes)`: expected [Bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0…

</details>

<a id="abi-decode-external-bytes"></a>
### `abi.decode` of bytes from outside the contract

**Category:** documented-difference · **Files:** 5 · **Rule:** `abi-decode-external-bytes`

**What it means.** The test passes EVM ABI-encoded bytes into the contract and decodes them with `abi.decode`. On Soroban, `abi.decode` expects host-object handles produced in the same invocation, so the host rejects the input (`Error(Value, InvalidInput)`).

**Suggested fix.** Same as `abi.encode`: document clearly, or support real byte encodings.

**Docs.** docs/targets/soroban_examples_coverage.rst, 'eth_abi'.

<details><summary>Files</summary>

- `abiEncoderV1/abi_decode_dynamic_array.sol` — failed at runtime `f(bytes)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `abiEncoderV1/abi_decode_static_array.sol` — failed at runtime `f(bytes)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `abiEncoderV1/abi_decode_static_array_v2.sol` — failed at runtime `f(bytes)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `abiEncoderV1/abi_decode_trivial.sol` — failed at runtime `f(bytes)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `abiEncoderV1/abi_decode_v2_calldata.sol` — failed at runtime `f(bytes)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)

</details>

<a id="evm-only-also-crashed"></a>
### EVM-only test that also crashes the compiler

**Category:** evm-only · **Files:** 155 · **Rule:** `evm-only-also-crashed`

**What it means.** The test uses an EVM-only feature (usually inline assembly), so it is excluded from the pass rate. Solang crashed on it instead of rejecting it cleanly, which is a minor diagnostic bug.

**Suggested fix.** Reject inline assembly for Soroban with a diagnostic (e.g. `sema/yul/builtin.rs` currently panics with `not implemented`).

**Docs.** docs/targets/soroban_support_matrix.rst, 'Yul and inline assembly': Unsupported.

<details><summary>Files</summary>

- `abiEncoderV1/dynamic_memory_copy.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV1/return_dynamic_types_cross_call_out_of_range_1.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV1/return_dynamic_types_cross_call_out_of_range_2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/address.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, i…
- `abiEncoderV2/cleanup/bool.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, i…
- `abiEncoderV2/cleanup/bytesx.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, i…
- `abiEncoderV2/cleanup/dynamic_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/intx.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, i…
- `abiEncoderV2/cleanup/reencoded_calldata_string.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/simple_struct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/static_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `abiEncoderV2/cleanup/uintx.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): Found IntValue(IntValue { int_value: Value { name: "", address: 0x…, i…
- `abiEncoderV2/struct/struct_validation.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/array_storage_index_zeroed_test.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/byte_array_storage_layout.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/bytes_to_fixed_bytes_cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/array_copy_cleanup_uint128.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/array_copy_cleanup_uint40.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/array_copy_clear_storage.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/array_copy_clear_storage_packed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/copy_byte_array_to_storage.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/dirty_memory_bytes_to_storage_copy.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/dirty_memory_bytes_to_storage_copy_ir.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/copying/empty_bytes_copy.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/delete/delete_bytes_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/delete/delete_storage_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/delete/delete_storage_array_packed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/invalid_encoding_for_storage_byte_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `array/pop/byte_array_pop_long_storage_empty_garbage_ref.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `asmForLoop/for_loop_break.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `asmForLoop/for_loop_continue.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `asmForLoop/for_loop_nested.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `builtinFunctions/ripemd160.sol` — excluded: ripemd160: ripemd160 — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): hash builtins is not supported for target soroban (at solang/src/emi…
- `builtinFunctions/ripemd160_packed.sol` — excluded: ripemd160: ripemd160 — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): hash builtins is not supported for target soroban (at solang/src/emi…
- `byte_array_to_storage_cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `cleanup/indexed_log_topic_during_explicit_downcast.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `constructor/callvalue_check.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/bound_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/library_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/module_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/static_base_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/subassembly_deduplication.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/super_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `deployedCodeExclusion/virtual_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `ecrecover/failing_ecrecover_invalid_input_asm.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `enums/invalid_enum_logged.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimi…
- `externalContracts/snark.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionCall/calling_uninitialized_function_in_detail.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionCall/calling_uninitialized_function_through_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionCall/return_size_bigger_than_expected.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionCall/return_size_shorter_than_expected.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionCall/return_size_shorter_than_expected_evm_version_after_homestead.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionTypes/comparison_operator_for_external_function_cleans_dirty_bits.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `functionTypes/inline_array_with_value_call_option.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/sema/mutab…
- `immutable/immutable_signed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'function() internal returns (uint256)' is not supported by the S…
- `inlineAssembly/basefee_berlin_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/calldata_array_assign_static.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code: Cannot assign to this expres…
- `inlineAssembly/calldata_struct_assign.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code: Cannot assign to this expres…
- `inlineAssembly/chainid.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/difficulty.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_embedded_function_call.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_for.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_for2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_function_call.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_function_call2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_function_call_assignment.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_if.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_in_modifiers.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/target…
- `inlineAssembly/inline_assembly_read_and_write_stack.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_recursion.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_storage_access.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_storage_access_local_var.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/inline_assembly_storage_access_via_pointer.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak256_assembly.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak256_optimization.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak256_optimizer_bug_different_memory_location.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak256_optimizer_cache_bug.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak_optimization_bug_string.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/keccak_yul_optimization.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/optimize_memory_store_multi_block.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/optimize_memory_store_multi_block_bugreport.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/prevrandao.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/selfbalance.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `inlineAssembly/shadowing_local_function_opcode.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `memoryManagement/return_variable.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `memoryManagement/static_memory_array_allocation.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `memoryManagement/struct_allocation.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `modifiers/function_modifier.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept; also crashed: solang panicked mid-compile (ICE): range 0..512 out of bounds: 256 (at bitvec-1.1.1/src/slice/api.rs:…
- `operators/shifts/bitwise_shifting_constantinople.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/shifts/bitwise_shifting_constantinople_combined.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/shifts/bitwise_shifting_constants_constantinople.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/shifts/shifts.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/userDefined/operator_making_pure_external_call.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/userDefined/operator_making_view_external_call.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/userDefined/operator_parameter_and_return_cleanup_between_calls.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `operators/userDefined/operator_parameter_cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `optimizer/shift_bytes.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `revertStrings/invalid_abi_decoding_memory_v1.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `reverts/invalid_enum_as_external_arg.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimi…
- `reverts/invalid_enum_as_external_ret.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimi…
- `reverts/invalid_enum_compared.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimi…
- `reverts/invalid_enum_stored.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): internal error: entered unreachable code (at solang/src/codegen/optimi…
- `reverts/invalid_instruction.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `reverts/revert.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `reverts/revert_return_area.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `shanghai/evmone_support.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `state/block_basefee.sol` — excluded: block.basefee: block.basefee — EVM-only builtin, no Soroban primitive; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `structs/recursive_struct_2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `structs/struct_delete_storage_nested_small.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `structs/struct_delete_storage_small.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `structs/struct_delete_storage_with_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `structs/struct_delete_storage_with_arrays_small.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/invalid_error_encoding.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/malformed_error.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/malformed_panic.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/malformed_panic_2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/malformed_panic_3.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `tryCatch/malformed_panic_4.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `uninitializedFunctionPointer/uninitialized_internal_storage_function_legacy.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban decoder for…
- `uninitializedFunctionPointer/uninitialized_internal_storage_function_via_yul.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'function() internal' is not supported by the Soroban decoder for…
- `unused_store_storage_removal_bug.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `userDefinedValueType/assembly_access_bytes2_abicoder_v1.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'usertype MyBytes2' is not supported by the Soroban decoder for t…
- `userDefinedValueType/assembly_access_bytes2_abicoder_v2.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'usertype MyBytes2' is not supported by the Soroban decoder for t…
- `userDefinedValueType/dirty_slot.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `userDefinedValueType/dirty_uint8_read.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `userDefinedValueType/immutable_signed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'usertype MyInt' is not supported by the Soroban decoder for targ…
- `userDefinedValueType/parameter.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): type 'usertype MyAddress' is not supported by the Soroban decoder for …
- `userDefinedValueType/storage_layout_struct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/address_code_complex.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/byte_optimization_bug.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/code_access_content.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/code_access_create.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/code_access_padding.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/code_access_runtime.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/code_length.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/codebalance_assembly.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/codehash_assembly.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/create_random.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `various/iszero_bnot_correct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/cleanup/checked_arithmetic.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/dirty_memory_dynamic_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/dirty_memory_int32.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/dirty_memory_static_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/dirty_memory_struct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/dirty_memory_uint32.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/empty_return_corrupted_free_memory_pointer.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/memory_struct_allow.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/msg_sender.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/storage/dirty_storage_bytes.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/storage/dirty_storage_bytes_long.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/storage/dirty_storage_dynamic_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/storage/dirty_storage_static_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/storage/dirty_storage_struct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)
- `viaYul/various_inline_asm.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also crashed: solang panicked mid-compile (ICE): not implemented (at solang/src/sema/yul/builtin.rs:25:32)

</details>

<a id="evm-only"></a>
### EVM-only feature

**Category:** evm-only · **Files:** 86 · **Rule:** `evm-only`

**What it means.** The test uses a feature that exists only on the EVM (assembly, `selfdestruct`, `ecrecover`, `tx.origin`, `msg.value`, low-level calls, EVM block fields). It is excluded from the pass rate.

**Suggested fix.** None needed.

**Docs.** docs/targets/soroban_support_matrix.rst.

<details><summary>Files</summary>

- `abiEncoderV1/cleanup/cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `abiEncoderV2/cleanup/cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `abiencodedecode/abi_encode_call_declaration.sol` — excluded: .staticcall: .staticcall — EVM-only construct, no Soroban mapping
- `arithmetics/check_var_init.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `array/delete/delete_memory_array.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also killed by signal 6: sorobench: llvm/lib/IR/Value.cpp:502: void llvm::Value::doRAUW(llvm::Value*, llvm::Value::Repl…
- `builtinFunctions/blockhash.sol` — excluded: blockhash: blockhash — EVM-only builtin, no Soroban primitive
- `builtinFunctions/blockhash_shadow_resolution.sol` — excluded: blockhash: blockhash — EVM-only builtin, no Soroban primitive
- `calldata/calldata_struct_cleaning.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `cleanup/cleanup_address_types_shortening.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `cleanup/indexed_log_topic_during_explicit_downcast_during_emissions.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `constants/asm_address_constant_regression.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `constants/asm_constant_file_level.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `ecrecover/ecrecover.sol` — excluded: ecrecover: ecrecover — EVM-only builtin, no Soroban primitive
- `ecrecover/ecrecover_abiV2.sol` — excluded: ecrecover: ecrecover — EVM-only builtin, no Soroban primitive
- `ecrecover/failing_ecrecover_invalid_input.sol` — excluded: ecrecover: ecrecover — EVM-only builtin, no Soroban primitive
- `ecrecover/failing_ecrecover_invalid_input_proper.sol` — excluded: ecrecover: ecrecover — EVM-only builtin, no Soroban primitive
- `events/event.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_anonymous_with_signature_collision.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_anonymous_with_signature_collision2.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_anonymous_with_topics.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_emit.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_emit_file_level.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_emit_from_other_contract.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `events/event_lots_of_data.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `exponentiation/signed_base.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `exponentiation/small_exp.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also killed by signal 6: sorobench: llvm/lib/Support/APInt.cpp:2106: void llvm::APInt::fromString(unsigned int, llvm::S…
- `expressions/bit_operators.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `externalContracts/FixedFeeRegistrar.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `externalContracts/deposit_contract.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `functionCall/delegatecall_return_value.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `functionCall/precompile_extcodesize_check.sol` — excluded: .staticcall: .staticcall — EVM-only construct, no Soroban mapping
- `functionCall/value_test.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `inlineAssembly/calldata_array_assign_dynamic.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_array_read.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_assign.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_assign_from_nowhere.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_length_read.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_offset_read.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_offset_read_write.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/calldata_struct_assign_and_return.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/constant_access.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/constant_access_referencing.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/inline_assembly_memory_access.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/inline_assembly_storage_access_inside_function.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/inline_assembly_write_to_stack.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/inlineasm_empty_let.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/slot_access.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/slot_access_via_mapping_pointer.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `inlineAssembly/truefalse.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `libraries/library_address.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `libraries/library_address_homestead.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `libraries/library_address_via_module.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `libraries/library_delegatecall_guard_pure.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `libraries/library_delegatecall_guard_view_needed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `libraries/library_delegatecall_guard_view_not_needed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `libraries/library_delegatecall_guard_view_staticcall.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `libraries/library_function_selectors.sol` — excluded: .delegatecall: .delegatecall — EVM-only construct, no Soroban mapping
- `libraries/library_function_selectors_struct.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `libraries/library_return_struct_with_mapping.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also killed by signal 6: sorobench: llvm/lib/IR/Value.cpp:505: void llvm::Value::doRAUW(llvm::Value*, llvm::Value::Repl…
- `operators/shifts/shift_bytes_cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also killed by signal 6: sorobench: llvm/lib/Support/APInt.cpp:2106: void llvm::APInt::fromString(unsigned int, llvm::S…
- `operators/shifts/shift_bytes_cleanup_viaYul.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping; also killed by signal 6: sorobench: llvm/lib/Support/APInt.cpp:2106: void llvm::APInt::fromString(unsigned int, llvm::S…
- `operators/userDefined/operator_return_parameter_cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `payable/no_nonpayable_circumvention_by_modifier.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `smoke/basic.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `smoke/fallback.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `state/block_chainid.sol` — excluded: block.chainid: block.chainid — EVM-only builtin, no Soroban primitive
- `state/block_coinbase.sol` — excluded: block.coinbase: block.coinbase — EVM-only builtin, no Soroban primitive
- `state/block_difficulty.sol` — excluded: block.difficulty: block.difficulty — EVM-only builtin, no Soroban primitive
- `state/block_difficulty_post_paris.sol` — excluded: block.difficulty: block.difficulty — EVM-only builtin, no Soroban primitive
- `state/block_gaslimit.sol` — excluded: block.gaslimit: block.gaslimit — EVM-only builtin, no Soroban primitive
- `state/block_prevrandao.sol` — excluded: block.prevrandao: block.prevrandao — EVM-only builtin, no Soroban primitive
- `state/block_prevrandao_pre_paris.sol` — excluded: block.prevrandao: block.prevrandao — EVM-only builtin, no Soroban primitive
- `state/blockhash_basic.sol` — excluded: blockhash: blockhash — EVM-only builtin, no Soroban primitive
- `state/gasleft.sol` — excluded: gasleft: gasleft — EVM-only builtin, no Soroban primitive
- `state/msg_value.sol` — excluded: msg.value: msg.value — offered by solang but no Soroban concept
- `state/tx_gasprice.sol` — excluded: tx.gasprice: tx.gasprice — offered by solang but no Soroban concept
- `state/tx_origin.sol` — excluded: tx.origin: tx.origin — EVM-only builtin, no Soroban primitive
- `state/uncalled_blockhash.sol` — excluded: blockhash: blockhash — EVM-only builtin, no Soroban primitive
- `userDefinedValueType/cleanup.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `userDefinedValueType/cleanup_abicoderv1.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `userDefinedValueType/storage_layout.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `userDefinedValueType/storage_signed.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `various/gasleft_decrease.sol` — excluded: gasleft: gasleft — EVM-only builtin, no Soroban primitive
- `various/selfdestruct.sol` — excluded: selfdestruct: selfdestruct — EVM-only builtin, no Soroban primitive
- `viaYul/storage/mappings.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping
- `viaYul/unary_operations.sol` — excluded: assembly: assembly — EVM-only construct, no Soroban mapping

</details>

<a id="evm-calldata-shape"></a>
### Malformed or raw EVM calldata

**Category:** evm-only · **Files:** 9 · **Rule:** `evm-calldata-shape`

**What it means.** These tests send deliberately malformed EVM calldata (a `bool` with value 2, wrong array lengths, dirty padding) or inspect EVM bytecode, and expect the EVM decoder to reject it. Soroban passes typed host values, so the malformed input cannot be expressed.

**Suggested fix.** sorobench: mark these as NoFaithful instead of failures.

**Docs.** Not applicable on Soroban.

<details><summary>Files</summary>

- `abiEncoderV1/calldata_arrays_too_large.sol` — should have reverted `f(uint256,uint256[],uint256)`: returned Int(7) instead of reverting
- `abiEncoderV2/bool_out_of_bounds.sol` — should have reverted `f(bool)`: returned Bool(true) instead of reverting
- `abiEncoderV2/calldata_nested_array_static_reencode.sol` — should have reverted `f(uint256[3][])`: returned (void) instead of reverting
- `cleanup/bool_conversion_v2.sol` — should have reverted `f(bool)`: returned Int(1) instead of reverting
- `dirty_calldata_bytes.sol` — wrong value `f(bytes)`: expected [Bool(true)], got Bool(false)
- `freeFunctions/free_runtimecode.sol` — wrong value `f()`: expected [Bool(true)], got Bool(false)
- `operators/shifts/shift_right_negative_lvalue_signextend_int32_v2.sol` — should have reverted `f(int32,uint32)`: returned Int(115792089237316195423570985008687907853269984665640564039457584007913129639833) instead of reverting
- `revertStrings/calldata_array_invalid_length.sol` — should have reverted `f(uint256[][])`: returned Int(0) instead of reverting
- `revertStrings/calldata_arrays_too_large.sol` — should have reverted `f(uint256,uint256[],uint256)`: returned Int(7) instead of reverting

</details>

<a id="evm-selector-concepts"></a>
### Function selectors (`msg.sig`, `interfaceId`)

**Category:** evm-only · **Files:** 4 · **Rule:** `evm-selector-concepts`

**What it means.** These values are EVM function selectors: the first 4 bytes of the Keccak hash of the function signature. Soroban calls functions by name, so there are no selectors and the values cannot match.

**Suggested fix.** sorobench: filter `msg.sig`, `msg.data` and `interfaceId` as EVM-only.

**Docs.** Not applicable on Soroban.

<details><summary>Files</summary>

- `builtinFunctions/msg_sig.sol` — wrong value `foo(uint256)`: expected [Bytes([47, 190, 189, 56])], got Bytes([0, 0, 0, 0])
- `builtinFunctions/msg_sig_after_internal_call_is_same.sol` — wrong value `foo(uint256)`: expected [Bytes([47, 190, 189, 56])], got Bytes([0, 0, 0, 0])
- `interfaceID/interfaces.sol` — wrong value `hello()`: expected [Bytes([25, 255, 29, 33])], got Bytes([133, 41, 88, 119])
- `state/msg_sig.sol` — wrong value `f()`: expected [Bytes([38, 18, 31, 240])], got Bytes([0, 0, 0, 0])

</details>

<a id="eth-address-literal"></a>
### Ethereum address literals

**Category:** evm-only · **Files:** 3 · **Rule:** `eth-address-literal`

**What it means.** The test contains a 20-byte Ethereum address literal (`0x1234…`). Soroban addresses are a different, 32-byte strkey format, so an Ethereum address has no meaning there.

**Suggested fix.** None needed; Soroban addresses come from the host (e.g. `msg.sender` equivalents or arguments).

**Docs.** docs/targets/soroban_rust_sdk_differences.rst (host-value representation).

<details><summary>Files</summary>

- `cleanup/cleanup_address_types_v1.sol` — rejected: ethereum address literal '0x1234567890123456789012345678901234567890' not supported on target Soroban; ethereum address literal '0x1234567890123456789012345678901234567890' not sup…
- `cleanup/cleanup_address_types_v2.sol` — rejected: ethereum address literal '0x1234567890123456789012345678901234567890' not supported on target Soroban; ethereum address literal '0x1234567890123456789012345678901234567890' not sup…
- `operators/userDefined/consecutive_operator_invocations.sol` — rejected: ethereum address literal '0x3333333333333333333333333333333333333333' not supported on target Soroban

</details>

<a id="evm-fallback-receive"></a>
### Fallback and receive functions

**Category:** evm-only · **Files:** 3 · **Rule:** `evm-fallback-receive`

**What it means.** These tests call a contract with empty or unknown calldata to trigger `fallback()`/`receive()`. Soroban invokes functions by name and has no fallback dispatch, so the fallback never runs.

**Suggested fix.** sorobench: filter fallback/receive-triggering calls.

**Docs.** Not applicable on Soroban.

<details><summary>Files</summary>

- `fallback/falback_return.sol` — wrong value `x()`: expected [Int(1)], got Int(0)
- `fallback/short_data_calls_fallback.sol` — wrong value `x()`: expected [Int(2)], got Int(0)
- `receive/empty_calldata_calls_receive.sol` — wrong value `x()`: expected [Int(1)], got Int(0)

</details>

<a id="address-code"></a>
### Reading contract bytecode (`address.code`)

**Category:** evm-only · **Files:** 2 · **Rule:** `address-code`

**What it means.** The test reads another contract's EVM bytecode. Soroban contracts are Wasm blobs referenced by hash; there is no equivalent of reading code by address.

**Suggested fix.** None needed.

**Docs.** Not applicable on Soroban.

<details><summary>Files</summary>

- `various/address_code.sol` — rejected: 'address.code' is not supported on Soroban; 'address.code' is not supported on Soroban; 'address.code' is not supported on Soroban; 'address.code' is not supported on Soroban
- `various/code_length_contract_member.sol` — rejected: 'address.code' is not supported on Soroban

</details>

<a id="external-source-files"></a>
### Tests that import files from outside the test

**Category:** tool · **Files:** 15 · **Rule:** `external-source-files`

**What it means.** The test imports a file that is not part of the test itself (solc's `==== ExternalSource: ====` helper libraries stored next to the suite). sorobench does not load external sources yet, so Solang cannot find the import. This says nothing about Solang.

**Suggested fix.** sorobench: load `ExternalSource` files relative to the test directory.

**Docs.** sorobench limitation.

<details><summary>Files</summary>

- `externalContracts/base64.sol` — rejected: file not found '_base64/base64_inline_asm.sol'; file not found '_base64/base64_no_inline_asm.sol'; 'InlineAsmBase64' not found; 'NoAsmBase64' not found; 'InlineAsmBase64' not found…
- `externalContracts/prbmath_signed.sol` — rejected: file not found '_prbmath/PRBMathSD59x18.sol'; 'PRBMathSD59x18' not found; method 'div' does not exist; method 'exp' does not exist; method 'exp2' does not exist; method 'gm' does n…
- `externalContracts/prbmath_unsigned.sol` — rejected: file not found '_prbmath/PRBMathUD60x18.sol'; 'PRBMathUD60x18' not found; method 'div' does not exist; method 'exp' does not exist; method 'exp2' does not exist; method 'gm' does n…
- `externalContracts/ramanujan_pi.sol` — rejected: file not found '_prbmath/PRBMathSD59x18.sol'; 'PRBMathSD59x18' not found; method 'pow' does not exist
- `externalContracts/strings.sol` — rejected: file not found '_stringutils/stringutils.sol'; 'strings' not found; 'strings' not found; 'strings' not found; 'strings' not found; method 'toSlice' does not exist; method 'toSlice'…
- `externalSource/multiple_equals_signs.sol` — rejected: file not found 'a'
- `externalSource/multiple_external_source.sol` — rejected: file not found '_external/external.sol'; file not found '_external/other_external.sol'
- `externalSource/multisource.sol` — rejected: file not found '_external/external.sol'
- `externalSource/non_normalized_paths.sol` — rejected: file not found '_non_normalized_paths//a.sol'; file not found 'C/////c.sol'; file not found 'C/../////D/d.sol'
- `externalSource/relative_imports.sol` — rejected: file not found '_relative_imports/dir/contract.sol'
- `externalSource/source.sol` — rejected: file not found '_external/external.sol'
- `externalSource/source_import.sol` — rejected: file not found '_external/external.sol'; file not found '_external/other_external.sol'
- `externalSource/source_import_subdir.sol` — rejected: file not found 'sub_external.sol'
- `externalSource/source_name_starting_with_dots.sol` — rejected: file not found '_source_name_starting_with_dots/dir/contract.sol'
- `externalSource/source_remapping.sol` — rejected: file not found 'ExtSource.sol'; file not found '/ExtSource.sol'; type 'External' not found; type 'OtherExternal' not found

</details>

<a id="test-env-block"></a>
### Block number / timestamp start at 0 in sorobench

**Category:** tool · **Files:** 2 · **Rule:** `test-env-block`

**What it means.** solc's test framework advances the block number and timestamp with every transaction; sorobench's local Soroban environment keeps them at 0. `block.number` and `block.timestamp` are supported by Solang, so this is a test-environment difference.

**Suggested fix.** sorobench: advance the ledger sequence and timestamp between calls like solc's framework.

**Docs.** docs/targets/soroban_support_matrix.rst, 'Soroban utilities': block.timestamp and block.number supported.

<details><summary>Files</summary>

- `state/block_number.sol` — wrong value `f()`: expected [Int(2)], got Int(0)
- `state/block_timestamp.sol` — wrong value `f()`: expected [Int(30)], got Int(0)

</details>

<a id="tool-unsupported"></a>
### Not runnable by sorobench

**Category:** tool · **Files:** 1 · **Rule:** `tool-unsupported`

**What it means.** sorobench cannot run this test yet (e.g. constructor arguments of an unsupported type).

**Suggested fix.** sorobench: extend argument mapping.

**Docs.** sorobench limitation.

<details><summary>Files</summary>

- `array/fixed_arrays_in_constructors.sol` — not runnable: constructor args: address argument (NoFaithful)

</details>

## Needs review — 17 file(s)

No dictionary rule explains these failures yet. Check each against Solang's Soroban docs; if it is documented or explainable, add a rule to `dictionary/soroban.toml`, otherwise it is a candidate bug.

- `abiEncoderV1/abi_encode_call.sol` — failed at runtime `f()`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput)
- `array/memory_arrays_of_various_sizes.sol` — failed at runtime `f(uint256,uint256)`: Error(Context, InvalidAction); log: runtime_error: array index out of bounds in test.sol:6:13-20 (+1 more) [warning: conversion truncates uint256 to uint32, as memory size is type uint32 on target Soroban]
- `array/push/push_no_args_struct.sol` — failed at runtime `a(uint256)`: Error(Context, InvalidAction); log: VM call trapped with HostError, a, Error(Value, InvalidInput) (+2 more)
- `array/string_bytes_conversion.sol` — failed at runtime `f(string,uint256)`: Error(Context, InvalidAction); log: VM call trapped with HostError, f, Error(Value, InvalidInput) (+1 more)
- `cleanup/cleanup_bytes_types_shortening_OldCodeGen.sol` — failed at runtime `f()`: Error(Context, InvalidAction); log: runtime_error: require condition failed in test.sol:10:9-16 [warning: local variable 'x' is unused]
- `constructor/base_constructor_arguments.sol` — wrong value `getA()`: expected [Int(49)], got Int(0)
- `constructor_inheritance_init_order.sol` — wrong value `y()`: expected [Int(42)], got Int(0)
- `events/event_really_really_lots_of_data_from_storage.sol` — failed at runtime `deposit()`: Error(Context, InvalidAction); log: VM call trapped with HostError, deposit, Error(Object, IndexBounds)
- `freeFunctions/free_namesake_contract_function.sol` — should have reverted `f()`: returned Int(0) instead of reverting [warning: f is already defined as a function]
- `functionCall/delegatecall_return_value_pre_byzantium.sol` — failed at runtime `assert0_delegated()`: Error(Context, InvalidAction); log: VM call trapped with HostError, assert0_delegated, Error(Value, InvalidInput) (+5 more)
- `functionCall/inheritance/super_skip_unimplemented_in_interface.sol` — wrong value `f()`: expected [Int(42)], got Int(0)
- `multiSource/free_function_resolution_override_virtual.sol` — wrong value `g()`: expected [Int(1337)], got Int(1338)
- `multiSource/free_function_resolution_override_virtual_transitive.sol` — wrong value `g()`: expected [Int(1339)], got Int(1337)
- `state_variables_init_order_3.sol` — wrong value `b_c()`: expected [Int(51)], got Int(126) (+1 more)
- `structs/delete_struct.sol` — failed at runtime `getTopValue()`: Error(Context, InvalidAction); log: VM call trapped with HostError, getTopValue, Error(Value, InvalidInput) (+5 more)
- `various/typed_multi_variable_declaration.sol` — wrong value `f()`: expected [Bool(true)], got Bool(false)
- `viaYul/mapping_getters.sol` — wrong value `m2(uint256,uint256)`: expected [Int(35)], got Int(0) (+2 more)

