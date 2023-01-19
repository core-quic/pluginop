use std::sync::Mutex;
use pluginop_wasm::PluginEnv;
use lazy_static::lazy_static;

struct Data {
    val1: i32,
    val2: i32,
}

lazy_static! {
    static ref DATA: Mutex<Data> = Mutex::new(Data {
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
    let mut data = (*DATA).lock().unwrap();
    (*data).val1 = val1;
    (*data).val2 = val2;
    0
}

#[no_mangle]
pub extern fn get_calc_value(penv: &mut PluginEnv) -> i64 {
    let data = (*DATA).lock().unwrap();
    let add = data.val1 + data.val2;
    let sub = data.val1 - data.val2;
    let mul = data.val1 * data.val2;
    let div = data.val1 / data.val2;
    match penv.save_outputs(&[add.into(), sub.into(), mul.into(), div.into()]) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}