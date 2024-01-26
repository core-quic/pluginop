use std::sync::Mutex;
use pluginop_wasm::{PluginCell, PluginEnv};
use lazy_static::lazy_static;

struct Data {
    val1: i32,
    val2: i32,
}

lazy_static! {
    static ref DATA: PluginCell<Data> = PluginCell::new(Data {
        val1: 0,
        val2: 1,
    });
}

// Export a function named "simple_call".
#[no_mangle]
pub extern fn set_values(penv: &mut PluginEnv) -> i64 {
    let (val1, val2) = if let (Ok(v1), Ok(v2)) = (penv.get_input(0), penv.get_input(1)) {
        (v1, v2)
    } else {
        return -1;
    };
    DATA.get_mut().val1 = val1;
    DATA.get_mut().val2 = val2;
    0
}

#[no_mangle]
pub extern fn get_calc_value(penv: &mut PluginEnv) -> i64 {
    let add = DATA.val1 + DATA.val2;
    let sub = DATA.val1 - DATA.val2;
    let mul = DATA.val1 * DATA.val2;
    let div = DATA.val1 / DATA.val2;
    match penv.save_outputs(&[add.into(), sub.into(), mul.into(), div.into()]) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}