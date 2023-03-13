use std::ops::{Deref, DerefMut};
use std::time::Duration;

use pluginop::api::ConnectionToPlugin;
use pluginop::common::quic::{self, Frame};
use pluginop::common::PluginOp;
use pluginop::common::{
    quic::{ConnectionField, RecoveryField},
    PluginVal,
};
use pluginop::plugin::Env;
use pluginop::pluginop_macro::{pluginop, pluginop_param};
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

    fn set_pluginizable_connection(&mut self, pc: *mut PluginizableConnection<Self>) {
        self.pc = Some(ParentReferencer::new(pc));
    }

    fn get_pluginizable_connection(&mut self) -> Option<&mut PluginizableConnection<Self>> {
        self.pc.as_deref_mut()
    }
}

impl ConnectionDummy {
    #[pluginop(PluginOp::UpdateRtt)]
    fn update_rtt(&mut self, latest_rtt: Duration, _ack_delay: Duration, _now: Instant) {
        self.srtt = latest_rtt;
    }

    pub fn recv_pkt(&mut self, latest_rtt: Duration, ack_delay: Duration, now: Instant) {
        self.update_rtt(latest_rtt, ack_delay, now);
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
        // Fake receive process.
        let hdr = quic::Header {
            first: 0,
            version: None,
            destination_cid: 0,
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
}

pub struct PluginizableConnectionDummy(Box<PluginizableConnection<ConnectionDummy>>);

impl PluginizableConnectionDummy {
    pub fn new_pluginizable_connection(
        exports_func: fn(&mut Store, &FunctionEnv<Env<ConnectionDummy>>) -> Exports,
    ) -> Box<PluginizableConnectionDummy> {
        let conn = ConnectionDummy {
            pc: None,
            max_tx_data: 2000,
            srtt: Duration::from_millis(333),
        };
        let mut ret = Box::new(PluginizableConnectionDummy(
            PluginizableConnection::new_pluginizable_connection(exports_func, conn),
        ));
        let pc_ptr = &mut *ret.0 as *mut _;
        ret.0.get_conn_mut().set_pluginizable_connection(pc_ptr);
        ret
    }

    pub fn recv_frame(&mut self, f: quic::Frame) {
        self.0.conn.recv_frame(f)
    }

    pub fn recv_pkt(&mut self, lrtt: Duration, ack_delay: Duration, now: Instant) {
        self.0.conn.recv_pkt(lrtt, ack_delay, now)
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
            quic::{Frame, MaxDataFrame, QVal},
            PluginOp, PluginVal,
        },
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
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("simple_call");
        assert!(pcd.0.get_ph().provides(&po, a));
        let ph = pcd.0.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
    }

    #[test]
    fn memory_allocation() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let path = "../tests/memory-allocation/memory_allocation.wasm".to_string();
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("check_data");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.0.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let (po2, a2) = PluginOp::from_name("free_data");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.0.get_ph();
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
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("get_mult_value");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
        let (po2, a2) = PluginOp::from_name("set_values");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.get_ph();
        let res = ph.call(&po2, &[(2 as i32).into(), (3 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let ph = pcd.get_ph();
        let res = ph.call(&po2, &[(0 as i32).into(), (0 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let ph = pcd.get_ph();
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
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("get_calc_value");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
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
        let ph = pcd.get_ph();
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
        let ph = pcd.get_ph();
        let res = ph.call(&po2, &[(0 as i32).into(), (1 as i32).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        let ph = pcd.get_ph();
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
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let pc_ptr = &*pcd.0 as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("process_frame_10");
        assert!(pcd.get_ph().provides(&po, a));
        let old_value = pcd.conn.max_tx_data;
        let new_value = old_value - 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[QVal::Frame(Frame::MaxData(md_frame)).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        assert_eq!(pcd.conn.max_tx_data, old_value);
        let new_value = old_value + 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pcd.get_ph();
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
        let pc_ptr = &*pcd.0 as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
    }

    #[test]
    fn pluginop_macro_simple() {
        let mut pcd =
            PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
        let pc_ptr = &mut *pcd.0 as *mut _;
        pcd.recv_pkt(
            Duration::from_millis(250),
            Duration::from_millis(10),
            Instant::now(),
        );
        let path = "../tests/macro-simple/macro_simple.wasm".to_string();
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        pcd.recv_pkt(
            Duration::from_millis(125),
            Duration::from_millis(10),
            Instant::now(),
        );
        assert!(pcd.conn.max_tx_data == 12500);
        assert!(pcd.conn.srtt == Duration::from_millis(250));
    }
}
