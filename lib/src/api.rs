use pluginop_common::{
    quic::{ConnectionField, RecoveryField},
    APIResult, WASMLen,
};
use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Imports, Store, WasmPtr};

use crate::{plugin::Env, PluginizableConnection};

pub enum CTPError {
    BadType,
    SerializeError,
}

/// A trait that needs to be implemented by the host implementation to provide
/// plugins information from the host.
pub trait ConnectionToPlugin<'a, P: PluginizableConnection>: Send + Unpin {
    /// Gets the related `ConnectionField` and writes it as a serialized value in `w`.
    /// It is up to the plugin to correctly handle the value and perform the serialization.
    fn get_connection(&self, field: ConnectionField, w: &mut [u8]) -> bincode::Result<()>;
    /// Sets the related `ConnectionField` to the provided value, that was serialized with content
    /// `value`. It is this function responsibility to correctly convert the
    /// input to the right type.
    fn set_connection(&mut self, field: ConnectionField, value: &[u8]) -> Result<(), CTPError>;
    /// Gets the related `RecoveryField` and writes it as a serialized value in `w`. It is up to the
    /// plugin to correctly handle the value and perform the serialization.
    fn get_recovery(&self, w: &mut [u8], field: RecoveryField) -> bincode::Result<()>;
    /// Sets the related `RecoveryField` to the provided value, that was serialized with content
    /// `value`. It is this function responsibility to correctly convert the
    /// input to the right type.
    fn set_recovery(&mut self, field: RecoveryField, value: &[u8]);
}

// -------------------------------- API FUNCTIONS ----------------------------------

/// Stores a value generated by a running plugin as one of its outputs.
///
/// Function intended to be part of the Plugin API.
fn save_output_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    ptr: WasmPtr<u8>,
    len: WASMLen,
) -> APIResult {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    let output_cells = match ptr.slice(&view, len) {
        Ok(oc) => oc,
        Err(_) => return -3,
    };
    let output_serialized = match output_cells.read_to_vec() {
        Ok(os) => os,
        Err(_) => return -4,
    };
    match bincode::deserialize_from(&*output_serialized) {
        Ok(pv) => {
            env.data_mut().outputs.push(pv);
            0
        }
        Err(_) => -5,
    }
}

/// Stores a value generated by a running plugin as one of its outputs.
///
/// Function intended to be part of the Plugin API.
fn save_outputs_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    ptr: WasmPtr<u8>,
    len: WASMLen,
) -> APIResult {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    let output_cells = match ptr.slice(&view, len) {
        Ok(oc) => oc,
        Err(_) => return -3,
    };
    let output_serialized = match output_cells.read_to_vec() {
        Ok(os) => os,
        Err(_) => return -4,
    };
    match bincode::deserialize_from(&*output_serialized) {
        Ok(pvs) => {
            *env.data_mut().outputs = pvs;
            0
        }
        Err(_) => -5,
    }
}

/// Stores a value in an opaque, persistent store maintained by the host implementation. The stored
/// value is identified by its tag.
///
/// Function intended to be part of the Plugin API.
fn store_opaque_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    tag: u64,
    val: u32,
) {
    let env_data = env.data_mut();
    env_data.opaque_values.insert(tag, val);
}

const OPAQUE_ERR_VALUE: u64 = u64::MAX;

/// Gets the value associated to `tag` from the opaque, persistant store maintained by the host
/// implementation. If the `tag` is not present in the store, returns `u32::MAX`.
///
/// Function intended to be part of the Plugin API.
fn get_opaque_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    tag: u64,
) -> u64 {
    let env_data = env.data_mut();
    match env_data.opaque_values.get(&tag) {
        Some(v) => u64::from(*v),
        None => OPAQUE_ERR_VALUE,
    }
}

/// Removes the value associated to `tag` for the opaque, persistant store maintained by the host
/// implementation.
fn remove_opaque_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    tag: u64,
) -> u64 {
    let env_data = env.data_mut();
    match env_data.opaque_values.remove(&tag) {
        Some(v) => u64::from(v),
        None => OPAQUE_ERR_VALUE,
    }
}

/// Gets a serialized input.
///
/// Function intended to be part of the Plugin API.
///
/// Returns `0` if the operation succeeded. Otherwise, returns a negative value.
fn get_input_from_plugin<P: PluginizableConnection>(
    env: FunctionEnvMut<Env<P>>,
    index: u32,
    mem_ptr: WasmPtr<u8>,
    mem_len: WASMLen,
) -> APIResult {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    let input = match env.data().inputs.get(index as usize) {
        Some(i) => i,
        None => return -3,
    };
    // Sanity check to avoid memory overwrite.
    match bincode::serialized_size(input) {
        Ok(l) if l > mem_len.into() => return -4,
        Err(_) => return -5,
        _ => {}
    };
    // SAFETY: Given that plugins are single-threaded per-connection, this does
    // not introduce any UB.
    let memory_slice = unsafe { view.data_unchecked_mut() };
    match bincode::serialize_into(&mut memory_slice[mem_ptr.offset() as usize..], input) {
        Ok(()) => 0,
        Err(_) => -6,
    }
}

/// Gets the serialized inputs.
///
/// Function intended to be part of the Plugin API.
///
/// Returns `0` if the operation succeeded. Otherwise, returns a negative value.
fn get_inputs_from_plugin<P: PluginizableConnection>(
    env: FunctionEnvMut<Env<P>>,
    mem_ptr: WasmPtr<u8>,
    mem_len: WASMLen,
) -> APIResult {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    // Sanity check to avoid memory overwrite.
    match bincode::serialized_size(&*env.data().inputs) {
        Ok(l) if l > mem_len.into() => return -3,
        Err(_) => return -4,
        _ => {}
    };
    // SAFETY: Given that plugins are single-threaded per-connection, this does
    // not introduce any UB.
    let memory_slice = unsafe { view.data_unchecked_mut() };
    match bincode::serialize_into(
        &mut memory_slice[mem_ptr.offset() as usize..],
        &*env.data().inputs,
    ) {
        Ok(()) => 0,
        Err(_) => -5,
    }
}

/// Prints the content of the plugin memory located at the address `ptr` as a `str` having a length
/// of `len`.
///
/// Code from https://github.com/wasmerio/wasmer-rust-example/blob/master/examples/string.rs
///
/// Function intended to be part of the Plugin API.
pub fn print_from_plugin<P: PluginizableConnection>(
    env: FunctionEnvMut<Env<P>>,
    ptr: WasmPtr<u8>,
    len: WASMLen,
) {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return,
    };
    let view = memory.view(&env);

    // Uses the WasmPtr wrapper to simplify the operation to get ptr memory.
    if let Ok(s) = ptr.read_utf8_string(&view, len) {
        println!("{s}");
    }
}

/// Gets a specific connection field.
///
/// Function intended to be part of the Plugin API.
pub fn get_connection_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    field_ptr: WasmPtr<u8>,
    field_len: WASMLen,
    res_ptr: WasmPtr<u8>,
    res_len: WASMLen,
) -> i64 {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    // SAFETY: Given that plugins are single-threaded per-connection, this does
    // not introduce any UB.
    let memory_slice = unsafe { view.data_unchecked_mut() };
    let field = match bincode::deserialize_from(
        &memory_slice[field_ptr.offset() as usize..(field_ptr.offset() + field_len) as usize],
    ) {
        Ok(f) => f,
        Err(_) => return -3,
    };
    let ph = if let Some(ph) = env.data_mut().get_ph() {
        ph
    } else {
        return -4;
    };
    let conn = match ph.get_conn() {
        Some(c) => c,
        None => return -5,
    };
    match conn.get_conn().get_connection(
        field,
        &mut memory_slice[res_ptr.offset() as usize..(res_ptr.offset() + res_len) as usize],
    ) {
        Ok(_) => 0,
        Err(_) => -6,
    }
}

/// Gets a specific connection field.
///
/// Function intended to be part of the Plugin API.
pub fn set_connection_from_plugin<P: PluginizableConnection>(
    mut env: FunctionEnvMut<Env<P>>,
    field_ptr: WasmPtr<u8>,
    field_len: WASMLen,
    val_ptr: WasmPtr<u8>,
    val_len: WASMLen,
) -> i64 {
    let instance = if let Some(i) = env.data().get_instance() {
        i
    } else {
        return -1;
    };
    let instance = instance.as_ref();
    let memory = match instance.exports.get_memory("memory") {
        Ok(m) => m,
        Err(_) => return -2,
    };
    let view = memory.view(&env);
    // SAFETY: Given that plugins are single-threaded per-connection, this does
    // not introduce any UB.
    let memory_slice = unsafe { view.data_unchecked() };
    let field = match bincode::deserialize_from(
        &memory_slice[field_ptr.offset() as usize..(field_ptr.offset() + field_len) as usize],
    ) {
        Ok(f) => f,
        Err(_) => return -3,
    };
    let ph = if let Some(ph) = env.data_mut().get_ph() {
        ph
    } else {
        return -4;
    };
    let conn = match ph.get_conn_mut() {
        Some(c) => c,
        None => return -5,
    };
    match conn.get_conn_mut().set_connection(
        field,
        &memory_slice[val_ptr.offset() as usize..(val_ptr.offset() + val_len) as usize],
    ) {
        Ok(()) => 0,
        Err(_) => -6,
    }
}

macro_rules! exports_insert {
    ($e:ident, $s:ident, $env:ident, $f:ident) => {
        $e.insert(stringify!($f), Function::new_typed_with_env($s, $env, $f));
    };
}

/// Gets the imports that are common to any implementation.
///
/// The host implementation still needs to privide the following functions:
///    - buffer_get_bytes_from_plugin
///    - buffer_put_bytes_from_plugin
pub fn get_imports_with<P: PluginizableConnection>(
    mut exports: Exports,
    store: &mut Store,
    env: &FunctionEnv<Env<P>>,
) -> Imports {
    // Place here all the functions that are common to any host.
    exports_insert!(exports, store, env, save_output_from_plugin);
    exports_insert!(exports, store, env, save_outputs_from_plugin);
    exports_insert!(exports, store, env, store_opaque_from_plugin);
    exports_insert!(exports, store, env, get_opaque_from_plugin);
    exports_insert!(exports, store, env, remove_opaque_from_plugin);
    exports_insert!(exports, store, env, get_input_from_plugin);
    exports_insert!(exports, store, env, get_inputs_from_plugin);
    exports_insert!(exports, store, env, print_from_plugin);
    exports_insert!(exports, store, env, get_connection_from_plugin);
    exports_insert!(exports, store, env, set_connection_from_plugin);

    let mut imports = Imports::new();
    imports.register_namespace("env", exports);
    imports
}
