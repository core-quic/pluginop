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
        val2: 0,
    });
}

// Export a function named "simple_call".
#[no_mangle]
pub extern fn set_values(penv: &mut PluginEnv) -> i64 {
    let inputs = match penv.get_inputs() {
        Ok(i) => i,
        Err(e) => return -1,
    };
    let (val1, val2) = if let (Ok(v1), Ok(v2)) = (inputs[0].try_into(), inputs[1].try_into()) {
        (v1, v2)
    } else {
        return -2;
    };
    let mut data = (*DATA).lock().unwrap();
    (*data).val1 = val1;
    (*data).val2 = val2;
    0
}

#[no_mangle]
pub extern fn get_mult_value(_penv: &mut PluginEnv) -> i64 {
    let data = (*DATA).lock().unwrap();
    ((*data).val1 * (*data).val2) as i64
}