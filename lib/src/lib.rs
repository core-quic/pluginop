//! The core library enabling pluginization of operations.

use std::{
    fmt::Debug,
    io::Write,
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    time::Instant,
};

use api::{CTPError, ConnectionToPlugin};
use bytes::Buf;
use common::PluginVal;
use handler::PluginHandler;
use plugin::Env;
use pluginop_common::{quic, PluginOp};
use pluginop_octets::{OctetsMutPtr, OctetsPtr};
use pluginop_rawptr::{BytesMutPtr, CursorBytesPtr, RawMutPtr};
use unix_time::Instant as UnixInstant;
use wasmer::RuntimeError;

/// Permission that can be granted to plugins.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Permission {
    /// Permission to save output (should be always granted)
    Output,
    /// Permission to store opaque values (should be always granted)
    Opaque,
    /// Permission to access the Connection state
    ConnectionAccess,
    /// Permission to access the write byte buffer
    WriteBuffer,
    /// Permission to access the read byte buffer
    ReadBuffer,
}

/// An enum storing the actual content of `Bytes` that are not directly exposed
/// to plugins. Some side utilities are provided to let plugins access these
/// values under some conditions.
#[derive(Debug)]
pub enum BytesContent {
    Copied(Vec<u8>),
    ZeroCopy(OctetsPtr),
    ZeroCopyMut(OctetsMutPtr),
    CursorBytes(CursorBytesPtr),
    BytesMut(BytesMutPtr),
}

impl BytesContent {
    /// The number of bytes available to read.
    pub fn read_len(&self) -> usize {
        match self {
            BytesContent::Copied(v) => v.len(),
            BytesContent::ZeroCopy(o) => o.cap(),
            BytesContent::ZeroCopyMut(_) => 0,
            BytesContent::CursorBytes(c) => c.remaining(),
            BytesContent::BytesMut(_) => 0,
        }
    }

    pub fn write_len(&self) -> usize {
        match self {
            BytesContent::Copied(v) => v.capacity() - v.len(),
            BytesContent::ZeroCopy(_) => 0,
            BytesContent::ZeroCopyMut(o) => o.cap(),
            BytesContent::CursorBytes(_) => 0,
            BytesContent::BytesMut(b) => b.capacity() - b.len(),
        }
    }

    /// Whether there is any bytes to read.
    pub fn is_empty(&self) -> bool {
        self.read_len() == 0
    }

    /// Drains `len` bytes of the `BytesContent` and writes them in the slice `w`.
    pub fn write_into(&mut self, len: usize, mut w: &mut [u8]) -> Result<usize, CTPError> {
        match self {
            BytesContent::Copied(v) => w
                .write(v.drain(..len).as_slice())
                .map_err(|_| CTPError::BadBytes),
            BytesContent::ZeroCopy(o) => {
                let b = o.get_bytes(len).map_err(|_| CTPError::BadBytes)?;
                w.copy_from_slice(b.buf());
                Ok(len)
            }
            BytesContent::ZeroCopyMut(_) => Err(CTPError::BadBytes),
            BytesContent::CursorBytes(c) => {
                if c.remaining() < w.len() {
                    return Err(CTPError::BadBytes);
                }
                c.copy_to_slice(w);
                Ok(w.len())
            }
            BytesContent::BytesMut(_) => Err(CTPError::BadBytes),
        }
    }

    /// Extends the `BytesContent` with the content of `r`.
    pub fn extend_from(&mut self, r: &[u8]) -> Result<usize, CTPError> {
        match self {
            BytesContent::Copied(v) => {
                v.extend_from_slice(r);
                Ok(r.len())
            }
            BytesContent::ZeroCopy(_) => Err(CTPError::BadBytes),
            BytesContent::ZeroCopyMut(o) => {
                o.put_bytes(r).map_err(|_| CTPError::BadBytes)?;
                Ok(r.len())
            }
            BytesContent::CursorBytes(_) => Err(CTPError::BadBytes),
            BytesContent::BytesMut(b) => {
                b.extend_from_slice(r);
                Ok(r.len())
            }
        }
    }
}

impl From<Vec<u8>> for BytesContent {
    fn from(value: Vec<u8>) -> Self {
        Self::Copied(value)
    }
}

impl From<OctetsPtr> for BytesContent {
    fn from(value: OctetsPtr) -> Self {
        Self::ZeroCopy(value)
    }
}

impl From<OctetsMutPtr> for BytesContent {
    fn from(value: OctetsMutPtr) -> Self {
        Self::ZeroCopyMut(value)
    }
}

impl From<CursorBytesPtr> for BytesContent {
    fn from(value: CursorBytesPtr) -> Self {
        Self::CursorBytes(value)
    }
}

impl From<BytesMutPtr> for BytesContent {
    fn from(value: BytesMutPtr) -> Self {
        Self::BytesMut(value)
    }
}

/// Pluginization wrapper structure before the QUIC connection structure exists.
///
/// Some implementations first perform the TLS exchange before allocating a
/// QUIC dedicated structure. Such a structure provides support for always-enabled
/// plugin operations, but not for others.
pub struct TLSBeforeQUIC<CTP: ConnectionToPlugin> {
    pub ph: PluginHandler<CTP>,
    _pin: PhantomPinned,
}

impl<CTP: ConnectionToPlugin> TLSBeforeQUIC<CTP> {
    /// Create a `TLSBeforeQUIC` structure.
    pub fn new(exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports) -> Box<Self> {
        // We return a `Box` to pin the structure.
        Box::new(Self {
            ph: PluginHandler::new(exports_func),
            _pin: PhantomPinned,
        })
    }
}

/// Pluginization wrapper structure for a QUIC connection.
pub struct PluginizableConnection<CTP: ConnectionToPlugin> {
    /// The pluginization handler.
    pub ph: PluginHandler<CTP>,
    /// The actual, wrapped connection structure.
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

    /// Create a new `PluginizableConnection`.
    pub fn new_pluginizable_connection(
        exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports,
        conn: CTP,
    ) -> Box<PluginizableConnection<CTP>> {
        // We return a `Box` to pin the structure.
        Box::new(Self::new(exports_func, conn))
    }

    /// Immutable reference to the inner connection structure.
    pub fn get_conn(&self) -> &CTP {
        &self.conn
    }

    /// Mutable reference to the inner connection structure.
    pub fn get_conn_mut(&mut self) -> &mut CTP {
        &mut self.conn
    }

    /// Immutable reference to the pluginization handler.
    pub fn get_ph(&self) -> &PluginHandler<CTP> {
        &self.ph
    }

    /// Mutable reference to the pluginization handler.
    pub fn get_ph_mut(&mut self) -> &mut PluginHandler<CTP> {
        &mut self.ph
    }
}

/// A structure getting the parent structure of the current one.
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
        // SAFETY: Only valid if T implements `!Unpin`.
        unsafe { &**self.inner }
    }
}

impl<T> DerefMut for ParentReferencer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Only valid if T implements `!Unpin`.
        unsafe { &mut **self.inner }
    }
}

/// An error that may happen during the operations of this library.
#[derive(Clone, Debug)]
pub enum Error {
    /// An internal error occurred.
    ///
    /// Feel free to open an issue when encountering such errors.
    InternalError(String),

    /// The plugin cannot be loaded.
    PluginLoadingError(String),

    /// A runtime error raised by the virtual machine subsystem.
    RuntimeError(RuntimeError),

    /// No default provided for the related `PluginOp`.
    NoDefault(PluginOp),

    /// This plugin operation has been disabled.
    Disabled,

    /// The plugin returned a non-zero error code.
    OperationError(i64),

    /// There is no plugin function for the requested `PluginOp`.
    NoPluginFunction,
}

pub enum ProtoOpFunc<CTP: ConnectionToPlugin> {
    ProcessFrame(fn(&mut CTP, quic::Frame, &quic::Header, quic::RcvInfo, epoch: u64, now: Instant)),
}

/// A trait allowing converting an host-implementation type to a `T` one, possibly
/// with the help of the `PluginHandler` if some information should not be directly
/// accessible to the plugins.
pub trait FromWithPH<T, CTP: ConnectionToPlugin>: Sized {
    fn from_with_ph(value: T, ph: &mut PluginHandler<CTP>) -> Self;
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
            fn from_with_ph(v: $t, _: &mut PluginHandler<CTP>) -> Self {
                v.into()
            }
        }
    };
}

impl<CTP: ConnectionToPlugin> FromWithPH<Instant, CTP> for PluginVal {
    fn from_with_ph(value: Instant, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::UNIXInstant(ph.get_unix_instant_from_instant(value))
    }
}

impl_from_with_ph!(PluginVal, bool);
impl_from_with_ph!(PluginVal, i32);
impl_from_with_ph!(PluginVal, i64);
impl_from_with_ph!(PluginVal, u32);
impl_from_with_ph!(PluginVal, u64);
impl_from_with_ph!(PluginVal, f32);
impl_from_with_ph!(PluginVal, f64);
impl_from_with_ph!(PluginVal, usize);
impl_from_with_ph!(PluginVal, std::time::Duration);
impl_from_with_ph!(PluginVal, UnixInstant);
impl_from_with_ph!(PluginVal, std::net::SocketAddr);
impl_from_with_ph!(PluginVal, quic::QVal);

impl_from_with_ph!(PluginVal, quic::Header);
impl_from_with_ph!(PluginVal, quic::Frame);
impl_from_with_ph!(PluginVal, quic::RcvInfo);
impl_from_with_ph!(PluginVal, quic::KPacketNumberSpace);
impl_from_with_ph!(PluginVal, quic::PacketType);

impl<CTP: ConnectionToPlugin> FromWithPH<Vec<u8>, CTP> for PluginVal {
    fn from_with_ph(value: Vec<u8>, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
    }
}

/// The reflexive trait of `FromWithPH`.
pub trait IntoWithPH<T, CTP: ConnectionToPlugin>: Sized {
    fn into_with_ph(self, ph: &mut PluginHandler<CTP>) -> T;
}

impl<T, CTP: ConnectionToPlugin> IntoWithPH<PluginVal, CTP> for T
where
    PluginVal: FromWithPH<T, CTP>,
{
    fn into_with_ph(self, ph: &mut PluginHandler<CTP>) -> PluginVal {
        PluginVal::from_with_ph(self, ph)
    }
}

impl<CTP: ConnectionToPlugin> FromWithPH<octets::OctetsPtr, CTP> for PluginVal {
    fn from_with_ph(value: octets::OctetsPtr, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
    }
}

impl<CTP: ConnectionToPlugin> FromWithPH<octets::OctetsMutPtr, CTP> for PluginVal {
    fn from_with_ph(value: octets::OctetsMutPtr, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
    }
}

impl<CTP: ConnectionToPlugin> FromWithPH<CursorBytesPtr, CTP> for PluginVal {
    fn from_with_ph(value: CursorBytesPtr, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
    }
}

impl<CTP: ConnectionToPlugin> FromWithPH<BytesMutPtr, CTP> for PluginVal {
    fn from_with_ph(value: BytesMutPtr, ph: &mut PluginHandler<CTP>) -> Self {
        PluginVal::Bytes(ph.add_bytes_content(value.into()))
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

impl<CTP: ConnectionToPlugin, T: TryFromWithPH<PluginVal, CTP>> TryIntoWithPH<T, CTP>
    for PluginVal
{
    type Error = <T as TryFromWithPH<PluginVal, CTP>>::Error;

    fn try_into_with_ph(self, ph: &PluginHandler<CTP>) -> Result<T, Self::Error> {
        T::try_from_with_ph(self, ph)
    }
}

pub mod api;
pub mod handler;
pub mod plugin;

// Reexport common, macro and octets.
pub use pluginop_common as common;
pub use pluginop_macro;
pub use pluginop_octets as octets;

// Also need to expose structures to create exports.
pub use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};
