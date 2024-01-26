use pluginop_wasm::{PluginCell, PluginEnv};
use lazy_static::lazy_static;

struct Data {
    val1: i32,
    val2: i32,
}

lazy_static! {
    static ref DATA: PluginCell<Data> = PluginCell::new(Data {
        val1: 0,
        val2: 0,
    });
}

// Export a function named "simple_call".
#[no_mangle]
pub extern fn set_values(_penv: &mut PluginEnv) -> i64 {
    DATA.get_mut().val1 = 12;
    DATA.get_mut().val2 = 3;
    0
}

#[no_mangle]
pub extern fn get_mult_value(_penv: &mut PluginEnv) -> i64 {
    let add = DATA.val1 + DATA.val2;
    let sub = DATA.val1 - DATA.val2;
    let mul = DATA.val1 * DATA.val2;
    let div = if DATA.val2 != 0 { DATA.val1 / DATA.val2 } else { 0 };
    (add + sub + mul + div).into()
}