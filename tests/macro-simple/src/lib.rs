use pluginop_wasm::{quic::ConnectionField, PluginEnv};

// Export a function named "simple_call".
#[no_mangle]
pub extern fn update_rtt(penv: &mut PluginEnv) -> i64 {
    match penv.set_connection(ConnectionField::MaxTxData, 12500 as u64) {
        Ok(()) => 0,
        Err(e) => {
            penv.print(&format!("Got error {e:?}"));
            -1
        }
    }
}
