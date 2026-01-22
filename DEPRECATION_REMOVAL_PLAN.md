# Plan: Remove Deprecated Items from ic-cdk

This plan outlines the work to remove all deprecated items introduced in v0.18.0 for the next major release.

## Summary

**Total deprecated items to remove: 44**
- Modules: 3
- Enums: 1
- Structs: 3
- Type aliases: 1
- Functions: 36

## Work Breakdown

### PR 1: Remove deprecated `api::stable` module

**Files to modify:**
- `ic-cdk/src/api.rs` - Remove the `pub mod stable;` declaration and deprecation notice
- `ic-cdk/src/api/stable.rs` - Delete entire file

**Migration:** Users should use `ic_cdk::stable` instead.

---

### PR 2: Remove deprecated `api::management_canister` module

**Files to modify:**
- `ic-cdk/src/api.rs` - Remove the `pub mod management_canister;` declaration and deprecation notice
- `ic-cdk/src/api/management_canister.rs` - Delete entire file (if exists)

**Migration:** Users should use `ic_cdk::management_canister` and `ic_cdk::bitcoin_canister` instead.

---

### PR 3: Remove deprecated functions from `api.rs`

**Items to remove from `ic-cdk/src/api.rs`:**

| Function | Line | Replacement |
|----------|------|-------------|
| `print()` | 650-655 | `debug_print` |
| `caller()` | 658-665 | `msg_caller` |
| `id()` | 668-675 | `canister_self` |
| `canister_balance()` | 685-692 | `canister_cycle_balance` |
| `canister_balance128()` | 695-699 | `canister_cycle_balance` |
| `set_certified_data()` | 719-723 | `certified_data_set` |
| `set_global_timer()` | 737-741 | `global_timer_set` |

**Also update `lib.rs`:**
- Remove re-exports at lines 29-34: `caller`, `id`, `print`

---

### PR 4: Remove deprecated `spawn` function from `lib.rs`

**Files to modify:**
- `ic-cdk/src/lib.rs` - Remove the `spawn` function (lines 71-79)

**Migration:** Users should use `ic_cdk::futures::spawn_017_compat` or migrate to `ic_cdk::futures::spawn`.

---

### PR 5: Remove deprecated `api::call` module (largest PR)

**Files to modify:**
- `ic-cdk/src/api.rs` - Remove the `pub mod call;` declaration and deprecation notice
- `ic-cdk/src/api/call.rs` - Delete entire file
- `ic-cdk/src/lib.rs` - Remove re-exports of `call` and `notify` (lines 29-34)

**Items being removed:**

| Category | Item | Replacement |
|----------|------|-------------|
| Enum | `RejectionCode` | `ic_cdk::call::RejectCode` |
| Type alias | `CallResult<R>` | `ic_cdk::call::CallResult` |
| Struct | `CallReplyWriter` | `ic_cdk::api::msg_reply` |
| Struct | `ArgDecoderConfig` | `candid::de::DecoderConfig` |
| Struct | `ManualReply<T>` | `std::marker::PhantomData` with manual_reply |
| Function | `notify_with_payment128()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `notify()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `notify_raw()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call_raw()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call_raw128()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call_with_payment()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call_with_payment128()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `call_with_config()` | `ic_cdk::call::Call::unbounded_wait()` |
| Function | `result()` | `ic_cdk::api::{msg_reject_code, msg_reject_msg}` |
| Function | `reject_code()` | `ic_cdk::api::msg_reject_code` |
| Function | `reject_message()` | `ic_cdk::api::msg_reject_msg` |
| Function | `reject()` | `ic_cdk::api::msg_reject` |
| Function | `reply()` | `ic_cdk::api::msg_reply` |
| Function | `msg_cycles_available()` | `ic_cdk::api::msg_cycles_available` |
| Function | `msg_cycles_available128()` | `ic_cdk::api::msg_cycles_available` |
| Function | `msg_cycles_refunded()` | `ic_cdk::api::msg_cycles_refunded` |
| Function | `msg_cycles_refunded128()` | `ic_cdk::api::msg_cycles_refunded` |
| Function | `msg_cycles_accept()` | `ic_cdk::api::msg_cycles_accept` |
| Function | `msg_cycles_accept128()` | `ic_cdk::api::msg_cycles_accept` |
| Function | `arg_data_raw()` | `ic_cdk::api::msg_arg_data` |
| Function | `arg_data_raw_size()` | `ic_cdk::api::msg_arg_data` |
| Function | `reply_raw()` | `ic_cdk::api::msg_reply` |
| Function | `arg_data()` | `ic_cdk::call::msg_arg_data` |
| Function | `accept_message()` | `ic_cdk::api::accept_message` |
| Function | `method_name()` | `ic_cdk::api::msg_method_name` |
| Function | `performance_counter()` | `ic_cdk::api::performance_counter` |
| Function | `is_recovering_from_trap()` | `ic_cdk::futures::is_recovering_from_trap` |

---

### PR 6: Update examples and tests

**Files to check and update:**
- All files in `examples/` directory
- All files in `e2e-tests/` directory
- Any internal usage within ic-cdk

---

### PR 7: Update documentation and CHANGELOG

**Files to modify:**
- `ic-cdk/CHANGELOG.md` - Document breaking changes
- `ic-cdk/README.md` - Update if necessary
- Remove or update `V18_GUIDE.md` if it exists

---

## Recommended Order

1. **PR 1-4** can be done in parallel (independent module/function removals)
2. **PR 5** should be done after PR 3 (api::call module depends on some api.rs items)
3. **PR 6** should be done after PR 1-5 (updates examples to use new APIs)
4. **PR 7** should be done last (documentation)

## Notes

- All deprecations are from v0.18.0 except `performance_counter()` in `api/call.rs` which is from v0.11.3
- The `#[allow(deprecated)]` at the top of `api/call.rs` should be removed along with the file
- The `#[doc(hidden)]` attributes on deprecated items indicate they were already hidden from documentation
- Consider running `cargo test` and the full CI pipeline after each PR
