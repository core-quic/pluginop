//! A sub-crate of `pluginop` that should be imported by plugins.
//!
//! Playing directly with WebAssembly export functions can be cumbersome.
//! Instead, we propose a crate offering wrappers for these external calls,
//! making the plugin development possible by only relying on safe Rust.

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
pub use pluginop_common::PluginVal;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
pub use std::time::Duration;
pub use unix_time::Instant as UnixInstant;

/// The maximum size of a result, may be subject to future changes.
const SIZE: usize = 1500;

/// Errors that may occur when interacting with the Plugin API.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Error {
    /// An error occurred in the host-side API function.
    APICallError,
    /// Requested operation on [`Bytes`] is invalid.
    BadBytes,
    /// Type mismatch with what is expected.
    BadType,
    /// The internal plugin buffer is too short to carry the data.
    ShortInternalBuffer,
    /// An error occurred during the (de)serialization process.
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
    /* Gets a recovery field */
    fn get_recovery_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
    /* Sets a recovery field */
    fn set_recovery_from_plugin(
        field_ptr: WASMPtr,
        field_len: WASMLen,
        value_ptr: WASMPtr,
        value_len: WASMLen,
    ) -> APIResult;
    /* Calls a plugin control */
    fn poctl_from_plugin(
        id: u64,
        input_ptr: WASMPtr,
        input_len: WASMLen,
        res_ptr: WASMPtr,
        res_len: WASMLen,
    ) -> APIResult;
}

/// A companion structure, always passed as first argument of any plugin operation function,
/// enabling the plugin to interact with the host implementation.
#[repr(C)]
pub struct PluginEnv(WASMPtr);

impl PluginEnv {
    /// Store a new plugin output.
    pub fn save_output(&self, v: PluginVal) -> Result<()> {
        let serialized_value = postcard::to_allocvec(&v).map_err(|_| Error::SerializeError)?;
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

    /// Store all the plugin outputs.
    pub fn save_outputs(&self, v: &[PluginVal]) -> Result<()> {
        let serialized_value = postcard::to_allocvec(&v).map_err(|_| Error::SerializeError)?;
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

    /// Store an opaque value.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn store_opaque(&self, tag: u64, ptr: u32) {
        unsafe { store_opaque_from_plugin(tag, ptr) }
    }

    /// Get an opaque value.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn get_opaque(&self, tag: u64) -> Option<u32> {
        let ret = unsafe { get_opaque_from_plugin(tag) };
        match u32::try_from(ret) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }

    /// Remove an opaque value and returns it.
    #[deprecated(note = "Please use static variables, possibly with Mutex and `lazy_static` macro")]
    pub fn remove_opaque(&self, tag: u64) -> Option<u32> {
        let ret = unsafe { remove_opaque_from_plugin(tag) };
        match u32::try_from(ret) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }

    /// Print the provided string on the standard output.
    pub fn print(&self, s: &str) {
        unsafe { print_from_plugin(s.as_ptr() as WASMPtr, s.len() as WASMLen) }
    }

    /// Get a connection field.
    pub fn get_connection<T>(&self, field: quic::ConnectionField) -> Result<T>
    where
        T: TryFrom<PluginVal>,
    {
        let serialized_field = postcard::to_allocvec(&field).map_err(|_| Error::SerializeError)?;
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
            postcard::from_bytes(slice).map_err(|_| Error::SerializeError)?;
        plugin_val.try_into().map_err(|_| Error::BadType)
    }

    /// Set a connection field.
    pub fn set_connection<T>(&mut self, field: quic::ConnectionField, v: T) -> Result<()>
    where
        T: Into<PluginVal>,
    {
        let serialized_field = postcard::to_allocvec(&field).map_err(|_| Error::SerializeError)?;
        let serialized_value =
            postcard::to_allocvec(&v.into()).map_err(|_| Error::SerializeError)?;
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

    /// Get a recovery field.
    pub fn get_recovery<'de, T>(&self, field: quic::RecoveryField) -> T
    where
        T: Deserialize<'de>,
    {
        let serialized_field = postcard::to_allocvec(&field).expect("serialized field");
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
        postcard::from_bytes(slice).expect("no error")
    }

    /// Set a recovery field.
    pub fn set_recovery<T>(&mut self, field: quic::RecoveryField, v: T) -> Result<()>
    where
        T: Into<PluginVal>,
    {
        let serialized_field = postcard::to_allocvec(&field).map_err(|_| Error::SerializeError)?;
        let serialized_value =
            postcard::to_allocvec(&v.into()).map_err(|_| Error::SerializeError)?;
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

    /// Get an input.
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
        let input: PluginVal = match postcard::from_bytes(slice) {
            Ok(i) => i,
            Err(_) => return Err(Error::SerializeError),
        };
        input.try_into().map_err(|_| Error::SerializeError)
    }

    /// Get the inputs.
    pub fn get_inputs(&self) -> Result<Vec<PluginVal>> {
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        if unsafe { get_inputs_from_plugin(res.as_mut_ptr() as WASMPtr, SIZE as WASMLen) } != 0 {
            return Err(Error::ShortInternalBuffer);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        postcard::from_bytes(slice).map_err(|_| Error::SerializeError)
    }

    /// Read some bytes and advances the related buffer (i.e., multiple calls give different results).
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

    /// Write some bytes and advances the related buffer (i.e., multiple calls gives different results).
    pub fn put_bytes(&mut self, tag: u64, b: &[u8]) -> Result<usize> {
        let written =
            unsafe { put_bytes_from_plugin(tag, b.as_ptr() as WASMPtr, b.len() as WASMLen) };
        if written < 0 {
            return Err(Error::BadBytes);
        }
        Ok(written as usize)
    }

    /// Perform a registration to the host implementation. This operation is usually performed during
    /// the initialization of the plugin.
    pub fn register(&mut self, r: Registration) -> Result<()> {
        let serialized = postcard::to_allocvec(&r).map_err(|_| Error::SerializeError)?;
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
        let serialized_ts = postcard::to_allocvec(&ts).map_err(|_| Error::SerializeError)?;
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
        postcard::from_bytes(slice).map_err(|_| Error::SerializeError)
    }

    /// Fully enable the plugin operations.
    /// Such a call is needed to enable plugin operations that are not
    /// `always_enabled()`.
    pub fn enable(&self) {
        unsafe { enable_from_plugin() };
    }

    /// Invoke a plugin operation control operation.
    pub fn poctl(&mut self, id: u64, params: &[PluginVal]) -> Result<Vec<PluginVal>> {
        let serialized_inputs =
            postcard::to_allocvec(&params).map_err(|_| Error::SerializeError)?;
        let mut res = Vec::<u8>::with_capacity(SIZE).into_boxed_slice();
        let err = unsafe {
            poctl_from_plugin(
                id,
                serialized_inputs.as_ptr() as WASMPtr,
                serialized_inputs.len() as WASMLen,
                res.as_mut_ptr() as WASMPtr,
                SIZE as WASMLen,
            )
        };
        if err != 0 {
            return Err(Error::APICallError);
        }
        let slice = unsafe { std::slice::from_raw_parts(res.as_ptr(), SIZE) };
        postcard::from_bytes(slice).map_err(|_| Error::SerializeError)
    }
}

/// A cell structure to be used in single-threaded plugins.
pub struct PluginCell<T>(UnsafeCell<T>);

impl<T> PluginCell<T> {
    pub fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }

    /// Get a mutable reference to the cell.
    // TODO: solve this lint.
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        // SAFETY: only valid in single-threaded mode, which is the case in the scope of the plugins.
        unsafe { &mut *self.0.get() }
    }
}

impl<T: Sync + Send> Deref for PluginCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: only valid in single-threaded mode, which is the case in the scope of the plugins.
        unsafe { &*self.0.get() }
    }
}

// SAFETY: only valid in single-threaded mode, which is the case in the scope of the plugins.
unsafe impl<T: Send> Send for PluginCell<T> {}
// SAFETY: only valid in single-threaded mode, which is the case in the scope of the plugins.
unsafe impl<T: Sync> Sync for PluginCell<T> {}

pub mod fd;
