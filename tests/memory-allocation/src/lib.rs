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
pub extern fn init(penv: &mut PluginEnv) {
    malloc_plugin_data(penv, PluginData{
        val1: 2,
        val2: 3, /* As 0 is already reserved */
    });
}

// This function determines if there are plugin frames that must be
// sent now or not.
// A bool MUST be a i32...
#[no_mangle]
pub extern fn check_data(penv: &mut PluginEnv) -> i32 {
    let pd = get_plugin_data(penv).unwrap();
    // TODO let have a better API than just plain numerics
    // Note: printing slows down the stack...
    // let txt = format!("pkt_type is {}, is_closing is {} and in_flight is {}", pkt_type, is_closing, pd.in_flight);
    // print(&txt);
    (pd.val1 * pd.val2) as i32
}