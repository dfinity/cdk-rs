// This file should compile on windows
fn main() {
    unsafe {
        ic0::trap(0, 0);
    }
}
