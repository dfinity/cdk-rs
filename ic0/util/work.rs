// This file should compile on windows
fn main() {
    // SAFETY: ic0.trap is always safe to call with size 0
    unsafe {
        ic0::trap(0, 0);
    }
}
