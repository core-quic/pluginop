use pluginop_wasm::{PluginEnv, Bytes, quic::{QVal, ConnectionField, Registration, Frame, MaxDataFrame, FrameSendKind, FrameSendOrder, FrameRegistration}};

const MD_FRAME_TYPE: u64 = 0x10;

// Initialize the plugin.
#[no_mangle]
pub extern fn init(penv: &mut PluginEnv) -> i64 {
    match penv.register(Registration::Frame(FrameRegistration::new(MD_FRAME_TYPE, FrameSendOrder::AfterACK, FrameSendKind::OncePerPacket, true, true))) {
        Ok(()) => 0,
        _ => -1,
    }
}

// This function determines if there are plugin frames that must be
// sent now or not.
#[no_mangle]
pub extern fn should_send_frame_10(penv: &mut PluginEnv) -> i64 {
    let out = true;
    match penv.save_output(out.into()) {
        Ok(()) => 0,
        Err(_) => -3,
    }
}

#[no_mangle]
pub extern fn prepare_frame_10(penv: &mut PluginEnv) -> i64 {
    // We need to save the max data frame.
    match penv.save_output(Frame::MaxData(MaxDataFrame { maximum_data: 0x2000 }).into()) {
        Ok(()) => 0,
        _ => -1,
    }
}

#[no_mangle]
pub extern fn write_frame_10(penv: &mut PluginEnv) -> i64 {
    let bytes = match penv.get_input::<Bytes>(1) {
        Ok(b) => b,
        _ => return -3,
    };
    // TODO: check if there is at least 3 bytes.
    let frame_bytes: [u8; 3] = [0x10, 0x60, 0x00];
    match penv.put_bytes(bytes.tag, &frame_bytes) {
        Ok(3) => {},
        _ => return -4,
    };
    match penv.save_output(frame_bytes.len().into()) {
        Ok(()) => 0,
        _ => -5,
    }
}

// Export a function named "log_frame_42".
#[no_mangle]
pub extern fn log_frame_10(_penv: &mut PluginEnv) -> i64 {
    // Do nothing.
    0
}

// Export a function named "parse_frame_42". This can then be called
// from the plugin crate!
#[no_mangle]
pub extern fn parse_frame_10(penv: &mut PluginEnv) -> i64 {
    let bytes = match penv.get_input::<Bytes>(0) {
        Ok(b) => b,
        _ => return -1,
    };

    // Get the data, only one byte is actually needed to parse the val
    // (as the type frame is already parsed).
    let val = match penv.get_bytes(bytes.tag, 2) {
        Ok(v) => v,
        _ => return -2,
    };
    // By some magic, we know that this is 0x6000...
    let val2 = (val[..2]).try_into().expect("???");
    let maximum_data = (u16::from_be_bytes(val2) & 0x3FFF) as u64;

    // This is kinda magic here, but it is just for benchmarking purposes.
    /* Don't forget this! */
    match penv.save_output(Frame::MaxData(MaxDataFrame { maximum_data }).into()) {
        Ok(()) => 0,
        _ => -3,
    }
}

#[no_mangle]
pub extern fn process_frame_10(penv: &mut PluginEnv) -> i64 {
    /* Retrieve my data */
    // let fd = get_frame_data(tag);
    // No processing, no error
    // Voluntary buggy implementation.
    let md = match penv.get_input::<QVal>(0) {
        Ok(QVal::Frame(Frame::MaxData(mdf))) => mdf,
        _ => return -1,
    };
    match penv.set_connection(ConnectionField::MaxTxData, md.maximum_data) {
        Ok(()) => 0,
        _ => -2,
    }
}

#[no_mangle]
pub extern fn wire_len_10(penv: &mut PluginEnv) -> i64 {
    // Note that we might need the tag to infer the size.
    let len: usize = 2 + 1; // Just the frame type and one byte of data for now.
                            // And 0x42 needs 2 bytes...
    match penv.save_output(len.into()) {
        Ok(()) => 0,
        _ => -1,
    }
}

#[no_mangle]
pub extern fn on_frame_reserved_10(_penv: &mut PluginEnv) -> i64 {
    0
}

#[no_mangle]
pub extern fn notify_frame_42(_penv: &mut PluginEnv) -> i64 {
    0
}