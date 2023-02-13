use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use api::ConnectionToPlugin;
use handler::PluginHandler;
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

pub trait PluginizableConnection: std::fmt::Debug + Send + 'static {
    fn get_conn(&self) -> &dyn api::ConnectionToPlugin;
    fn get_conn_mut(&mut self) -> &mut dyn api::ConnectionToPlugin;
    fn get_ph(&self) -> &PluginHandler;
    fn get_ph_mut(&mut self) -> &mut PluginHandler;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
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

pub enum ProtoOpFunc {
    ProcessFrame(
        fn(
            &mut dyn ConnectionToPlugin,
            quic::Frame,
            &quic::Header,
            quic::RcvInfo,
            epoch: u64,
            now: Instant,
        ),
    ),
}

pub mod api;
pub mod handler;
pub mod plugin;
mod rawptr;

// Reexport common and macro.
pub use pluginop_common as common;
pub use pluginop_macro;
