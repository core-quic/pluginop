use pluginop_wasm::PluginEnv;

#[no_mangle]
pub extern fn plugin_control_1(penv: &mut PluginEnv) -> i64 {
    let (val1, val2): (i64, i64) = if let (Ok(v1), Ok(v2)) = (penv.get_input(0), penv.get_input(1)) {
        (v1, v2)
    } else {
        return -1;
    };
    match penv.save_outputs(&[(val1 + val2).into()]) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern fn plugin_control_2(penv: &mut PluginEnv) -> i64 {
    let (val1, val2): (i64, i64) = if let (Ok(v1), Ok(v2)) = (penv.get_input(0), penv.get_input(1)) {
        (v1, v2)
    } else {
        return -1;
    };
    match penv.save_outputs(&[(val1 - val2).into()]) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}