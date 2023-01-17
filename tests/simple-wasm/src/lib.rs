use pluginop_wasm::PluginEnv;

// Export a function named "simple_call".
#[no_mangle]
pub extern fn simple_call(_penv: &mut PluginEnv) -> i64 {
    42
}
