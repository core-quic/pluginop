//! A sub-crate of `protocol-operation` that should be imported by plugins.

use std::cell::UnsafeCell;
use std::convert::TryInto;
use std::mem;
use std::ops::Deref;

pub use pluginop_common::quic;
use pluginop_common::APIResult;
pub use pluginop_common::PluginOp;
use pluginop_common::WASMLen;
use pluginop_common::WASMPtr;

use pluginop_common::quic::Registration;
pub use pluginop_common::Bytes;
use pluginop_common::PluginVal;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
pub use std::time::Duration;
pub use unix_time::Instant as UnixInstant;

/// The maximum size of a result, may be subject to future changes.
const SIZE: usize = 1500;

// Playing directly with export functions can be cumbersome. Instead, we propose wrappers for these
// external calls that are easier to use when developing plugins.

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Error {
    APICallError,
    BadBytes,
    BadType,
    ShortInternalBuffer,
    SerializeError,
}

pub type Result<T> = std::result::Result<T, Error>;

extern "C" {
    /* General output function */
    fn save_output_from_plugin(ptr: WASMPtr, len: WASMLen) -> APIResult;
    /* Output function to call only once, with all the outputs */
    fn save_outputs_from_plugin(ptr: WASMPtr, len: WASMLen) -> APIResult;
    /* Store opaque value */
    fn store_opaque_from_plugin(tag: u64, ptr: WASMPtr);
    /* Get opaque value */
    fn get_opaque_from_plugin(tag: u64) -> u64;
    /* Remove opaque value */
    fn remove_opaque_from_plugin(tag: u64) -> u64;
    /* Classical debug function, from
     * https://github.com/wasmerio/wasmer-rust-example/blob/master/examples/string.rs */
    fn print_from_plugin(ptr: WASMPtr, len: WASMLen);
    /* Gets a connection field */
    fn get_connection_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
    /* Sets a connection field */
    fn set_connection_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        value_ptr: WASMPtr,
        value_len: WASMLen,
    ) -> APIResult;
    /* Gets an input */
    fn get_input_from_plugin(index: u32, res_ptr: WASMPtr, res_len: WASMLen) -> APIResult;
    /* Gets all inputs */
    fn get_inputs_from_plugin(res_ptr: WASMPtr, res_len: WASMLen) -> APIResult;
    /* Read the bytes */
    fn get_bytes_from_plugin(tag: u64, len: u64, res_ptr: WASMPtr, res_len: WASMLen) -> i64;
    /* Put some bytes */
    fn put_bytes_from_plugin(tag: u64, ptr: WASMPtr, len: WASMLen) -> i64;
    /* Register a parametrized protocol operation */
    fn register_from_plugin(ptr: WASMPtr, len: WASMLen) -> i64;
    /* Set a custom timer */
    fn set_timer_from_plugin(ts_ptr: WASMPtr, ts_len: WASMLen, id: u64, timer_id: u64)
        -> APIResult;
    /* Cancel the timer with the given id */
    fn cancel_timer_from_plugin(id: u64) -> APIResult;
    /* Gets the current UNIX time */
    fn get_unix_instant_from_plugin(res_ptr: WASMPtr, res_len: WASMLen) -> APIResult;
    /* Fully enable the plugin operations */
    fn enable_from_plugin();
    /* Sets a recovery field */
    fn set_recovery_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        value_ptr: WASMPtr,
        value_len: WASMLen,
    ) -> APIResult;
    // ----- TODOs -----
    /* Functions for the buffer to read */
    fn buffer_get_bytes_from_plugin(ptr: WASMPtr, len: WASMLen) -> APIResult;
    /* Functions for the buffer to write */
    fn buffer_put_bytes_from_plugin(ptr: WASMPtr, len: WASMLen) -> APIResult;
    /* Subject to many API changes */
    fn call_proto_op_from_plugin(
        po_ptr: WASMPtr,
        po_len: WASMLen,
        po_args_ptr: WASMPtr,
        po_args_len: WASMLen,
        po_input_ptr: WASMPtr,
        po_input_len: WASMLen,
        po_res_ptr: WASMPtr,
        po_res_len: WASMLen,
    );
    /* Gets a recovery field */
    fn get_recovery_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
    /* Gets a sent packet field */
    fn get_sent_packet_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
    /* Get a received packet field */
    fn get_rcv_packet_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
    /* Generates a connection ID */
    fn generate_connection_id_from_plugin(res_ptr: WASMPtr, res_len: WASMLen) -> APIResult;
}

#[repr(C)]
pub struct PluginEnv(WASMPtr);

impl PluginEnv {
    /// Stores a new plugin output.
    pub fn save_output(&self, v: PluginVal) -> Result<()> {
        let serialized_value = bincode::serialize(&v).map_err(|_| Error::SerializeError)?;
        match unsafe {
            save_output_from_plugin(
                serialized_value.as_ptr() as WASMPtr,
                serialized_value.len() as WASMLen,
            )
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Stores a new plugin output.
    pub fn save_outputs(&self, v: &[PluginVal]) -> Result<()> {
        let serialized_value = bincode::serialize(&v).map_err(|_| Error::SerializeError)?;
        match unsafe {
            save_outputs_from_plugin(
                serialized_value.as_ptr() as WASMPtr,
                serialized_value.len() as WASMLen,
            )
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Stores an opaque value.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn store_opaque(&self, tag: u64, ptr: u32) {
        unsafe { store_opaque_from_plugin(tag, ptr) }
    }

    /// Gets an opaque value.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn get_opaque(&self, tag: u64) -> Option<u32> {
        let ret = unsafe { get_opaque_from_plugin(tag) };
        match u32::try_from(ret) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }

    /// Removes an opaque value and returns it.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn remove_opaque(&self, tag: u64) -> Option<u32> {
        let ret = unsafe { remove_opaque_from_plugin(tag) };
        match u32::try_from(ret) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }

    /// Prints the provided string on the standard output.
    pub fn print(&self, s: &str) {
        unsafe { print_from_plugin(s.as_ptr() as WASMPtr, s.len() as WASMLen) }
    }

    /// Gets a connection field.
    pub fn get_connection<T>(&self, field: quic::ConnectionField) -> Result<T>
    where
        T: TryFrom<PluginVal>,
    {
        let serialized_field = bincode::serialize(&field).map_err(|_| Error::SerializeError)?;
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        let err = unsafe {
            get_connection_from_plugin(
                serialized_field.as_ptr() as WASMPtr,
                serialized_field.len() as WASMLen,
                res.as_mut_ptr() as WASMPtr,
                SIZE as WASMLen,
            )
        };
        if err != 0 {
            return Err(Error::APICallError);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        let plugin_val: PluginVal =
            bincode::deserialize(slice).map_err(|_| Error::SerializeError)?;
        plugin_val.try_into().map_err(|_| Error::BadType)
    }

    /// Sets a connection field.
    pub fn set_connection<T>(&mut self, field: quic::ConnectionField, v: T) -> Result<()>
    where
        T: Into<PluginVal>,
    {
        let serialized_field = bincode::serialize(&field).map_err(|_| Error::SerializeError)?;
        let serialized_value = bincode::serialize(&v.into()).map_err(|_| Error::SerializeError)?;
        match unsafe {
            set_connection_from_plugin(
                serialized_field.as_ptr() as WASMPtr,
                serialized_field.len() as WASMLen,
                serialized_value.as_ptr() as WASMPtr,
                serialized_value.len() as WASMLen,
            )
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Gets a recovery field.
    pub fn get_recovery<'de, T>(&self, field: quic::RecoveryField) -> T
    where
        T: Deserialize<'de>,
    {
        let serialized_field = bincode::serialize(&field).expect("serialized field");
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        unsafe {
            get_recovery_from_plugin(
                serialized_field.as_ptr() as WASMPtr,
                serialized_field.len() as WASMLen,
                res.as_mut_ptr() as WASMPtr,
                SIZE as WASMLen,
            );
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        bincode::deserialize(slice).expect("no error")
    }

    /// Sets a recovery field.
    pub fn set_recovery<T>(&mut self, field: quic::RecoveryField, v: T) -> Result<()>
    where
        T: Into<PluginVal>,
    {
        let serialized_field = bincode::serialize(&field).map_err(|_| Error::SerializeError)?;
        let serialized_value = bincode::serialize(&v.into()).map_err(|_| Error::SerializeError)?;
        match unsafe {
            set_recovery_from_plugin(
                serialized_field.as_ptr() as WASMPtr,
                serialized_field.len() as WASMLen,
                serialized_value.as_ptr() as WASMPtr,
                serialized_value.len() as WASMLen,
            )
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
        
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
                serialized_field.as_ptr() as WASMPtr,
                serialized_field.len() as WASMLen,
                res.as_mut_ptr() as WASMPtr,
                SIZE as WASMLen,
            );
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        bincode::deserialize(slice).expect("no error")
    }

    /// Gets an input. May panic.
    pub fn get_input<T>(&self, index: u32) -> Result<T>
    where
        T: TryFrom<PluginVal>,
        <T as TryFrom<PluginVal>>::Error: std::fmt::Debug,
    {
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        if unsafe { get_input_from_plugin(index, res.as_mut_ptr() as WASMPtr, SIZE as WASMLen) }
            != 0
        {
            return Err(Error::ShortInternalBuffer);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        let input: PluginVal = match bincode::deserialize(slice) {
            Ok(i) => i,
            Err(_) => return Err(Error::SerializeError),
        };
        input.try_into().map_err(|_| Error::SerializeError)
    }

    /// Gets the inputs.
    pub fn get_inputs(&self) -> Result<Vec<PluginVal>> {
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        if unsafe { get_inputs_from_plugin(res.as_mut_ptr() as WASMPtr, SIZE as WASMLen) } != 0 {
            return Err(Error::ShortInternalBuffer);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        bincode::deserialize(slice).map_err(|_| Error::SerializeError)
    }

    /// Reads some bytes and advances the related buffer (i.e., multiple calls gives different results).
    pub fn get_bytes(&mut self, tag: u64, len: u64) -> Result<Vec<u8>> {
        let mut res = Vec::<u8>::with_capacity(len as usize).into_boxed_slice();
        let len =
            unsafe { get_bytes_from_plugin(tag, len, res.as_mut_ptr() as WASMPtr, len as WASMLen) };
        if len < 0 {
            return Err(Error::BadBytes);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), len as usize) };
        Ok(slice.to_vec())
    }

    /// Writes some bytes and advances the related buffer (i.e., multiple calls gives different results).
    pub fn put_bytes(&mut self, tag: u64, b: &[u8]) -> Result<usize> {
        let written =
            unsafe { put_bytes_from_plugin(tag, b.as_ptr() as WASMPtr, b.len() as WASMLen) };
        if written < 0 {
            return Err(Error::BadBytes);
        }
        Ok(written as usize)
    }

    pub fn register(&mut self, r: Registration) -> Result<()> {
        let serialized = bincode::serialize(&r).map_err(|_| Error::SerializeError)?;
        match unsafe {
            register_from_plugin(serialized.as_ptr() as WASMPtr, serialized.len() as WASMLen)
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Set a timer at the provided time to call the given callback function with the
    /// provided name.
    ///
    /// Returns the identifier to the timer event, as provided as argument.
    pub fn set_timer(&mut self, ts: UnixInstant, id: u64, timer_id: u64) -> Result<()> {
        let serialized_ts = bincode::serialize(&ts).map_err(|_| Error::SerializeError)?;
        match unsafe {
            set_timer_from_plugin(
                serialized_ts.as_ptr() as WASMPtr,
                serialized_ts.len() as WASMLen,
                id,
                timer_id,
            )
        } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Cancel the timer event having the identifier provided.
    pub fn cancel_timer(&mut self, id: u64) -> Result<()> {
        match unsafe { cancel_timer_from_plugin(id) } {
            0 => Ok(()),
            _ => Err(Error::APICallError),
        }
    }

    /// Get the current UNIX instant.
    pub fn get_unix_instant(&self) -> Result<UnixInstant> {
        let size = mem::size_of::<UnixInstant>();
        let mut res = Vec::<u8>::with_capacity(size).into_boxed_slice();
        let err =
            unsafe { get_unix_instant_from_plugin(res.as_mut_ptr() as WASMPtr, size as WASMLen) };
        if err != 0 {
            return Err(Error::APICallError);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), size) };
        bincode::deserialize(slice).map_err(|_| Error::SerializeError)
    }

    /// Fully enable the plugin operations.
    /// Such a call is needed to enable plugin operations that are not
    /// `always_enabled()`.
    pub fn enable(&self) {
        unsafe { enable_from_plugin() };
    }
}

/// A cell structure to be used in single-threaded plugins.
pub struct PluginCell<T>(UnsafeCell<T>);

impl<T> PluginCell<T> {
    pub fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }

    // TODO: solve this lint.
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: Sync + Send> Deref for PluginCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

unsafe impl<T: Send> Send for PluginCell<T> {}
unsafe impl<T: Sync> Sync for PluginCell<T> {}

#[allow(dead_code)]
mod todo {
    use pluginop_common::{
        quic::{self, ConnectionId},
        APIResult, PluginOp, PluginVal, WASMLen, WASMPtr,
    };
    use serde::{Deserialize, Serialize};

    use crate::{
        buffer_get_bytes_from_plugin, buffer_put_bytes_from_plugin, call_proto_op_from_plugin,
        generate_connection_id_from_plugin, get_sent_packet_from_plugin, set_recovery_from_plugin,
        PluginEnv, SIZE,
    };

    impl PluginEnv {
        /// Gets a sent packet field.
        fn get_sent_packet<'de, T>(field: quic::SentPacketField) -> T
        where
            T: Deserialize<'de>,
        {
            let serialized_field = bincode::serialize(&field).expect("serialized field");
            let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
            unsafe {
                // FIXME we should handle error.
                get_sent_packet_from_plugin(
                    serialized_field.as_ptr() as WASMPtr,
                    serialized_field.len() as WASMLen,
                    res.as_mut_ptr() as WASMPtr,
                    SIZE as WASMLen,
                );
            }
            let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
            bincode::deserialize(slice).expect("no error")
        }

        /// Reads bytes from a buffer. The read bytes are consumed.
        fn buffer_get_bytes(&self, b: &mut Vec<u8>) -> APIResult {
            unsafe {
                buffer_get_bytes_from_plugin(
                    b.as_mut_slice() as *mut [u8] as *mut u8 as WASMPtr,
                    b.len() as WASMLen,
                )
            }
        }

        /// Writes bytes in a buffer.
        fn buffer_put_bytes(&self, b: &[u8]) -> i64 {
            unsafe { buffer_put_bytes_from_plugin(b.as_ptr() as WASMPtr, b.len() as WASMLen) }
        }

        /// Reads a variable integer from the read buffer and advances it.
        fn buffer_get_varint(&self) -> (i64, u64) {
            let mut val: Vec<u8> = vec![0];
            let read = self.buffer_get_bytes(&mut val);
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
                2 => val2.extend_from_slice(&[0; 3]),
                3 => val2.extend_from_slice(&[0; 7]),
                _ => unreachable!(),
            };
            let read2 = self.buffer_get_bytes(&mut val2);
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
                3 => u64::from_be_bytes(val[0..8].try_into().unwrap()),
                _ => unreachable!(),
            };
            (read + read2, v)
        }

        /// Writes a integer using variable-length encoding in the write buffer.
        fn buffer_put_varint(&self, v: u64) -> i64 {
            let mut vb = v.to_be_bytes();
            let write_bytes: Vec<u8> = if v < 64 {
                vb[7..8].to_vec()
            } else if v < 16384 {
                vb[6] |= 0x40;
                vb[6..8].to_vec()
            } else if v < 1073741824 {
                vb[4] |= 0x80;
                vb[4..8].to_vec()
            } else {
                vb[0] |= 0xc0;
                vb[0..8].to_vec()
            };
            self.buffer_put_bytes(&write_bytes)
        }

        /// Calls the protocol operation `po` with the provided arguments.
        fn call_protoop(
            po: PluginOp,
            args: Vec<PluginVal>,
            inputs: Vec<PluginVal>,
        ) -> Vec<PluginVal> {
            let serialized_po = bincode::serialize(&po).expect("serialized po");
            let serialized_args = bincode::serialize(&args).expect("serialized args");
            let serialized_inputs = bincode::serialize(&inputs).expect("serialized inputs");
            let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
            unsafe {
                call_proto_op_from_plugin(
                    serialized_po.as_ptr() as WASMPtr,
                    serialized_po.len() as WASMLen,
                    serialized_args.as_ptr() as WASMPtr,
                    serialized_args.len() as WASMLen,
                    serialized_inputs.as_ptr() as WASMPtr,
                    serialized_inputs.len() as WASMLen,
                    res.as_mut_ptr() as WASMPtr,
                    SIZE as WASMLen,
                );
            }
            let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
            bincode::deserialize(slice).expect("no error")
        }

        /// Generates a connection ID for the connection and record it for the endpoint. Returns `None` if
        /// the connection ID cannot be generated for some reason.
        fn generate_connection_id() -> Option<ConnectionId> {
            let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
            let err = unsafe {
                generate_connection_id_from_plugin(res.as_mut_ptr() as WASMPtr, SIZE as WASMLen)
            };
            if err != 0 {
                return None;
            }
            let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
            let cid: ConnectionId = bincode::deserialize(slice).expect("no error");
            Some(cid)
        }
    }
}

pub mod fd;
