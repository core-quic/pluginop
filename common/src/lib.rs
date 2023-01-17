//! Sub-crate of `protocol-operation` containing structures needed for operations of both the
//! host instance and the plugin ones.

use serde::{Deserialize, Serialize};
use std::{hash::Hash, net::SocketAddr, num::ParseIntError, time::Duration};
use unix_time::Instant;

#[derive(Clone, Debug)]
pub enum ConversionError {
    InvalidDuration,
    InvalidInstant,
    InvalidFrame,
    InvalidFrameParam,
    InvalidHeader,
    InvalidSentPacket,
    InvalidSocketAddr,
}

// FIXME: move these protoops in their respective protocols.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ProtoOp {
    Init,

    // These derive from quic-invariants, from version.
    // Note that parsing them is an invariant, so we just have process here.
    ProcessLongHeader(u32),
    ProcessShortHeader(u32),
    ProcessVersionNegotiation,

    DecodeTransportParameter(u64),
    WriteTransportParameter(u64),

    LogFrame(u64),
    NotifyFrame(u64),
    OnFrameReserved(u64),
    ParseFrame(u64),
    PrepareFrame(u64),
    ProcessFrame(u64),
    ShouldSendFrame(u64),
    WireLen(u64),
    WriteFrame(u64),

    GetPacketToSend,

    // I think at some point there should have some value here.
    // TODO not supported yet.
    DecryptPacket,

    OnPacketProcessed,

    OnPacketSent,
    SetLossDetectionTimer,
    UpdateRtt,

    // Plugin control operation, unspecified protocol operations called by the application.
    PluginControl(u64),

    // TODO: think about the sending of packets, I think this should be version specific (only one protoop?).

    // In case we have custom protocol operations.
    Other([u8; 32]),
}

#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    Pre,
    Replace,
    Post,
}

fn extract_po_param(name: &str) -> Result<u64, ParseIntError> {
    let end_num = &name[name.rfind('_').unwrap() + 1..];
    u64::from_str_radix(end_num, 16)
}

impl ProtoOp {
    // FIXME find a more idiomatic way
    pub fn from_name(name: &str) -> (ProtoOp, Anchor) {
        let (name, anchor) = if let Some(po_name) = name.strip_prefix("pre_") {
            (po_name, Anchor::Pre)
        } else if let Some(po_name) = name.strip_prefix("post_") {
            (po_name, Anchor::Post)
        } else {
            (name, Anchor::Replace)
        };

        if name == "init" {
            (ProtoOp::Init, anchor)
        } else if name.starts_with("decode_transport_parameter_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::DecodeTransportParameter(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_long_header_") {
            match extract_po_param(name) {
                Ok(version) => match u32::try_from(version) {
                    Ok(version) => (ProtoOp::ProcessLongHeader(version), anchor),
                    Err(_) => panic!("Invalid protocol operation name"),
                },
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_short_header_") {
            match extract_po_param(name) {
                Ok(version) => match u32::try_from(version) {
                    Ok(version) => (ProtoOp::ProcessShortHeader(version), anchor),
                    Err(_) => panic!("Invalid protocol operation name"),
                },
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name == "process_version_negotiation" {
            (ProtoOp::ProcessVersionNegotiation, anchor)
        } else if name.starts_with("write_transport_parameter_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::WriteTransportParameter(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("log_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::LogFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("notify_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::NotifyFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("on_frame_reserved_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::OnFrameReserved(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("parse_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::ParseFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("prepare_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::PrepareFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("process_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::ProcessFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("should_send_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::ShouldSendFrame(frame_type), anchor),
                Err(_) => panic!("Invalid protocol operation name"),
            }
        } else if name.starts_with("wire_len_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::WireLen(frame_type), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name.starts_with("write_frame_") {
            match extract_po_param(name) {
                Ok(frame_type) => (ProtoOp::WriteFrame(frame_type), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name.starts_with("plugin_control_") {
            match extract_po_param(name) {
                Ok(val) => (ProtoOp::PluginControl(val), anchor),
                Err(e) => panic!("Invalid protocol operation name: {e}"),
            }
        } else if name == "get_packet_to_send" {
            (ProtoOp::GetPacketToSend, anchor)
        } else if name == "decrypt_packet" {
            (ProtoOp::DecryptPacket, anchor)
        } else if name == "on_packet_processed" {
            (ProtoOp::OnPacketProcessed, anchor)
        } else if name == "on_packet_sent" {
            (ProtoOp::OnPacketSent, anchor)
        } else if name == "set_loss_detection_timer" {
            (ProtoOp::SetLossDetectionTimer, anchor)
        } else if name == "update_rtt" {
            (ProtoOp::UpdateRtt, anchor)
        } else {
            let mut name_array = [0; 32];
            name_array[..name.len()].copy_from_slice(name.as_bytes());
            (ProtoOp::Other(name_array), anchor)
        }
    }
}

/// A value, stored in the plugin memory, that can serve as argument or result of a protocol
/// operation called by the plugin.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum PluginVal {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl From<i32> for PluginVal {
    fn from(i: i32) -> Self {
        PluginVal::I32(i)
    }
}

impl From<i64> for PluginVal {
    fn from(i: i64) -> Self {
        PluginVal::I64(i)
    }
}

impl From<f32> for PluginVal {
    fn from(f: f32) -> Self {
        PluginVal::F32(f)
    }
}

impl From<f64> for PluginVal {
    fn from(f: f64) -> Self {
        PluginVal::F64(f)
    }
}

/// Inputs that can be passed to protocol operations.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Input {
    /// A duration.
    Duration(Duration),
    /// A specific instant in time relative to the UNIX epoch.
    Instant(Instant),
    /// A socket address.
    SocketAddr(SocketAddr),
    /// QUIC specific inputs.
    QUIC(quic::QInput),
}

impl From<Duration> for Input {
    fn from(d: Duration) -> Self {
        Self::Duration(d)
    }
}

impl TryFrom<Input> for Duration {
    type Error = ConversionError;

    fn try_from(value: Input) -> Result<Self, Self::Error> {
        match value {
            Input::Duration(d) => Ok(d),
            _ => Err(ConversionError::InvalidDuration),
        }
    }
}

impl From<Instant> for Input {
    fn from(i: Instant) -> Self {
        Self::Instant(i)
    }
}

impl TryFrom<Input> for Instant {
    type Error = ConversionError;

    fn try_from(value: Input) -> Result<Self, Self::Error> {
        match value {
            Input::Instant(i) => Ok(i),
            _ => Err(ConversionError::InvalidInstant),
        }
    }
}

impl From<SocketAddr> for Input {
    fn from(sa: SocketAddr) -> Self {
        Self::SocketAddr(sa)
    }
}

impl TryFrom<Input> for SocketAddr {
    type Error = ConversionError;

    fn try_from(value: Input) -> Result<Self, Self::Error> {
        match value {
            Input::SocketAddr(sa) => Ok(sa),
            _ => Err(ConversionError::InvalidSocketAddr),
        }
    }
}

pub mod quic;
