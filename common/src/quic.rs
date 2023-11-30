use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use unix_time::Instant as UnixInstant;

use crate::{Bytes, ConversionError, PluginVal};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum FrameSendKind {
    OncePerPacket,
    ManyPerPacket,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum FrameSendOrder {
    First,
    AfterACK,
    BeforeStream,
    End,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct FrameRegistration {
    ty: u64,
    send_order: FrameSendOrder,
    send_kind: FrameSendKind,
    ack_eliciting: bool,
    count_in_flight: bool,
}

impl FrameRegistration {
    pub fn new(
        ty: u64,
        send_order: FrameSendOrder,
        send_kind: FrameSendKind,
        ack_eliciting: bool,
        count_in_flight: bool,
    ) -> Self {
        Self {
            ty,
            send_order,
            send_kind,
            ack_eliciting,
            count_in_flight,
        }
    }

    pub fn get_type(&self) -> u64 {
        self.ty
    }

    pub fn send_order(&self) -> FrameSendOrder {
        self.send_order
    }

    pub fn ack_eliciting(&self) -> bool {
        self.ack_eliciting
    }

    pub fn count_for_in_flight(&self) -> bool {
        self.count_in_flight
    }
}

/// A request from the plugin at initialization time.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum Registration {
    TransportParameter(u64),
    Frame(FrameRegistration),
}

/// QUIC packet type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum PacketType {
    /// Initial packet.
    Initial,

    /// Retry packet.
    Retry,

    /// Handshake packet.
    Handshake,

    /// 0-RTT packet.
    ZeroRTT,

    /// Version negotiation packet.
    VersionNegotiation,

    /// 1-RTT short header packet.
    Short,
}

/// An enum to enumerate the three packet number spaces, as defined by Section
/// A.2 of quic-recovery.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(usize)]
pub enum KPacketNumberSpace {
    Initial = 0,
    Handshake = 1,
    ApplicationData = 2,
}

/// A QUIC packet number.
pub type PacketNumber = u64;

/// Fields for the Recovery as defined by quic-recovery, Section A.3. These
/// fields also include the congestion control ones, as defined by
/// quic-recovery, Section B.2.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum RecoveryField {
    /// The most recent RTT measurement made when receiving an ack for a
    /// previously unacked packet.
    LatestRtt,
    /// The smoothed RTT of the connection, computed as described in Section
    /// 5.3 of quic-recovery.
    SmoothedRtt,
    /// The RTT variation, computed as described in Section 5.3 of
    /// quic-recovery.
    Rttvar,
    /// The minimum RTT seen in the connection, ignoring acknowledgment delay,
    /// as described in Section 5.2 of quic-recovery.
    MinRtt,
    /// The time that the first RTT sample was obtained.
    FirstRttSample,
    /// The maximum amount of time by which the receiver intends to delay
    /// acknowledgments for packets in the Application Data packet number
    /// space, as defined by the eponymous transport parameter (Section
    /// 18.2 of [QUIC-TRANSPORT]).  Note that the actual ack_delay in a
    /// received ACK frame may be larger due to late timers, reordering,
    /// or loss.
    MaxAckDelay,
    /// Multi-modal timer used for loss detection.
    LossDetectionTimer,
    /// The number of times a PTO has been sent without receiving an ack.
    /// Unlike specified in the recovery draft, this value is returned on a
    /// per-epoch basis. Returns a `usize`.
    PtoCount(KPacketNumberSpace),
    /// The time the most recent ack-eliciting packet was sent.
    TimeOfLastAckElicitingPacket(KPacketNumberSpace),
    /// The largest packet number acknowledged in the packet number space so
    /// far.
    LargestAckedPacket(KPacketNumberSpace),
    /// The time at which the next packet inthat packet number space will be
    /// considered lost based on exceeding the reordering window in time.
    LossTime(KPacketNumberSpace),
    /// An association of packet numbers in a packet number space to
    /// information about them. Described in detail above in Appendix A.1 of
    /// quic-recovery.
    SentPackets(KPacketNumberSpace, PacketNumber),
    /// The sender's current maximum payload size. Does not include UDP or IP
    /// overhead. The max datagram size is used for congestion window
    /// computations. An endpoint sets the value of this variable based on its
    /// Path Maximum Transmission Unit (PMTU; see Section 14.2 of
    /// [QUIC-TRANSPORT]), with a minimum value of 1200 bytes.
    MaxDatagramSize,
    /// The highest value reported for the ECN-CE counter in the packet number
    /// space by the peer in an ACK frame. This value is used to detect
    /// increases in the reported ECN-CE counter.
    EcnCeCounters(KPacketNumberSpace),
    /// The sum of the size in bytes of all sent packets that contain at least
    /// one ack-eliciting or PADDING frame, and have not been acknowledged or
    /// declared lost. The size does not include IP or UDP overhead, but does
    /// include the QUIC header and AEAD overhead. Packets only containing ACK
    /// frames do not count towards bytes_in_flight to ensure congestion control
    /// does not impede congestion feedback.
    BytesInFlight,
    /// Maximum number of bytes-in-flight that may be sent.
    CongestionWindow,
    /// The time when QUIC first detects congestion due to loss or ECN, causing
    /// it to enter congestion recovery. When a packet sent after this time is
    /// acknowledged, QUIC exits congestion recovery.
    CongestionRecoveryStartTime,
    /// Slow start threshold in bytes. When the congestion window is below
    /// ssthresh, the mode is slow start and the window grows by the number of
    /// bytes acknowledged.
    Ssthresh,
}

/// Field of a range set.
///
/// WARNING: unstable API.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum RangeSetField {
    /// The length of the range set as a `usize`.
    Length,
}

/// Field of a packet number space.
///
/// WARNING: unstable API.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum PacketNumberSpaceField {
    /// Whether there is unacknowledged received packets, as a `bool`.
    ReceivedPacketNeedAck,
    /// Boolean indicating if a ACK frame must be sent.
    AckEllicited,
    /// The next packet number to be sent, as a `u64`.
    NextPacketNumber,
    /// Indicates if this packet number space has the keys to send packets over it, as a `bool`.
    HasSendKeys,
    /// Indicates if this packet number space must send some data, as a `bool`.
    ShouldSend,
    /// The largest packet number being received, as a `u64`.
    LargestRxPacketNumber,
}

/// Indicates if the info is local or remote.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum Host {
    Local,
    Remote,
}

/// Indicates if the info is about source or destination.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum Direction {
    Source,
    Destination,
}

/// Classical transport parameters.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum TransportParameterField {
    AckDelayExponent,
}

/// A subfield used when a collection of elements using a monotonically increasing ID.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum IDList {
    /// The length of the `IDList`. This is a `u64`.
    Length,
    /// The minimum ID used in the `IDList´. This is a `u64`.
    MinID,
    /// The maximum ID used in the `IDList`. This is a `u64`.
    MaxID,
    /// The element with index `ID`. When getting this, returns a `Option<Elem>`.
    Elem(u64),
    /// Get all elements, regardless of their ID. Returns a `Vec<Elem>`.
    All,
}

/// A QUIC address available over the connection.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Address {
    /// The value of the address.
    pub addr: SocketAddr,
    /// Is this address local?
    pub local: bool,
    /// If it is local, is this address verified by the peer? If it is remote, was
    /// this address verified?
    pub verified: bool,
}

/// Field of a connection.
///
/// WARNING: changing API.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum ConnectionField {
    /// Boolean indicating if this is a server-side connection.
    IsServer,
    /// An `Option<u64>` being an internal identifier of this connection. Might be `None`.
    InternalID,
    /// The version used by this connection, as a `u32`.
    Version,
    /// Peer's flow control limit for the connection.
    MaxTxData,
    /// Connection IDs associated to this connection. The ID corresponds to the sequence number.
    ConnectionID(Direction, IDList),
    /// Packet number space.
    PacketNumberSpace(KPacketNumberSpace, PacketNumberSpaceField),
    /// Exchanged transport parameters.
    TransportParameter(Host, TransportParameterField),
    /// The token used over this connection, as an `Option<Vec<u8>>`.
    Token,
    /// The connection error code, if any, as an `Option<u64>`.
    ConnectionError,
    /// The handshake write level, as a `i32`.
    /// TODO FIXME: this should probably move in a crypto field.
    HandshakeWriteLevel,
    /// Indicates if the handshake completed, as a `bool`.
    IsEstablished,
    /// Indicates if the connection is in the early data, as a `bool`.
    IsInEarlyData,
    /// Indicates if the connection is blocked by the connection-level flow limit, as a `bool`.
    IsBlocked,
    /// Indicates if the connection has flushable streams, as a `bool`.
    HasFlushableStreams,
    /// Indicates if the connection has blocked streams, as a `bool`.
    HasBlockedStreams,
    /// Returns the maximum length of a packet to be sent as a `u64`.
    MaxSendUdpPayloadLength,
    /// Returns the maximum number of bytes the server can send without creating amplification
    /// attacks, as a `u64`.
    MaxSendBytes,
    /// The addresses associated to the connection, as `Address`.
    Address(Host, IDList),
    /// Total number of bytes received from the peer, as a `u64`.
    RxData,
}

/// Fields of the SentPacket as defined by quic-recovery, Section A.1.1. Compared to the
/// specification, additional fields are present.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum SentPacketField {
    /// The packet number of the sent packet.
    PacketNumber,
    /// A boolean that indicates whether a packet is ack-eliciting.  If true,
    /// it is expected that an acknowledgement will be received, though the
    /// peer could delay sending the ACK frame containing it by up to the
    /// max_ack_delay.
    AckEliciting,
    /// A boolean that indicates whether the packet counts towards bytes in
    /// flight.
    InFlight,
    /// The number of bytes sent in the packet, not including UDP or IP
    /// overhead, but including QUIC framing overhead.
    SentBytes,
    /// The time the packet was sent, as a `Instant`.
    TimeSent,
    /// The source address used by this packet, as a `SocketAddr`.
    SourceAddress,
    /// The destination address used by this packet, as a `SocketAddr´.
    DestinationAddress,
}

/// Fields of the RcvPacket, only available in receiving workflow.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub enum RcvPacketField {
    /// The source address contained in this packet, as a `SocketAddr`.
    SourceAddress,
    /// The destination address contained in this packet, as a `SocketAddr`.
    DestinationAddress,
}

/// Some additional fields that may be present in QUICv1, but are not ensure to be always
/// present.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct HeaderExt {
    /// The packet number.
    pub packet_number: Option<u64>,
    /// The packet number length.
    pub packet_number_len: Option<u8>,
    /// The address verification token of the packet. Only present in `Initial` and
    /// `Retry` packets.
    pub token: Option<Bytes>,
    /// The key phase of the packet.
    pub key_phase: Option<bool>,
}

/// A QUIC packet header structure, as close as it is encoded on the wire.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct Header {
    /// The first byte of the header, defining its type + version-specific bits.
    pub first: u8,
    /// The version contained in the header, if any. Is 0 for version negotiation packets.
    pub version: Option<u32>,
    /// The destination connection ID. A 0-length connection ID is Some of an empty Vec.
    pub destination_cid: Bytes,
    /// The source connection ID. A 0-length connection ID is Some of an empty Vec, absence of
    /// source connection ID is None.
    pub source_cid: Option<Bytes>,
    /// Supported version, only present in a Version Negotiation packet. Should represents a
    /// `Vec<u32>`.
    pub supported_versions: Option<Bytes>,
    /// Additional fields that are not guaranteed to stay in the invariants. The host implementation
    /// may provide some information here for received packets, but it is not mandatory. All fields
    /// being part of the header but requiring decryption are put there. Hence, prior to decryption
    /// process, this field may contain meaningless information. The main use of this field is for
    /// the sending of packets.
    pub ext: Option<HeaderExt>,
}

/// A SentPacket as defined by quic-recovery, Section A.1.1. This has the difference that this
/// structure is not only used for recovery purpose, but also during the whole process of packet
/// sending.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct SentPacket {
    /// The header of the packet being sent.
    pub header: Header,
    /// The source address used by this packet.
    pub source_address: SocketAddr,
    /// The destination address used by this packet.
    pub destination_address: SocketAddr,
    /// The packet number of the sent packet.
    pub packet_number: u64,
    /// The length of the packet number endoded in the header.
    pub packet_number_len: u8,
    /// Is this packet ack-elliciting? If true, it is expected that an acknowledgment with be
    /// received, though the peer could delay sending the ACK frame containing it up to the
    /// max_ack_delay.
    pub ack_elliciting: bool,
    /// Does this packet counts towards bytes in flight?
    pub in_flight: bool,
    /// The number of bytes sent in this packet, not including UDP or IP overhead, but including
    /// QUIC framing overhead.
    pub sent_bytes: usize,
    /// The time the packet was sent, relative to the beginning of the session.
    pub time_sent: UnixInstant,
}

/// Each ACK Range consists of alternating Gap and ACK Range values in descending packet number
/// order. ACK Ranges can be repeated. The number of Gap and ACK Range values is determined by the
/// ACK Range Count field; one of each value is present for each value in the ACK Range Count field.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct AckRange {
    /// A variable-length integer indicating the number of contiguous unacknowledged packets
    /// preceding the packet number one lower than the smallest in the preceding ACK Range.
    pub gap: u64,
    /// A variable-length integer indicating the number of contiguous acknowledged packets
    /// preceding the largest packet number, as determined by the preceding Gap.
    pub ack_range_length: u64,
}

/// The ACK frame uses the least significant bit (that is, type 0x03) to indicate ECN feedback and
/// report receipt of QUIC packets with associated ECN codepoints of ECT(0), ECT(1), or CE in the
/// packet's IP header. ECN Counts are only present when the ACK frame type is 0x03.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct EcnCount {
    /// A variable-length integer representing the total number of packets received with the ECT(0)
    /// codepoint in the packet number space of the ACK frame.
    pub ect0_count: u64,
    /// A variable-length integer representing the total number of packets received with the ECT(1)
    /// codepoint in the packet number space of the ACK frame.
    pub ect1_count: u64,
    /// A variable-length integer representing the total number of packets received with the CE
    /// codepoint in the packet number space of the ACK frame.
    pub ectce_count: u64,
}

/// A Connection ID structure, exposed to the plugin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct ConnectionId {
    /// The sequence number assigned to the connection ID by the sender, encoded as a
    /// variable-length integer.
    pub sequence_number: u64,
    /// The raw value of the connection ID. Its length can be obtained using the `len()` method.
    pub connection_id: Vec<u8>,
    /// An associated stateless reset token.
    pub stateless_reset_token: Option<Vec<u8>>,
}

/// A QUIC frame structure, as close as it is encoded on the wire.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[non_exhaustive]
#[repr(C)]
pub enum Frame {
    Padding(PaddingFrame),
    Ping(PingFrame),
    ACK(ACKFrame),
    ResetStream(ResetStreamFrame),
    StopSending(StopSendingFrame),
    Crypto(CryptoFrame),
    NewToken(NewTokenFrame),
    Stream(StreamFrame),
    MaxData(MaxDataFrame),
    MaxStreamData(MaxStreamDataFrame),
    MaxStreams(MaxStreamsFrame),
    DataBlocked(DataBlockedFrame),
    StreamDataBlocked(StreamDataBlockedFrame),
    StreamsBlocked(StreamsBlockedFrame),
    NewConnectionId(NewConnectionIdFrame),
    RetireConnectionId(RetireConnectionIdFrame),
    PathChallenge(PathChallengeFrame),
    PathResponse(PathResponseFrame),
    ConnectionClose(ConnectionCloseFrame),
    HandshakeDone(HandshakeDoneFrame),
    Extension(ExtensionFrame),
}

/// A PADDING frame (type=0x00) has no semantic value. PADDING frames can be used to increase
/// the size of a packet. Padding can be used to increase an initial client packet to the
/// minimum required size, or to provide protection against traffic analysis for protected
/// packets.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct PaddingFrame {
    /// The number of consecutive padding frames put together.
    pub length: u64,
}

/// Endpoints can use PING frames (type=0x01) to verify that their peers are still alive or to
/// check reachability to the peer.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct PingFrame;

/// Receivers send ACK frames (types 0x02 and 0x03) to inform senders of packets they have
/// received and processed. The ACK frame contains one or more ACK Ranges. ACK Ranges identify
/// acknowledged packets. If the frame type is 0x03, ACK frames also contain the sum of QUIC
/// packets with associated ECN marks received on the connection up until this point. QUIC
/// implementations MUST properly handle both types and, if they have enabled ECN for packets
/// they send, they SHOULD use the information in the ECN section to manage their congestion
/// state.
///
/// QUIC acknowledgements are irrevocable. Once acknowledged, a packet remains acknowledged,
/// even if it does not appear in a future ACK frame. This is unlike reneging for TCP SACKs
/// (see \[RFC2018\]).
///
/// Packets from different packet number spaces can be identified using the same numeric value.
/// An acknowledgment for a packet needs to indicate both a packet number and a packet number
/// space. This is accomplished by having each ACK frame only acknowledge packet numbers in the
/// same space as the packet in which the ACK frame is contained.
///
/// Version Negotiation and Retry packets cannot be acknowledged because they do not contain a
/// packet number. Rather than relying on ACK frames, these packets are implicitly acknowledged
/// by the next Initial packet sent by the client.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct ACKFrame {
    /// A variable-length integer representing the largest packet number the peer is
    /// acknowledging; this is usually the largest packet number that the peer has received
    /// prior to generating the ACK frame. Unlike the packet number in the QUIC long or short
    /// header, the value in an ACK frame is not truncated.
    pub largest_acknowledged: u64,
    /// A variable-length integer encoding the acknowledgement delay in microseconds; see
    /// Section 13.2.5. It is decoded by multiplying the value in the field by 2 to the power
    /// of the ack_delay_exponent transport parameter sent by the sender of the ACK frame; see
    /// Section 18.2. Compared to simply expressing the delay as an integer, this encoding
    /// allows for a larger range of values within the same number of bytes, at the cost of
    /// lower resolution.
    pub ack_delay: u64,
    /// A variable-length integer specifying the number of Gap and ACK Range fields in the
    /// frame.
    pub ack_range_count: u64,
    /// A variable-length integer indicating the number of contiguous packets preceding the
    /// Largest Acknowledged that are being acknowledged. The First ACK Range is encoded as an
    /// ACK Range; see Section 19.3.1 starting from the Largest Acknowledged. That is, the
    /// smallest packet acknowledged in the range is determined by subtracting the First ACK
    /// Range value from the Largest Acknowledged.
    pub first_ack_range: u64,
    /// Contains additional ranges of packets that are alternately not acknowledged (Gap) and
    /// acknowledged (ACK Range).
    pub ack_ranges: Bytes,
    /// The three ECN Counts.
    pub ecn_counts: Option<EcnCount>,
}

/// An endpoint uses a RESET_STREAM frame (type=0x04) to abruptly terminate the sending part of
/// a stream.
///
/// After sending a RESET_STREAM, an endpoint ceases transmission and retransmission of STREAM
/// frames on the identified stream. A receiver of RESET_STREAM can discard any data that it
/// already received on that stream.An endpoint that receives a RESET_STREAM frame for a
/// send-only stream MUST terminate the connection with error STREAM_STATE_ERROR.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct ResetStreamFrame {
    /// A variable-length integer encoding of the Stream ID of the stream being terminated.
    pub stream_id: u64,
    /// A variable-length integer containing the application protocol error code that indicates
    /// why the stream is being closed.
    pub application_protocol_error_code: u64,
    /// A variable-length integer indicating the final size of the stream by the RESET_STREAM
    /// sender, in unit of bytes.
    pub final_size: u64,
}

/// An endpoint uses a STOP_SENDING frame (type=0x05) to communicate that incoming data is
/// being discarded on receipt at application request. STOP_SENDING requests that a peer cease
/// transmission on a stream.
///
/// A STOP_SENDING frame can be sent for streams in the Recv or Size Known states; see Section
/// 3.1. Receiving a STOP_SENDING frame for a locally-initiated stream that has not yet been
/// created MUST be treated as a connection error of type STREAM_STATE_ERROR. An endpoint that
/// receives a STOP_SENDING frame for a receive-only stream MUST terminate the connection with
/// error STREAM_STATE_ERROR.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct StopSendingFrame {
    /// A variable-length integer carrying the Stream ID of the stream being ignored.
    pub stream_id: u64,
    /// A variable-length integer containing the application-specified reason the sender is
    /// ignoring the stream.
    pub application_protocol_error_code: u64,
}

/// A CRYPTO frame (type=0x06) is used to transmit cryptographic handshake messages. It can be
/// sent in all packet types except 0-RTT. The CRYPTO frame offers the cryptographic protocol
/// an in-order stream of bytes. CRYPTO frames are functionally identical to STREAM frames,
/// except that they do not bear a stream identifier; they are not flow controlled; and they do
/// not carry markers for optional offset, optional length, and the end of the stream.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct CryptoFrame {
    /// A variable-length integer specifying the byte offset in the stream for the data in this
    /// CRYPTO frame.
    pub offset: u64,
    /// A variable-length integer specifying the length of the Crypto Data field in this CRYPTO
    /// frame.
    pub length: u64,
    /// The cryptographic message data.
    pub crypto_data: Bytes,
}

/// A server sends a NEW_TOKEN frame (type=0x07) to provide the client with a token to send in
/// the header of an Initial packet for a future connection.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct NewTokenFrame {
    /// A variable-length integer specifying the length of the token in bytes.
    pub token_length: u64,
    /// An opaque blob that the client can use with a future Initial packet. The token MUST NOT
    /// be empty. A client MUST treat receipt of a NEW_TOKEN frame with an empty Token field as
    /// a connection error of type FRAME_ENCODING_ERROR.
    pub token: Bytes,
}

/// STREAM frames implicitly create a stream and carry stream data. The STREAM frame Type field
/// takes the form 0b00001XXX (or the set of values from 0x08 to 0x0f). The three low-order
/// bits of the frame type determine the fields that are present in the frame:
/// - The OFF bit (0x04) in the frame type is set to indicate that there is an Offset field
///   present. When set to 1, the Offset field is present. When set to 0, the Offset field is
///   absent and the Stream Data starts at an offset of 0 (that is, the frame contains the
///   first bytes of the stream, or the end of a stream that includes no data).
/// - The LEN bit (0x02) in the frame type is set to indicate that there is a Length field
///   present. If this bit is set to 0, the Length field is absent and the Stream Data field
///   extends to the end of the packet. If this bit is set to 1, the Length field is present.
/// - The FIN bit (0x01) indicates that the frame marks the end of the stream. The final size
///   of the stream is the sum of the offset and the length of this frame.
///
/// An endpoint MUST terminate the connection with error STREAM_STATE_ERROR if it receives a
/// STREAM frame for a locally-initiated stream that has not yet been created, or for a
/// send-only stream.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct StreamFrame {
    /// A variable-length integer indicating the stream ID of the stream.
    pub stream_id: u64,
    /// A variable-length integer specifying the byte offset in the stream for the data in this
    /// STREAM frame. This field is present when the OFF bit is set to 1. When the Offset field
    /// is absent, the offset is 0.
    pub offset: Option<u64>,
    /// A variable-length integer specifying the length of the Stream Data field in this STREAM
    /// frame. This field is present when the LEN bit is set to 1. When the LEN bit is set to
    /// 0, the Stream Data field consumes all the remaining bytes in the packet.
    pub length: Option<u64>,
    /// Indicates that the frame marks the end of the stream. The final size of the stream is
    /// the sum of the offset and the length of this frame.
    pub fin: bool,
    /// The bytes from the designated stream to be delivered.
    pub stream_data: Bytes,
}

/// A MAX_DATA frame (type=0x10) is used in flow control to inform the peer of the maximum
/// amount of data that can be sent on the connection as a whole.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct MaxDataFrame {
    /// A variable-length integer indicating the maximum amount of data that can be sent on the
    /// entire connection, in units of bytes.
    pub maximum_data: u64,
}

/// A MAX_STREAM_DATA frame (type=0x11) is used in flow control to inform a peer of the maximum
/// amount of data that can be sent on a stream.
///
/// A MAX_STREAM_DATA frame can be sent for streams in the Recv state. Receiving a
/// MAX_STREAM_DATA frame for a locally-initiated stream that has not yet been created MUST be
/// treated as a connection error of type STREAM_STATE_ERROR. An endpoint that receives a
/// MAX_STREAM_DATA frame for a receive-only stream MUST terminate the connection with error
/// STREAM_STATE_ERROR.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct MaxStreamDataFrame {
    /// The stream ID of the stream that is affected encoded as a variable-length integer.
    pub stream_id: u64,
    /// A variable-length integer indicating the maximum amount of data that can be sent on the
    /// identified stream, in units of bytes.
    pub maximum_stream_data: u64,
}

/// A MAX_STREAMS frame (type=0x12 or 0x13) inform the peer of the cumulative number of streams
/// of a given type it is permitted to open. A MAX_STREAMS frame with a type of 0x12 applies to
/// bidirectional streams, and a MAX_STREAMS frame with a type of 0x13 applies to
/// unidirectional streams.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct MaxStreamsFrame {
    /// Indicates if this frame concerns unidirectional streams (type=0x13) or bidirectional
    /// streams (type=0x12).
    pub unidirectional: bool,
    /// A count of the cumulative number of streams of the corresponding type that can be
    /// opened over the lifetime of the connection. This value cannot exceed 2^60, as it is not
    /// possible to encode stream IDs larger than 2^62-1. Receipt of a frame that permits
    /// opening of a stream larger than this limit MUST be treated as a FRAME_ENCODING_ERROR.
    pub maximum_streams: u64,
}

/// A sender SHOULD send a DATA_BLOCKED frame (type=0x14) when it wishes to send data, but is
/// unable to do so due to connection-level flow control. DATA_BLOCKED frames can be used as
/// input to tuning of flow control algorithms.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct DataBlockedFrame {
    /// A variable-length integer indicating the connection-level limit at which blocking
    /// occurred.
    pub maximum_data: u64,
}

/// A sender SHOULD send a STREAM_DATA_BLOCKED frame (type=0x15) when it wishes to send data,
/// but is unable to do so due to stream-level flow control. This frame is analogous to
/// DATA_BLOCKED.
///
/// An endpoint that receives a STREAM_DATA_BLOCKED frame for a send-only stream MUST terminate
/// the connection with error STREAM_STATE_ERROR.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct StreamDataBlockedFrame {
    /// A variable-length integer indicating the stream that is blocked due to flow control.
    pub stream_id: u64,
    /// A variable-length integer indicating the offset of the stream at which the blocking
    /// occurred.
    pub maximum_stream_data: u64,
}

/// A sender SHOULD send a STREAMS_BLOCKED frame (type=0x16 or 0x17) when it wishes to open a
/// stream, but is unable to due to the maximum stream limit set by its peer. A STREAMS_BLOCKED
/// frame of type 0x16 is used to indicate reaching the bidirectional stream limit, and a
/// STREAMS_BLOCKED frame of type 0x17 is used to indicate reaching the unidirectional stream
/// limit.
///
/// A STREAMS_BLOCKED frame does not open the stream, but informs the peer that a new stream
/// was needed and the stream limit prevented the creation of the stream.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct StreamsBlockedFrame {
    /// Indicates if this frame concerns unidirectional streams (type=0x17) or bidirectional
    /// streams (type=0x16).
    pub unidirectional: bool,
    /// A variable-length integer indicating the maximum number of streams allowed at the time
    /// the frame was sent. This value cannot exceed 2^60, as it is not possible to encode
    /// stream IDs larger than 2^62-1. Receipt of a frame that encodes a larger stream ID MUST
    /// be treated as a STREAM_LIMIT_ERROR or a FRAME_ENCODING_ERROR.
    pub maximum_streams: u64,
}

/// An endpoint sends a NEW_CONNECTION_ID frame (type=0x18) to provide its peer with
/// alternative connection IDs that can be used to break linkability when migrating
/// connections.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct NewConnectionIdFrame {
    /// The sequence number assigned to the connection ID by the sender, encoded as a
    /// variable-length integer.
    pub sequence_number: u64,
    /// A variable-length integer indicating which connection IDs should be retired.
    pub retire_prior_to: u64,
    /// An 8-bit unsigned integer containing the length of the connection ID. Values less than
    /// 1 and greater than 20 are invalid and MUST be treated as a connection error of type
    /// FRAME_ENCODING_ERROR.
    pub length: u8,
    /// A connection ID of the specified length.
    pub connection_id: Bytes,
    /// A 128-bit value that will be used for a stateless reset when the associated connection
    /// ID is used. Probably easier to manipulate as a `Vec<u8>`.
    pub stateless_reset_token: Bytes,
}

/// An endpoint sends a RETIRE_CONNECTION_ID frame (type=0x19) to indicate that it will no
/// longer use a connection ID that was issued by its peer. This includes the connection ID
/// provided during the handshake. Sending a RETIRE_CONNECTION_ID frame also serves as a
/// request to the peer to send additional connection IDs for future use. New connection IDs
/// can be delivered to a peer using the NEW_CONNECTION_ID frame.
///
/// Retiring a connection ID invalidates the stateless reset token associated with that
/// connection ID.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct RetireConnectionIdFrame {
    /// The sequence number of the connection ID being retired.
    pub sequence_number: u64,
}

/// Endpoints can use PATH_CHALLENGE frames (type=0x1a) to check reachability to the peer and
/// for path validation during connection migration.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct PathChallengeFrame {
    /// This 8-byte field contains arbitrary data.
    pub data: u64,
}

/// A PATH_RESPONSE frame (type=0x1b) is sent in response to a PATH_CHALLENGE frame.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct PathResponseFrame {
    /// This 8-byte field contains arbitrary data.
    pub data: u64,
}

/// An endpoint sends a CONNECTION_CLOSE frame (type=0x1c or 0x1d) to notify its peer that the
/// connection is being closed. The CONNECTION_CLOSE with a frame type of 0x1c is used to
/// signal errors at only the QUIC layer, or the absence of errors (with the NO_ERROR code).
/// The CONNECTION_CLOSE frame with a type of 0x1d is used to signal an error with the
/// application that uses QUIC.
///
/// If there are open streams that have not been explicitly closed, they are implicitly closed
/// when the connection is closed.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct ConnectionCloseFrame {
    /// A variable-length integer error code that indicates the reason for closing this
    /// connection. A CONNECTION_CLOSE frame of type 0x1c uses codes from the space. A
    /// CONNECTION_CLOSE frame of type 0x1d uses codes from the application protocol error code
    /// space.
    pub error_code: u64,
    /// A variable-length integer encoding the type of frame that triggered the error. A value
    /// of 0 (equivalent to the mention of the PADDING frame) is used when the frame type is
    /// unknown. The application-specific variant of CONNECTION_CLOSE (type 0x1d) does not
    /// include this field.
    pub frame_type: Option<u64>,
    /// A variable-length integer specifying the length of the reason phrase in bytes. Because
    /// a CONNECTION_CLOSE frame cannot be split between packets, any limits on packet size
    /// will also limit the space available for a reason phrase.
    pub reason_phrase_length: u64,
    /// A human-readable explanation for why the connection was closed. This can be zero length
    /// if the sender chooses not to give details beyond the Error Code. This SHOULD be a UTF-8
    /// encoded string \[RFC3629\].
    pub reason_phrase: Bytes,
}

/// The server uses a HANDSHAKE_DONE frame (type=0x1e) to signal confirmation of the handshake
/// to the client.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct HandshakeDoneFrame;

/// QUIC frames do not use a self-describing encoding. An endpoint therefore needs to
/// understand the syntax of all frames before it can successfully process a packet. This
/// allows for efficient encoding of frames, but it means that an endpoint cannot send a frame
/// of a type that is unknown to its peer.
///
/// An extension to QUIC that wishes to use a new type of frame MUST first ensure that a peer
/// is able to understand the frame. An endpoint can use a transport parameter to signal its
/// willingness to receive extension frame types. One transport parameter can indicate support
/// for one or more extension frame types.
///
/// Extensions that modify or replace core protocol functionality (including frame types) will
/// be difficult to combine with other extensions that modify or replace the same functionality
/// unless the behavior of the combination is explicitly defined. Such extensions SHOULD define
/// their interaction with previously-defined extensions modifying the same protocol
/// components.
///
/// Extension frames MUST be congestion controlled and MUST cause an ACK frame to be sent. The
/// exception is extension frames that replace or supplement the ACK frame. Extension frames
/// are not included in flow control unless specified in the extension.
///
/// An IANA registry is used to manage the assignment of frame types
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub struct ExtensionFrame {
    /// The corresponding frame type of the extension frame.
    pub frame_type: u64,
    /// The content of the frame is opaque to the host implementation. All the frame-specific
    /// fields are maintained by the plugin itself. The tag enables the plugin to retrieve such
    /// information.
    pub tag: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
/// Network-layer information about the packet being received.
pub struct RcvInfo {
    /// The source address of the received packet.
    pub from: SocketAddr,
    /// The destination address of the received packet.
    pub to: SocketAddr,
}

/// Inputs that can be passed to protocol operations for the QUIC protocol.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd)]
#[repr(C)]
pub enum QVal {
    /// The QUIC Header.
    Header(Header),
    /// The QUIC Frame.
    Frame(Frame),
    /// Reception information.
    RcvInfo(RcvInfo),
    /// Packet number space.
    PacketNumberSpace(KPacketNumberSpace),
    // Packet type.
    PacketType(PacketType),
    // /// The next packet to be sent.
    // SentPacket(SentPacket),
}

macro_rules! impl_from_try_from_qval {
    ($e:ident, $v:ident, $t:ty, $err:ident, $verr:ident) => {
        impl From<$t> for $e {
            fn from(v: $t) -> Self {
                $e::QUIC(QVal::$v(v))
            }
        }

        impl TryFrom<$e> for $t {
            type Error = $err;

            fn try_from(v: $e) -> Result<Self, Self::Error> {
                match v {
                    $e::QUIC(QVal::$v(v)) => Ok(v),
                    _ => Err($err::$verr),
                }
            }
        }
    };
}

impl_from_try_from_qval!(PluginVal, Header, Header, ConversionError, InvalidQVal);
impl_from_try_from_qval!(PluginVal, Frame, Frame, ConversionError, InvalidQVal);
impl_from_try_from_qval!(PluginVal, RcvInfo, RcvInfo, ConversionError, InvalidQVal);
impl_from_try_from_qval!(
    PluginVal,
    PacketNumberSpace,
    KPacketNumberSpace,
    ConversionError,
    InvalidQVal
);
impl_from_try_from_qval!(
    PluginVal,
    PacketType,
    PacketType,
    ConversionError,
    InvalidQVal
);

// impl From<Header> for Input {
//     fn from(h: Header) -> Self {
//         Self::QUIC(QVal::Header(h))
//     }
// }

// impl TryFrom<Input> for Header {
//     type Error = ConversionError;

//     fn try_from(value: Input) -> Result<Self, Self::Error> {
//         match value {
//             Input::QUIC(QVal::Header(h)) => Ok(h),
//             _ => Err(ConversionError::InvalidHeader),
//         }
//     }
// }

// impl From<Frame> for Input {
//     fn from(f: Frame) -> Self {
//         Self::QUIC(QVal::Frame(f))
//     }
// }

// impl TryFrom<Input> for Frame {
//     type Error = ConversionError;

//     fn try_from(value: Input) -> Result<Self, Self::Error> {
//         match value {
//             Input::QUIC(QVal::Frame(f)) => Ok(f),
//             _ => Err(ConversionError::InvalidFrame),
//         }
//     }
// }

// impl From<SentPacket> for Input {
//     fn from(sp: SentPacket) -> Self {
//         Input::QUIC(QVal::SentPacket(sp))
//     }
// }

// impl TryFrom<Input> for SentPacket {
//     type Error = ConversionError;

//     fn try_from(value: Input) -> Result<Self, Self::Error> {
//         match value {
//             Input::QUIC(QVal::SentPacket(sp)) => Ok(sp),
//             _ => Err(ConversionError::InvalidSentPacket),
//         }
//     }
// }
