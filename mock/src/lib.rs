use std::ops::{Deref, DerefMut};
use std::time::Duration;

use pluginop::api::{ConnectionToPlugin, ToPluginizableConnection};
use pluginop::common::quic::{self, Frame, Registration};
use pluginop::common::{
    quic::{ConnectionField, RecoveryField},
    PluginVal,
};
use pluginop::common::{Bytes, PluginOp};
use pluginop::octets::{Octets, OctetsMut};
use pluginop::plugin::Env;
use pluginop::pluginop_macro::{pluginop, pluginop_param, pluginop_result, pluginop_result_param};
use pluginop::{api::CTPError, ParentReferencer, PluginizableConnection};
use unix_time::Instant;
use wasmer::{Exports, FunctionEnv, Store};

/// Dummy object
pub struct ConnectionDummy {
    pc: Option<ParentReferencer<PluginizableConnection<Self>>>,
    pub max_tx_data: u64,
    pub srtt: Duration,
}

impl ConnectionToPlugin for ConnectionDummy {
    fn get_recovery(&self, _: &mut [u8], _: RecoveryField) -> bincode::Result<()> {
        todo!()
    }

    fn set_recovery(&mut self, _: RecoveryField, _: &[u8]) {
        todo!()
    }

    fn get_connection(&self, field: ConnectionField, w: &mut [u8]) -> bincode::Result<()> {
        let pv: PluginVal = match field {
            ConnectionField::MaxTxData => self.max_tx_data.into(),
            _ => todo!(),
        };
        bincode::serialize_into(w, &pv)
    }

    fn set_connection(&mut self, field: ConnectionField, r: &[u8]) -> Result<(), CTPError> {
        let pv: PluginVal = bincode::deserialize_from(r).map_err(|_| CTPError::SerializeError)?;
        match field {
            ConnectionField::MaxTxData => {
                self.max_tx_data = pv.try_into().map_err(|_| CTPError::BadType)?
            }
            _ => todo!(),
        };
        Ok(())
    }
}

impl ToPluginizableConnection<ConnectionDummy> for ConnectionDummy {
    fn set_pluginizable_connection(&mut self, pc: *mut PluginizableConnection<Self>) {
        self.pc = Some(ParentReferencer::new(pc));
    }

    fn get_pluginizable_connection(&mut self) -> Option<&mut PluginizableConnection<Self>> {
        self.pc.as_deref_mut()
    }
}

#[derive(Debug)]
pub struct Error;

pub enum MyResult<T> {
    Ok(T),
    Err(Error),
}

impl From<i64> for Error {
    fn from(_: i64) -> Self {
        Error
    }
}

impl ConnectionDummy {
    #[pluginop(PluginOp::UpdateRtt)]
    fn update_rtt(&mut self, latest_rtt: Duration, _ack_delay: Duration, _now: Instant) {
        self.srtt = latest_rtt;
    }

    #[pluginop(PluginOp::Test)]
    fn test1(&mut self, _latest_rtt: Duration) -> u64 {
        42
    }

    #[pluginop_result(PluginOp::Test)]
    fn test2(&mut self, _latest_rtt: Duration) -> Result<(), Error> {
        Ok(())
    }

    #[pluginop_result_param(po = "PluginOp::ParseFrame", param = "ty")]
    fn parse_frame(
        &mut self,
        ty: u64,
        buf: &mut Octets,
        pkt_type: quic::PacketType,
    ) -> Result<quic::Frame, Error> {
        match ty {
            0x10 => {
                if pkt_type != quic::PacketType::Short {
                    return Err(Error);
                }
                let maximum_data = buf.get_varint().map_err(|_| Error)?;
                Ok(quic::Frame::MaxData(quic::MaxDataFrame { maximum_data }))
            }
            _ => Err(Error),
        }
    }

    #[pluginop_param(po = "PluginOp::ProcessFrame", param = "ty")]
    fn process_frame(
        &mut self,
        ty: u64,
        f: quic::Frame,
        _hdr: quic::Header,
        _rcv_info: quic::RcvInfo,
        _epoch: u64,
        now: Instant,
    ) {
        match f {
            Frame::MaxData(mdf) => {
                // Voluntary buggy implementation.
                self.max_tx_data = mdf.maximum_data;
            }
            Frame::ACK(af) => {
                let latest_rtt = Duration::from_millis(100);
                self.update_rtt(latest_rtt, Duration::from_millis(af.ack_delay), now);
            }
            _ => todo!(),
        }
    }

    pub fn recv_frame(&mut self, f: quic::Frame) {
        let dcid = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let bytes = self
            .get_pluginizable_connection()
            .unwrap()
            .get_ph_mut()
            .add_bytes_content(dcid.into());
        // Fake receive process.
        let hdr = quic::Header {
            first: 0,
            version: None,
            destination_cid: bytes,
            source_cid: None,
            supported_versions: None,
            ext: None,
        };
        let rcv_info = quic::RcvInfo {
            from: "0.0.0.0:1234".parse().unwrap(),
            to: "0.0.0.0:4321".parse().unwrap(),
        };
        let epoch = 2;
        let now = Instant::now();
        let ty = match f {
            Frame::MaxData(_) => 0x10,
            _ => 0x99,
        };
        self.process_frame(ty, f, hdr, rcv_info, epoch, now);
    }

    #[pluginop_param(po = "PluginOp::ShouldSendFrame", param = "ty")]
    fn should_send_frame(
        &mut self,
        ty: u64,
        pkt_type: quic::PacketType,
        epoch: quic::KPacketNumberSpace,
        is_closing: bool,
        left: usize,
    ) -> bool {
        // We only agree to send MAX_DATA frames.
        ty == 0x10
    }

    #[pluginop_result_param(po = "PluginOp::PrepareFrame", param = "ty")]
    fn prepare_frame(
        &mut self,
        ty: u64,
        epoch: quic::KPacketNumberSpace,
        left: usize,
    ) -> Result<quic::Frame, Error> {
        if ty == 0x10 {
            Ok(quic::Frame::MaxData(quic::MaxDataFrame {
                maximum_data: 0x2000,
            }))
        } else {
            Err(Error)
        }
    }

    #[pluginop_param(po = "PluginOp::WireLen", param = "ty")]
    fn wire_len(&mut self, ty: u64, f: quic::Frame) -> usize {
        if ty == 0x10 {
            // 1 ty byte + 2 byte encoding for maximum data 0x2000
            1 + 2
        } else {
            5000
        }
    }

    #[pluginop_result_param(po = "PluginOp::WriteFrame", param = "ty")]
    fn write_frame(
        &mut self,
        ty: u64,
        f: quic::Frame,
        buf: &mut OctetsMut,
    ) -> Result<usize, Error> {
        if ty == 0x10 {
            let before = buf.cap();
            buf.put_varint(0x10).map_err(|_| Error)?;
            buf.put_varint(0x2000).map_err(|_| Error)?;
            Ok(before - buf.cap())
        } else {
            Err(Error)
        }
    }

    #[pluginop_param(po = "PluginOp::OnFrameReserved", param = "ty")]
    fn on_frame_reserved(&mut self, ty: u64, f: quic::Frame) {}

    #[pluginop_param(po = "PluginOp::NotifyFrame", param = "ty")]
    fn notify_frame(&mut self, ty: u64, f: quic::Frame, lost: bool) {}

    pub fn send_pkt(&mut self, buf: &mut OctetsMut, notify: Option<bool>) -> usize {
        let registrations = self
            .get_pluginizable_connection()
            .unwrap()
            .get_ph()
            .get_registrations()
            .iter()
            .copied()
            .filter_map(|r| {
                if let Registration::Frame(f) = r {
                    Some(f)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut _ack_eliciting = false;
        let mut _in_flight = false;
        let mut frames = Vec::new();
        for r in registrations {
            if self.should_send_frame(
                r.get_type(),
                quic::PacketType::Short,
                quic::KPacketNumberSpace::ApplicationData,
                false,
                buf.len(),
            ) {
                let f = match self.prepare_frame(
                    r.get_type(),
                    quic::KPacketNumberSpace::ApplicationData,
                    buf.cap(),
                ) {
                    Ok(f) => f,
                    Err(Error) => continue,
                };
                if self.wire_len(r.get_type(), f) <= buf.cap() {
                    match self.write_frame(r.get_type(), f, buf) {
                        Ok(_) => {
                            self.on_frame_reserved(r.get_type(), f);
                            frames.push((r.get_type(), f));
                            _ack_eliciting |= r.ack_eliciting();
                            _in_flight |= r.count_for_in_flight();
                        }
                        Err(Error) => continue,
                    }
                }
            }
        }

        // Fake here the acknowledgment of the sent frames.
        if let Some(lost) = notify {
            for (ty, f) in frames {
                self.notify_frame(ty, f, lost);
            }
        }

        buf.off()
    }

    pub fn recv_pkt(&mut self, buf: &mut Octets, now: Instant) -> Result<usize, Error> {
        // Fake receive process.
        // XXX: here, we do not allow the plugin to read the DCID.
        let hdr = quic::Header {
            first: 0,
            version: None,
            destination_cid: Bytes {
                tag: u64::MAX,
                max_read_len: 0,
                max_write_len: 0,
            },
            source_cid: None,
            supported_versions: None,
            ext: None,
        };
        let rcv_info = quic::RcvInfo {
            from: "0.0.0.0:1234".parse().unwrap(),
            to: "0.0.0.0:4321".parse().unwrap(),
        };
        let epoch = 2;
        // Parse unencrypted bytes as direct frames.
        while buf.cap() > 0 {
            let ty = buf.get_varint().map_err(|_| Error)?;
            let f = self.parse_frame(ty, buf, quic::PacketType::Short)?;
            self.process_frame(ty, f, hdr, rcv_info, epoch, now);
        }

        Ok(buf.off())
    }
}

pub struct PluginizableConnectionDummy(Box<PluginizableConnection<ConnectionDummy>>);

impl PluginizableConnectionDummy {
    pub fn new_pluginizable_connection(
        exports_func: fn(&mut Store, &FunctionEnv<Env<ConnectionDummy>>) -> Exports,
    ) -> PluginizableConnectionDummy {
        let conn = ConnectionDummy {
            pc: None,
            max_tx_data: 2000,
            srtt: Duration::from_millis(333),
        };
        let mut ret = PluginizableConnectionDummy(
            PluginizableConnection::new_pluginizable_connection(exports_func, conn),
        );
        let pc_ptr = ret.0.as_mut() as *mut _;
        ret.0.get_conn_mut().set_pluginizable_connection(pc_ptr);
        ret.0.get_ph_mut().set_pluginizable_connection(pc_ptr);
        ret
    }

    pub fn recv_frame(&mut self, f: quic::Frame) {
        self.0.conn.recv_frame(f)
    }

    // Should not be really part of an official API, but kept for testing purposes.
    pub fn update_rtt(&mut self, lrtt: Duration, ack_delay: Duration, now: Instant) {
        self.0.conn.update_rtt(lrtt, ack_delay, now)
    }

    pub fn recv_pkt(&mut self, buf: &mut Octets, now: Instant) -> Result<usize, Error> {
        self.0.conn.recv_pkt(buf, now)
    }

    pub fn send_pkt(&mut self, buf: &mut OctetsMut, notify: Option<bool>) -> usize {
        self.0.conn.send_pkt(buf, notify)
    }
}

impl Deref for PluginizableConnectionDummy {
    type Target = PluginizableConnection<ConnectionDummy>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PluginizableConnectionDummy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pluginop::{
        common::{
            quic::{self, Frame, MaxDataFrame, QVal},
            PluginOp, PluginVal,
        },
        octets::{Octets, OctetsMut},
        plugin::Env,
        Error,
    };
    use unix_time::Instant;
    use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};

    use crate::{ConnectionDummy, PluginizableConnectionDummy};

    fn add_one(_: FunctionEnvMut<Env<ConnectionDummy>>, x: u64) -> u64 {
        x + 1
    }

    fn exports_func_external_test(
        store: &mut Store,
        env: &FunctionEnv<Env<ConnectionDummy>>,
    ) -> Exports {
        let mut exports = Exports::new();
        exports.insert("add_one", Function::new_typed_with_env(store, env, add_one));
        exports
    }

    #[test]
    fn simple_wasm() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/simple-wasm/simple_wasm.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let (po, a) = PluginOp::from_name("simple_call");
        assert!(pcd.0.get_ph().provides(&po, a));
        let ph = pcd.0.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
    }

    #[test]
    fn memory_allocation() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/memory-allocation/memory_allocation.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let (po, a) = PluginOp::from_name("check_data");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.0.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let (po2, a2) = PluginOp::from_name("free_data");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.0.get_ph_mut();
        let _ = ph.call(&po2, &[]);
        let res = ph.call(&po, &[]);
        assert!(res.is_err());
        if let Error::OperationError(e) = res.unwrap_err() {
            assert_eq!(e, -1);
        } else {
            assert!(false);
        }
    }

    fn memory_run(path: &str) {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = path.to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let (po, a) = PluginOp::from_name("get_mult_value");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
        let (po2, a2) = PluginOp::from_name("set_values");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po2, &[(2 as i32).into(), (3 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po2, &[(0 as i32).into(), (0 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
    }

    #[test]
    fn static_memory() {
        memory_run("../tests/static-memory/static_memory.wasm");
    }

    #[test]
    fn inputs_support() {
        memory_run("../tests/inputs-support/inputs_support.wasm");
    }

    #[test]
    fn input_outputs() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/input-outputs/input_outputs.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let (po, a) = PluginOp::from_name("get_calc_value");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(
            *res.unwrap(),
            [
                PluginVal::I32(1),
                PluginVal::I32(-1),
                PluginVal::I32(0),
                PluginVal::I32(0)
            ]
        );
        let (po2, a2) = PluginOp::from_name("set_values");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po2, &[(12 as i32).into(), (3 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(
            *res.unwrap(),
            [
                PluginVal::I32(15),
                PluginVal::I32(9),
                PluginVal::I32(36),
                PluginVal::I32(4)
            ]
        );
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po2, &[(0 as i32).into(), (1 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(
            *res.unwrap(),
            [
                PluginVal::I32(1),
                PluginVal::I32(-1),
                PluginVal::I32(0),
                PluginVal::I32(0)
            ]
        );
    }

    #[test]
    fn increase_max_data() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string(); // "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let (po, a) = PluginOp::from_name("process_frame_10");
        assert!(pcd.get_ph().provides(&po, a));
        let old_value = pcd.conn.max_tx_data;
        let new_value = old_value - 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[QVal::Frame(Frame::MaxData(md_frame)).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        assert_eq!(pcd.conn.max_tx_data, old_value);
        let new_value = old_value + 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pcd.get_ph_mut();
        let res = ph.call(&po, &[QVal::Frame(Frame::MaxData(md_frame)).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        assert_eq!(pcd.conn.max_tx_data, new_value);
    }

    #[test]
    fn first_pluginop() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
        assert_eq!(pcd.conn.max_tx_data, 2000);
        // Fix this with the plugin.
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
    }

    #[test]
    fn pluginop_macro_simple() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        pcd.update_rtt(
            Duration::from_millis(250),
            Duration::from_millis(10),
            Instant::now(),
        );
        let path = "../tests/macro-simple/macro_simple.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        pcd.update_rtt(
            Duration::from_millis(125),
            Duration::from_millis(10),
            Instant::now(),
        );
        assert!(pcd.conn.max_tx_data == 12500);
        assert!(pcd.conn.srtt == Duration::from_millis(250));
    }

    #[test]
    fn max_data() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        pcd.get_ph_mut()
            .add_registration(quic::Registration::Frame(quic::FrameRegistration::new(
                0x10,
                quic::FrameSendOrder::AfterACK,
                quic::FrameSendKind::OncePerPacket,
                true,
                true,
            )));
        let mut orig_buf = [0; 1350];
        let mut buf = OctetsMut::with_slice(&mut orig_buf);
        let w = pcd.send_pkt(&mut buf, Some(false));
        assert_eq!(w, 3);
        assert_eq!(&[0x10, 0x60, 0x00], &orig_buf[..3]);
        let mut buf = Octets::with_slice(&mut orig_buf[..3]);
        let res = pcd.recv_pkt(&mut buf, Instant::now());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3);
    }

    #[test]
    fn max_data_wasm() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/max-data-frame/max_data_frame.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let mut orig_buf = [0; 1350];
        let mut buf = OctetsMut::with_slice(&mut orig_buf);
        let w = pcd.send_pkt(&mut buf, Some(false));
        assert_eq!(w, 3);
        assert_eq!(&[0x10, 0x60, 0x00], &orig_buf[..3]);
        let mut buf = Octets::with_slice(&mut orig_buf[..3]);
        let res = pcd.recv_pkt(&mut buf, Instant::now());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3);
    }

    #[test]
    fn super_frame() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/super-frame/super_frame.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into());
        assert!(ok.is_ok());
        let mut orig_buf = [0; 1350];
        let mut buf = OctetsMut::with_slice(&mut orig_buf);
        let w = pcd.send_pkt(&mut buf, Some(false));
        assert_eq!(w, 3);
        assert_eq!(&[0x40, 0x42, 0x00], &orig_buf[..3]);
        let mut buf = Octets::with_slice(&mut orig_buf[..3]);
        let res = pcd.recv_pkt(&mut buf, Instant::now());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3);
    }
}
