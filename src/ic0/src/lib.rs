#[link(wasm_import_module = "ic0")]
extern "C" {
    pub fn msg_arg_data_size() -> i32;
    pub fn msg_arg_data_copy(dst: i32, offset: i32, size: i32) -> ();
}
