use std::{
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
};

use api::ConnectionToPlugin;
use handler::PluginHandler;
use plugin::Env;
use pluginop_common::{quic, PluginInputType, PluginOp, PluginOutputType};
use rawptr::RawMutPtr;
use unix_time::Instant;
use wasmer::{RuntimeError, TypedFunction};

pub type PluginFunction = TypedFunction<PluginInputType, PluginOutputType>;

#[derive(Default)]
pub struct POCode {
    pre: Option<PluginFunction>,
    replace: Option<PluginFunction>,
    post: Option<PluginFunction>,
}

pub struct PluginizableConnection<CTP: ConnectionToPlugin> {
    pub ph: PluginHandler<CTP>,
    pub conn: CTP,
    _pin: PhantomPinned,
}

impl<CTP: ConnectionToPlugin> PluginizableConnection<CTP> {
    fn new(exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports, conn: CTP) -> Self {
        Self {
            ph: PluginHandler::new(exports_func),
            conn,
            _pin: PhantomPinned,
        }
    }

    pub fn new_pluginizable_connection(
        exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports,
        conn: CTP,
    ) -> Box<PluginizableConnection<CTP>> {
        Box::new(Self::new(exports_func, conn))
    }

    pub fn get_conn(&self) -> &CTP {
        &self.conn
    }

    pub fn get_conn_mut(&mut self) -> &mut CTP {
        &mut self.conn
    }

    pub fn get_ph(&self) -> &PluginHandler<CTP> {
        &self.ph
    }

    pub fn get_ph_mut(&mut self) -> &mut PluginHandler<CTP> {
        &mut self.ph
    }
}

#[derive(Debug)]
pub struct ParentReferencer<T> {
    inner: RawMutPtr<T>,
}

impl<T> ParentReferencer<T> {
    pub fn new(v: *mut T) -> ParentReferencer<T> {
        Self {
            inner: RawMutPtr::new(v),
        }
    }
}

impl<T> Deref for ParentReferencer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &**self.inner }
    }
}

impl<T> DerefMut for ParentReferencer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut **self.inner }
    }
}

/// An error that may happen during the operations of this library.
#[derive(Clone, Debug)]
pub enum Error {
    RuntimeError(RuntimeError),
    NoDefault(PluginOp),
    OutputConversionError(String),
    OperationError(i64),
}

pub enum ProtoOpFunc<CTP: ConnectionToPlugin> {
    ProcessFrame(fn(&mut CTP, quic::Frame, &quic::Header, quic::RcvInfo, epoch: u64, now: Instant)),
}

pub mod api;
pub mod handler;
pub mod plugin;
mod rawptr;

// Reexport common and macro.
pub use pluginop_common as common;
pub use pluginop_macro;

// Also need to expose structures to create exports.
pub use wasmer::{Exports, FunctionEnv, Store};
