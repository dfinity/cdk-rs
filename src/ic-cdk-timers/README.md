# ic-cdk-timers

A library for Internet Computer canisters to schedule one-shot or repeating timers, to execute a function at some point in the future.

## Example

```rust
ic_cdk_timers::set_timer(Duration::from_secs(1), || ic_cdk::println!("Hello from the future!"));
```
