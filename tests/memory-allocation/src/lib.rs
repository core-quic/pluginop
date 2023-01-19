use std::format;

use pluginop_wasm::{
    PluginEnv,
};

#[derive(Debug)]
struct PluginData {
    val1: u64,
    val2: u64,
}

const PLUGIN_DATA_TAG: u64 = 0;

///////////////////////////////////////////////////////////////////////

// Various helpers
fn malloc_plugin_data(penv: &mut PluginEnv, p: PluginData) -> u32 {
    let b = Box::new(p);
    // Save it in the opaque store
    let ptr = Box::into_raw(b) as u32;
    penv.store_opaque(PLUGIN_DATA_TAG, ptr);
    ptr
}

fn get_plugin_data(penv: &mut PluginEnv) -> Option<&'static mut PluginData> {
    match penv.get_opaque(PLUGIN_DATA_TAG) {
        Some(ptr) => unsafe { Some(&mut*(ptr as *mut PluginData)) },
        None => None,
    }
}

fn free_plugin_data(penv: &mut PluginEnv) {
    match penv.remove_opaque(PLUGIN_DATA_TAG) {
        Some(ptr) => unsafe { Box::from_raw(ptr as *mut PluginData); },
        None => {},
    }
}

///////////////////////////////////////////////////////////////////////

// Initialize the plugin.
#[no_mangle]
pub extern fn init(penv: &mut PluginEnv) -> i64 {
    malloc_plugin_data(penv, PluginData{
        val1: 2,
        val2: 3,
    });
    0
}

// Checking the content of the memory.
#[no_mangle]
pub extern fn check_data(penv: &mut PluginEnv) -> i64 {
    match get_plugin_data(penv) {
        Some(pd) => {
            let res = (pd.val1 * pd.val2) as i64;
            match penv.save_output(res.into()) {
                Ok(()) => 0,
                Err(_) => -2,
            }
        },
        None => -1,
    }
}

#[no_mangle]
pub extern fn free_data(penv: &mut PluginEnv) -> i64 {
    free_plugin_data(penv);
    0
}