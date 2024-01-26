use pluginop_wasm::{PluginEnv, PluginVal};

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

#[no_mangle]
pub extern fn plugin_control_3(penv: &mut PluginEnv) -> i64 {
    let add = match penv.poctl(1, &[(10i64).into(), (5i64).into()]) {
        Ok(v) if v.len() == 1 => match v[0] {
            PluginVal::I64(a) => a,
            _ => return -1,
        },
        _ => return -2,
    };
    if add != 15 {
        return -3;
    }
    let sub = match penv.poctl(2, &[(10i64).into(), (5i64).into()]) {
        Ok(v) if v.len() == 1 => match v[0] {
            PluginVal::I64(a) => a,
            _ => return -4,
        },
        _ => return -5,
    };
    if sub != 5 {
        return -6;
    }
    0
}