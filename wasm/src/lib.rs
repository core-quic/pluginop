//! A sub-crate of `protocol-operation` that should be imported by plugins.

use std::convert::TryInto;

pub use pluginop_common::quic;
pub use pluginop_common::PluginEnv;
pub use pluginop_common::PluginVal;
pub use pluginop_common::ProtoOp;

use pluginop_common::{quic::ConnectionId, Input};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
pub use unix_time::Instant;

/// The maximum size of a result, may be subject to future changes.
const SIZE: usize = 1500;

// Playing directly with export functions can be cumbersome. Instead, we propose wrappers for these
// external calls that are easier to use when developing plugins.

extern "C" {
    /* General output function */
    fn save_output_from_plugin(ptr: u32, len: u32);
    /* Store opaque value */
    fn store_opaque_from_plugin(tag: u64, ptr: u32);
    /* Get opaque value */
    fn get_opaque_from_plugin(tag: u64) -> u64;
    /* Remove opaque value */
    fn remove_opaque_from_plugin(tag: u64) -> u64;
    /* Functions for the buffer to read */
    fn buffer_get_bytes_from_plugin(ptr: u32, len: u32) -> i64;
    /* Functions for the buffer to write */
    fn buffer_put_bytes_from_plugin(ptr: u32, len: u32) -> i64;
    /* Classical debug function, from
     * https://github.com/wasmerio/wasmer-rust-example/blob/master/examples/string.rs */
    fn print_from_plugin(ptr: *const u8, len: usize);
    /* Subject to many API changes */
    fn call_proto_op_from_plugin(
        po_ptr: u32,
        po_len: u32,
        po_args_ptr: u32,
        po_args_len: u32,
        po_input_ptr: u32,
        po_input_len: u32,
        po_res_ptr: u32,
        po_res_len: u32,
    );
    /* Gets a connection field */
    fn get_connection_from_plugin(
        field_ptr: u32,
        field_len: u32,
        res_ptr: u32,
        res_len: u32,
    ) -> i64;
    /* Sets a connection field */
    fn set_connection_from_plugin(field_ptr: u32, field_len: u32, value_ptr: u32, value_len: u32);
    /* Gets a recovery field */
    fn get_recovery_from_plugin(field_ptr: u32, field_len: u32, res_ptr: u32, res_len: u32) -> i64;
    /* Sets a recovery field */
    fn set_recovery_from_plugin(field_ptr: u32, field_len: u32, value_ptr: u32, value_len: u32);
    /* Gets a sent packet field */
    fn get_sent_packet_from_plugin(
        field_ptr: u32,
        field_len: u32,
        res_ptr: u32,
        res_len: u32,
    ) -> i64;
    /* Get a received packet field */
    fn get_rcv_packet_from_plugin(
        field_ptr: u32,
        field_len: u32,
        res_ptr: u32,
        res_len: u32,
    ) -> i64;
    /* Gets the current time */
    fn get_current_time_from_plugin(res_ptr: u32, res_len: u32);
    /* Registers a protocol operation */
    fn register_from_plugin(field_ptr: u32, field_len: u32);
    /* Gets an input */
    fn get_input_from_plugin(index: u32, res_ptr: u32, res_len: u32);
    /* Gets the time */
    fn get_time_from_plugin(res_ptr: u32, res_len: u32);
    /* Generates a connection ID */
    fn generate_connection_id_from_plugin(res_ptr: u32, res_len: u32) -> i64;
    /* Set a custom timer */
    fn set_timer_from_plugin(ts_ptr: u32, ts_len: u32, id: u64, cb_ptr: u32, cb_len: u32) -> u64;
    /* Cancel the timer with the given id */
    fn cancel_timer_from_plugin(id: u64);
}

/// Stores a new plugin output.
pub fn save_output<T>(v: T)
where
    T: Serialize,
{
    let serialized_value = bincode::serialize(&v).expect("serialized value");
    unsafe {
        save_output_from_plugin(
            serialized_value.as_ptr() as u32,
            serialized_value.len() as u32,
        )
    }
}

/// Stores an opaque value.
#[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
pub fn store_opaque(tag: u64, ptr: u32) {
    unsafe { store_opaque_from_plugin(tag, ptr) }
}

/// Gets an opaque value.
#[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
pub fn get_opaque(tag: u64) -> Option<u32> {
    let ret = unsafe { get_opaque_from_plugin(tag) };
    match u32::try_from(ret) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

/// Removes an opaque value and returns it.
#[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
pub fn remove_opaque(tag: u64) -> Option<u32> {
    let ret = unsafe { remove_opaque_from_plugin(tag) };
    match u32::try_from(ret) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

/// Reads bytes from a buffer. The read bytes are consumed.
pub fn buffer_get_bytes(b: &mut Vec<u8>) -> i64 {
    unsafe {
        buffer_get_bytes_from_plugin(
            b.as_mut_slice() as *mut [u8] as *mut u8 as u32,
            b.len() as u32,
        )
    }
}

/// Writes bytes in a buffer.
pub fn buffer_put_bytes(b: &[u8]) -> i64 {
    unsafe { buffer_put_bytes_from_plugin(b.as_ptr() as u32, b.len() as u32) }
}

/// Reads a variable integer from the read buffer and advances it.
pub fn buffer_get_varint() -> (i64, u64) {
    let mut val: Vec<u8> = vec![0];
    let read = buffer_get_bytes(&mut val);
    if read != 1 {
        return (read, 0);
    }
    let l = (val[0] & 0xC0) / 0x40;
    // Already clear now the leading bits of first byte for parsing.
    val[0] &= 0x3F;
    let mut val2: Vec<u8> = Vec::new();
    match l {
        0 => {}
        1 => val2.push(0),
        2 => val2.extend_from_slice(&vec![0; 3]),
        3 => val2.extend_from_slice(&vec![0; 7]),
        _ => unreachable!(),
    };
    let read2 = buffer_get_bytes(&mut val2);
    match l {
        0 => {
            if read2 != 0 {
                return (read2, 0);
            }
        }
        1 => {
            if read2 != 1 {
                return (read2, 0);
            }
        }
        2 => {
            if read2 != 3 {
                return (read2, 0);
            }
        }
        3 => {
            if read2 != 7 {
                return (read2, 0);
            }
        }
        _ => unreachable!(),
    }
    val.extend_from_slice(&val2);
    let v: u64 = match l {
        0 => val[0].into(),
        1 => u16::from_be_bytes(val[0..2].try_into().unwrap()).into(),
        2 => u32::from_be_bytes(val[0..4].try_into().unwrap()).into(),
        3 => u64::from_be_bytes(val[0..8].try_into().unwrap()).into(),
        _ => unreachable!(),
    };
    (read + read2, v)
}

/// Writes a integer using variable-length encoding in the write buffer.
pub fn buffer_put_varint(v: u64) -> i64 {
    let mut vb = v.to_be_bytes();
    let write_bytes: Vec<u8> = if v < 64 {
        (&vb[7..8]).iter().map(|b| *b).collect()
    } else if v < 16384 {
        vb[6] |= 0x40;
        (&vb[6..8]).iter().map(|b| *b).collect()
    } else if v < 1073741824 {
        vb[4] |= 0x80;
        (&vb[4..8]).iter().map(|b| *b).collect()
    } else {
        vb[0] |= 0xc0;
        (&vb[0..8]).iter().map(|b| *b).collect()
    };
    buffer_put_bytes(&write_bytes)
}

/// Prints the provided string on the standard output.
pub fn print(s: &str) {
    unsafe { print_from_plugin(s.as_ptr(), s.len()) }
}

/// Gets a connection field.
pub fn get_connection<'de, T>(field: quic::ConnectionField) -> T
where
    T: Deserialize<'de>,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        // FIXME we should handle error
        get_connection_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            res.as_mut_ptr() as u32,
            SIZE as u32,
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("the requested type is not correct")
}

/// Sets a connection field.
pub fn set_connection<T>(field: quic::ConnectionField, v: T)
where
    T: Serialize,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let serialized_value = bincode::serialize(&v).expect("serialized value");
    unsafe {
        set_connection_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            serialized_value.as_ptr() as u32,
            serialized_value.len() as u32,
        );
    }
}

/// Gets a recovery field.
pub fn get_recovery<'de, T>(field: quic::RecoveryField) -> T
where
    T: Deserialize<'de>,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        get_recovery_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            res.as_mut_ptr() as u32,
            SIZE as u32,
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

/// Sets a recovery field.
pub fn set_recovery<T>(field: quic::RecoveryField, v: T)
where
    T: Serialize,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let serialized_value = bincode::serialize(&v).expect("serialized value");
    unsafe {
        set_recovery_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            serialized_value.as_ptr() as u32,
            serialized_value.len() as u32,
        );
    }
}

/// Gets a sent packet field.
pub fn get_sent_packet<'de, T>(field: quic::SentPacketField) -> T
where
    T: Deserialize<'de>,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        // FIXME we should handle error.
        get_sent_packet_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            res.as_mut_ptr() as u32,
            SIZE as u32,
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

/// Get a received packet field.
pub fn get_rcv_packet<'de, T>(field: quic::RcvPacketField) -> T
where
    T: Deserialize<'de>,
{
    let serialized_field = bincode::serialize(&field).expect("serialized field");
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        // FIXME we should handle error.
        get_rcv_packet_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
            res.as_mut_ptr() as u32,
            SIZE as u32,
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

/// Calls the protocol operation `po` with the provided arguments.
pub fn call_protoop(po: ProtoOp, args: Vec<PluginVal>, inputs: Vec<Input>) -> Vec<PluginVal> {
    let serialized_po = bincode::serialize(&po).expect("serialized po");
    let serialized_args = bincode::serialize(&args).expect("serialized args");
    let serialized_inputs = bincode::serialize(&inputs).expect("serialized inputs");
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        call_proto_op_from_plugin(
            serialized_po.as_ptr() as u32,
            serialized_po.len() as u32,
            serialized_args.as_ptr() as u32,
            serialized_args.len() as u32,
            serialized_inputs.as_ptr() as u32,
            serialized_inputs.len() as u32,
            res.as_mut_ptr() as u32,
            SIZE as u32,
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

pub fn get_current_time() -> Instant {
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        get_current_time_from_plugin(res.as_mut_ptr() as u32, SIZE as u32);
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

pub fn register(r: quic::Registration) {
    let serialized_field = bincode::serialize(&r).expect("serialized field");
    unsafe {
        register_from_plugin(
            serialized_field.as_ptr() as u32,
            serialized_field.len() as u32,
        )
    }
}

/// Gets an input. May panic.
pub fn get_input<T>(index: u32) -> T
where
    T: TryFrom<Input>,
    <T as TryFrom<Input>>::Error: std::fmt::Debug,
{
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        get_input_from_plugin(index, res.as_mut_ptr() as u32, SIZE as u32);
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    let input: Input = bincode::deserialize(slice).expect("no error");
    input.try_into().expect("cannot convert to wanted type")
}

pub fn get_time() -> unix_time::Instant {
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    unsafe {
        get_time_from_plugin(res.as_mut_ptr() as u32, SIZE as u32);
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    bincode::deserialize(slice).expect("no error")
}

/// Generates a connection ID for the connection and record it for the endpoint. Returns `None` if
/// the connection ID cannot be generated for some reason.
pub fn generate_connection_id() -> Option<ConnectionId> {
    let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
    let err = unsafe { generate_connection_id_from_plugin(res.as_mut_ptr() as u32, SIZE as u32) };
    if err != 0 {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE as usize) };
    let cid: ConnectionId = bincode::deserialize(slice).expect("no error");
    Some(cid)
}

/// Set a timer at the provided time to call the given callback function with the
/// provided name.
///
/// Returns the identifier to the timer event, as provided as argument.
pub fn set_timer(ts: unix_time::Instant, id: u64, callback_name: &str) -> u64 {
    let serialized_ts = bincode::serialize(&ts).expect("serialized ts");
    let serialized_cb = bincode::serialize(callback_name).expect("serialized callback_name");
    unsafe {
        set_timer_from_plugin(
            serialized_ts.as_ptr() as u32,
            serialized_ts.len() as u32,
            id,
            serialized_cb.as_ptr() as u32,
            serialized_cb.len() as u32,
        )
    }
}

/// Cancel the timer event having the identifier provided.
pub fn cancel_timer(id: u64) {
    unsafe { cancel_timer_from_plugin(id) }
}

pub mod fd;
