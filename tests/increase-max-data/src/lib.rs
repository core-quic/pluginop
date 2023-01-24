use pluginop_wasm::{PluginEnv, quic::{QVal, ConnectionField, Frame}};

#[no_mangle]
pub extern fn process_frame_10(penv: &mut PluginEnv) -> i64 {
    let md_frame = match penv.get_input::<QVal>(0) {
        Ok(QVal::Frame(Frame::MaxData(md))) => md,
        _ => return -1,
    };
    let curr_max: u64 = if let Ok(v) = penv.get_connection(ConnectionField::MaxTxData) {v} else { return -2 };
    if md_frame.maximum_data > curr_max {
        if penv.set_connection(ConnectionField::MaxTxData, md_frame.maximum_data).is_err() {
            return -3;
        }
    }
    0
}
