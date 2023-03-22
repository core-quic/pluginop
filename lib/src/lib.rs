use std::{
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
};

use api::ConnectionToPlugin;
use common::PluginVal;
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

/// A trait allowing converting an host-implementation type to a `T` one, possibly
/// with the help of the `PluginHandler` if some information should not be directly
/// accessible to the plugins.
pub trait FromWithPH<T, CTP: ConnectionToPlugin>: Sized {
    fn from_with_ph(value: T, ph: &PluginHandler<CTP>) -> Self;
}

// For the following, a bit of explanation is required.
//
// Theoretically, we could have written the following.
// ```rust
// impl<T: Into<PluginVal>, CTP: ConnectionToPlugin> FromWithPH<T, CTP> for PluginVal {
//     fn from_with_ph(value: T, _: &PluginHandler<CTP>) -> Self {
//         value.into()
//     }
// }
// ```
//
// However, this would prevent us from doing the specific implementation for `Vec<u8>`.
// That's sad, but yet required.
macro_rules! impl_from_with_ph {
    ($e:ident, $t:ty) => {
        impl<CTP: ConnectionToPlugin> FromWithPH<$t, CTP> for $e {
            fn from_with_ph(v: $t, _: &PluginHandler<CTP>) -> Self {
                v.into()
            }
        }
    };
}

impl_from_with_ph!(PluginVal, i32);
impl_from_with_ph!(PluginVal, i64);
impl_from_with_ph!(PluginVal, u32);
impl_from_with_ph!(PluginVal, u64);
impl_from_with_ph!(PluginVal, f32);
impl_from_with_ph!(PluginVal, f64);
impl_from_with_ph!(PluginVal, usize);
impl_from_with_ph!(PluginVal, std::time::Duration);
impl_from_with_ph!(PluginVal, unix_time::Instant);
impl_from_with_ph!(PluginVal, std::net::SocketAddr);
impl_from_with_ph!(PluginVal, quic::QVal);

impl_from_with_ph!(PluginVal, quic::Header);
impl_from_with_ph!(PluginVal, quic::Frame);
impl_from_with_ph!(PluginVal, quic::RcvInfo);
impl_from_with_ph!(PluginVal, quic::KPacketNumberSpace);

impl<CTP: ConnectionToPlugin> FromWithPH<Vec<u8>, CTP> for PluginVal {
    fn from_with_ph(value: Vec<u8>, ph: &PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
    }
}

/// The reflexive trait of `FromWithPH`.
pub trait IntoWithPH<T, CTP: ConnectionToPlugin>: Sized {
    fn into_with_ph(self, ph: &PluginHandler<CTP>) -> T;
}

impl<T, CTP: ConnectionToPlugin> IntoWithPH<PluginVal, CTP> for T
where
    PluginVal: FromWithPH<T, CTP>,
{
    fn into_with_ph(self, ph: &PluginHandler<CTP>) -> PluginVal {
        PluginVal::from_with_ph(self, ph)
    }
}

/// A trait trying to convert a type to another one, possibly with side-data stored
/// in the `PluginHandler` if some information should not be directly accessible to
/// the plugins.
pub trait TryFromWithPH<T, CTP: ConnectionToPlugin>: Sized {
    type Error;

    fn try_from_with_ph(value: T, ph: &PluginHandler<CTP>) -> Result<Self, Self::Error>;
}

impl<T: TryFrom<PluginVal>, CTP: ConnectionToPlugin> TryFromWithPH<PluginVal, CTP> for T {
    type Error = <T as TryFrom<PluginVal>>::Error;

    fn try_from_with_ph(value: PluginVal, _: &PluginHandler<CTP>) -> Result<Self, Self::Error> {
        match value {
            PluginVal::Bytes(_) => todo!("try_from_with_ph bytes"),
            _ => value.try_into(),
        }
    }
}

/// The reflexive trait of `TryFromWithPH`.
pub trait TryIntoWithPH<T, CTP: ConnectionToPlugin>: Sized {
    type Error;

    fn try_into_with_ph(self, ph: &PluginHandler<CTP>) -> Result<T, Self::Error>;
}

impl<T: TryFrom<PluginVal>, CTP: ConnectionToPlugin> TryIntoWithPH<T, CTP> for PluginVal {
    type Error = <T as TryFrom<PluginVal>>::Error;

    fn try_into_with_ph(self, _: &PluginHandler<CTP>) -> Result<T, Self::Error> {
        match self {
            PluginVal::Bytes(_) => todo!("try_into_with_ph bytes"),
            _ => self.try_into(),
        }
    }
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
