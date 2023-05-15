use pluginop_wasm::{PluginEnv, Error, quic::{QVal, ConnectionField, Frame}};

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
    let hdr = match penv.get_input::<QVal>(1) {
        Ok(QVal::Header(hdr)) => hdr,
        Err(Error::ShortInternalBuffer) => return 0,
        _ => return -4,
    };
    let _bytes = match penv.get_bytes(hdr.destination_cid.tag, hdr.destination_cid.max_read_len) {
        Ok(dcid) => dcid,
        Err(_) => return -5,
    };
    let bytes = vec![42, 24, 36, 48, 90, 23, 12, 4];
    match penv.put_bytes(hdr.destination_cid.tag, &bytes) {
        Ok(8) => {},
        Ok(_) => return -6,
        Err(_) => return -7,
    };
    let actual_bytes = match penv.get_bytes(hdr.destination_cid.tag, 8) {
        Ok(dcid) => dcid,
        Err(_) => return -8,
    };
    if bytes != actual_bytes {
        return -9;
    }
    0
}
