//! Sub-crate of `protocol-operation` containing structures needed for operations of both the
//! host instance and the plugin ones.

use std::{hash::Hash, net::SocketAddr, num::ParseIntError, time::Duration};

use serde::{Deserialize, Serialize};
use unix_time::Instant as UnixInstant;

pub type PluginInputType = u32;
pub type PluginOutputType = i64;
pub type WASMPtr = u32;
pub type WASMLen = u32;
pub type APIResult = i64;

/// The different conversion errors that may arise with plugin-processable structures.
#[derive(Clone, Debug)]
pub enum ConversionError {
    InvalidBool,
    InvalidI32,
    InvalidI64,
    InvalidU32,
    InvalidU64,
    InvalidF32,
    InvalidF64,
    InvalidUsize,
    InvalidBytes,
    InvalidDuration,
    InvalidInstant,
    InvalidFrame,
    InvalidFrameParam,
    InvalidHeader,
    InvalidSentPacket,
    InvalidSocketAddr,
    InvalidQVal,
}

/// The actual plugin operations.
///
/// FIXME: move these protoops in their respective protocols.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, PartialOrd)]
pub enum PluginOp {
    /// Operation that will always be called once the plugin got loaded.
    Init,

    /// Operation with no particular meaning, only for testing purposes.
    Test,

    /// Plugin control operation, unspecified protocol operations called by the application.
    PluginControl(u64),

    /// Specific protocol operation when a plugin triggers some timers.
    OnPluginTimeout(u64),

    /// Decode from the wire the QUIC transport parameter having the specified type.
    DecodeTransportParameter(u64),
    /// Write to the wire the QUIC transport parameter having the specified type.
    WriteTransportParameter(u64),

    /// Provide some textual logging of the frame with specified type.
    LogFrame(u64),
    /// Report whether the frame was successfully acknowledged or lost.
    NotifyFrame(u64),
    /// Callback event when the frame has been confirmed scheduling for the current packet.
    OnFrameReserved(u64),
    /// Converts a wire-format frame into a plugin-processable structure.
    ParseFrame(u64),
    /// Generate a plugin-processable structure representing the next frame to send.
    PrepareFrame(u64),
    /// Process, at receiver side, the plugin-processable structure.
    ProcessFrame(u64),
    /// Return whether a frame of the corresponding type should be scheduled for sending in
    /// the next packet.
    ShouldSendFrame(u64),
    /// Return the length of the complete frame on the wire.
    WireLen(u64),
    /// From a plugin-processable structure, write the frame on the wire.
    WriteFrame(u64),

    // These derive from quic-invariants, from version.
    // Note that parsing them is an invariant, so we just have process here.
    ProcessLongHeader(u32),
    ProcessShortHeader(u32),
    ProcessVersionNegotiation,

    GetPacketToSend,

    // I think at some point there should have some value here.
    // TODO not supported yet.
    DecryptPacket,

    OnPacketProcessed,

    OnPacketSent,
    SetLossDetectionTimer,
    UpdateRtt,

    /// For experimentation purposes.
    Other([u8; 32]),
}

/// The different anchors where plugin bytecodes can be attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// Execute just before calling the operation. Cannot modify the running context.
    Before,
    /// Execute in place of the operation. Can modify the running context.
    Define,
    /// Execute just after returning from the operation. Cannot modify the running context.
    After,
}

impl Anchor {
    /// Returns an index value for the Anchor.
    pub fn index(&self) -> usize {
        match self {
            Anchor::Before => 0,
            Anchor::Define => 1,
            Anchor::After => 2,
        }
    }
}

fn extract_po_param(name: &str) -> Result<u64, ParseIntError> {
    let end_num = name.rfind('_').map(|i| &name[i + 1..]).unwrap_or("");
    u64::from_str_radix(end_num, 16)
}

impl PluginOp {
    /// Convert a string into the corresponding protocol operation and anchor.
    ///
    /// FIXME find a more idiomatic way
    pub fn from_name(name: &str) -> (PluginOp, Anchor) {
        let (name, anchor) = if let Some(po_name) = name.strip_prefix("pre_") {
            (po_name, Anchor::Before)
        } else if let Some(po_name) = name.strip_prefix("before_") {
            (po_name, Anchor::Before)
        } else if let Some(po_name) = name.strip_prefix("post_") {
            (po_name, Anchor::After)
        } else if let Some(po_name) = name.strip_prefix("after_") {
            (po_name, Anchor::After)
        } else {
            (name, Anchor::Define)
        };

        if name == "init" {
            (PluginOp::Init, anchor)
        } else if name.starts_with("decode_transport_parameter_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::DecodeTransportParameter(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_long_header_") {
            match extract_po_param(name) {
                Ok(version) => match u32::try_from(version) {
                    Ok(version) => (PluginOp::ProcessLongHeader(version), anchor),
                    Err(_) => panic!("Invalid protocol operation name"),
                },
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_short_header_") {
            match extract_po_param(name) {
                Ok(version) => match u32::try_from(version) {
                    Ok(version) => (PluginOp::ProcessShortHeader(version), anchor),
                    Err(_) => panic!("Invalid protocol operation name"),
                },
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name == "process_version_negotiation" {
            (PluginOp::ProcessVersionNegotiation, anchor)
        } else if name.starts_with("write_transport_parameter_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::WriteTransportParameter(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("log_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::LogFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("notify_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::NotifyFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("on_frame_reserved_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::OnFrameReserved(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("parse_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::ParseFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("prepare_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::PrepareFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::ProcessFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("should_send_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::ShouldSendFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("wire_len_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::WireLen(frame_type), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name.starts_with("write_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (PluginOp::WriteFrame(frame_type), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name.starts_with("plugin_control_") {
            match extract_po_param(name) {
                Ok(val) => (PluginOp::PluginControl(val), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name.starts_with("on_plugin_timeout_") {
            match extract_po_param(name) {
                Ok(val) => (PluginOp::OnPluginTimeout(val), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name == "get_packet_to_send" {
            (PluginOp::GetPacketToSend, anchor)
        } else if name == "decrypt_packet" {
            (PluginOp::DecryptPacket, anchor)
        } else if name == "on_packet_processed" {
            (PluginOp::OnPacketProcessed, anchor)
        } else if name == "on_packet_sent" {
            (PluginOp::OnPacketSent, anchor)
        } else if name == "set_loss_detection_timer" {
            (PluginOp::SetLossDetectionTimer, anchor)
        } else if name == "update_rtt" {
            (PluginOp::UpdateRtt, anchor)
        } else {
            let mut name_array = [0; 32];
            name_array[..name.len()].copy_from_slice(name.as_bytes());
            (PluginOp::Other(name_array), anchor)
        }
    }

    /// Returns whether the plugin operation can be called, even if it is not fully
    /// loaded.
    pub fn always_enabled(&self) -> bool {
        matches!(
            self,
            PluginOp::Init
                | PluginOp::DecodeTransportParameter(_)
                | PluginOp::WriteTransportParameter(_)
        )
    }
}

/// A type, implemented as an access token, providing a capability-based access to raw
/// bytes between the host implementation and the plugin bytecode.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct Bytes {
    /// The tag to use to retrieve the associated data.
    pub tag: u64,
    /// The maximum number of bytes that can be fetched.
    pub max_read_len: u64,
    /// The maximum number of bytes that can be written.
    pub max_write_len: u64,
}

/// Values used to communicate with underlying plugins, either as inputs or
/// outputs.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum PluginVal {
    /// A boolean value.
    Bool(bool),
    /// A i32.
    I32(i32),
    /// A i64.
    I64(i64),
    /// A u32.
    U32(u32),
    /// A u64.
    U64(u64),
    /// A f32.
    F32(f32),
    /// A f64.
    F64(f64),
    /// A Usize, but encoded as a u64.
    Usize(u64),
    /// An access token to some raw bytes.
    Bytes(Bytes),
    /// A duration.
    Duration(Duration),
    /// A UNIX-based instant.
    UNIXInstant(UnixInstant),
    /// A socket address.
    SocketAddr(SocketAddr),
    /// QUIC specific inputs.
    QUIC(quic::QVal),
}

macro_rules! impl_from_try_from {
    ($e:ident, $v:ident, $t:ty, $err:ident, $verr:ident) => {
        impl From<$t> for $e {
            fn from(v: $t) -> Self {
                $e::$v(v)
            }
        }

        impl TryFrom<$e> for $t {
            type Error = $err;

            fn try_from(v: $e) -> Result<Self, Self::Error> {
                match v {
                    $e::$v(v) => Ok(v),
                    _ => Err($err::$verr),
                }
            }
        }
    };
}

impl From<usize> for PluginVal {
    fn from(value: usize) -> Self {
        PluginVal::Usize(value as u64)
    }
}

impl TryFrom<PluginVal> for usize {
    type Error = ConversionError;

    fn try_from(value: PluginVal) -> Result<Self, Self::Error> {
        match value {
            PluginVal::Usize(v) => Ok(v as Self),
            _ => Err(ConversionError::InvalidUsize),
        }
    }
}

impl TryFrom<PluginVal> for () {
    type Error = ConversionError;

    fn try_from(_: PluginVal) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl_from_try_from!(PluginVal, Bool, bool, ConversionError, InvalidBool);
impl_from_try_from!(PluginVal, I32, i32, ConversionError, InvalidI32);
impl_from_try_from!(PluginVal, I64, i64, ConversionError, InvalidI64);
impl_from_try_from!(PluginVal, U32, u32, ConversionError, InvalidU32);
impl_from_try_from!(PluginVal, U64, u64, ConversionError, InvalidU64);
impl_from_try_from!(PluginVal, F32, f32, ConversionError, InvalidF32);
impl_from_try_from!(PluginVal, F64, f64, ConversionError, InvalidF64);
impl_from_try_from!(PluginVal, Bytes, Bytes, ConversionError, InvalidBytes);
impl_from_try_from!(
    PluginVal,
    Duration,
    Duration,
    ConversionError,
    InvalidDuration
);
impl_from_try_from!(
    PluginVal,
    UNIXInstant,
    UnixInstant,
    ConversionError,
    InvalidInstant
);
impl_from_try_from!(
    PluginVal,
    SocketAddr,
    SocketAddr,
    ConversionError,
    InvalidSocketAddr
);
impl_from_try_from!(PluginVal, QUIC, quic::QVal, ConversionError, InvalidQVal);

pub mod quic;
