use pluginop_wasm::PluginEnv;

#[no_mangle]
pub extern fn decode_transport_parameter_aaaaaaaa(penv: &mut PluginEnv) -> i64 {
    // This is an always enabled PO.
    penv.enable();
    0
}

// Export a function named "simple_call".
#[no_mangle]
pub extern fn simple_call(_penv: &mut PluginEnv) -> i64 {
    0
}
