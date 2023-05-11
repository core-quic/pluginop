use pluginop_wasm::{PluginEnv, PluginCell, Instant, Duration};

use lazy_static::lazy_static;

struct Data {
    success_fired: bool,
    success_cancelled: bool,
    success_time: Instant,
}

lazy_static! {
    static ref DATA: PluginCell<Data> =PluginCell::new(Data {
        success_fired: false,
        success_cancelled: false,
        success_time: Instant::at(0, 0),
    });
}

// Export a function named "simple_call".
#[no_mangle]
pub extern fn launch_timers(penv: &mut PluginEnv) -> i64 {
    // We start two timers. One will fire, the second will be cancelled.
    let now = match penv.get_input::<Instant>(0) {
        Ok(i) => i,
        _ => return -1,
    };
    let timeout_1 = now + Duration::from_millis(20);
    let timeout_2 = now + Duration::from_millis(50);
    let success_time = now + Duration::from_millis(60);
    if penv.set_timer(timeout_1, 1, 1).is_err() {
        return -2;
    }
    if penv.set_timer(timeout_2, 2, 2).is_err() {
        return -3;
    }
    DATA.get_mut().success_fired = false;
    DATA.get_mut().success_cancelled = true;
    DATA.get_mut().success_time = success_time;
    0
}

#[no_mangle]
pub extern fn on_plugin_timeout_1(penv: &mut PluginEnv) -> i64 {
    DATA.get_mut().success_fired = true;
    if penv.cancel_timer(2).is_err() {
        return -1;
    }
    0
}

#[no_mangle]
pub extern fn on_plugin_timeout_2(_: &mut PluginEnv) -> i64 {
    DATA.get_mut().success_cancelled = false;
    -1
}

#[no_mangle]
pub extern fn check_success(penv: &mut PluginEnv) -> i64 {
    // Get the time. If we are too early, return 0 (false). Otherwise,
    // return true. A failed test returns a non-zero error code.
    let now = match penv.get_input::<Instant>(0) {
        Ok(i) => i,
        _ => return -1,
    };
    if now < DATA.success_time {
        return match penv.save_output((false).into()) {
            Ok(()) => 0,
            _ => -1,
        };
    }

    if !DATA.success_fired || !DATA.success_cancelled {
        return -1;
    }

    match penv.save_output((true).into()) {
        Ok(()) => 0,
        _ => -1,
    }
}