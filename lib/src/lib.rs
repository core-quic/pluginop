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

#[cfg(test)]
mod tests {
    use std::{marker::PhantomPinned, time::Duration};

    use pluginop_common::{
        quic::{self, ConnectionField, Frame, MaxDataFrame, QVal, RecoveryField},
        PluginVal,
    };
    use pluginop_macro::{pluginop, pluginop_param};
    use unix_time::Instant;
    use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};

    use crate::{api::CTPError, plugin::Env};

    use super::*;

    /// Dummy object
    #[derive(Debug)]
    struct ConnectionDummy {
        pc: Option<ParentReferencer<Box<dyn PluginizableConnection>>>,
        max_tx_data: u64,
        srtt: Duration,
    }

    impl api::ConnectionToPlugin for ConnectionDummy {
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
            let pv: PluginVal =
                bincode::deserialize_from(r).map_err(|_| CTPError::SerializeError)?;
            match field {
                ConnectionField::MaxTxData => {
                    self.max_tx_data = pv.try_into().map_err(|_| CTPError::BadType)?
                }
                _ => todo!(),
            };
            Ok(())
        }

        fn set_pluginizable_connection(&mut self, pc: *mut Box<dyn PluginizableConnection>) {
            self.pc = Some(ParentReferencer::new(pc));
        }

        fn get_pluginizable_connection(&mut self) -> &mut Box<dyn PluginizableConnection> {
            &mut *self.pc.as_mut().unwrap()
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
            _now: Instant,
        ) {
            match f {
                Frame::MaxData(mdf) => {
                    // Voluntary buggy implementation.
                    self.max_tx_data = mdf.maximum_data;
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

    #[derive(Debug)]
    struct PluginizableConnectionDummy {
        ph: PluginHandler,
        conn: Box<ConnectionDummy>,
        _pin: PhantomPinned,
    }

    impl PluginizableConnection for PluginizableConnectionDummy {
        fn get_conn(&self) -> &dyn api::ConnectionToPlugin {
            &*self.conn
        }

        fn get_conn_mut(&mut self) -> &mut dyn api::ConnectionToPlugin {
            // SAFETY: only valid as long as we are single-thread.
            &mut *self.conn
        }

        fn get_ph(&self) -> &PluginHandler {
            &self.ph
        }

        fn get_ph_mut(&mut self) -> &mut PluginHandler {
            // SAFETY: only valid as loing as we are single-thread.
            &mut self.ph
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    fn add_one(_: FunctionEnvMut<Env>, x: u64) -> u64 {
        x + 1
    }

    fn exports_func_external_test(store: &mut Store, env: &FunctionEnv<Env>) -> Exports {
        let mut exports = Exports::new();
        exports.insert("add_one", Function::new_typed_with_env(store, env, add_one));
        exports
    }

    impl PluginizableConnectionDummy {
        fn new(
            exports_func: fn(&mut Store, &FunctionEnv<Env>) -> Exports,
        ) -> Box<dyn PluginizableConnection> {
            Box::new(PluginizableConnectionDummy {
                ph: PluginHandler::new(exports_func),
                conn: Box::new(ConnectionDummy {
                    pc: None,
                    max_tx_data: 2000,
                    srtt: Duration::from_millis(333),
                }),
                _pin: PhantomPinned,
            })
        }

        fn recv_frame(&mut self, f: quic::Frame) {
            self.conn.recv_frame(f)
        }

        fn recv_pkt(&mut self, lrtt: Duration, ack_delay: Duration, now: Instant) {
            self.conn.recv_pkt(lrtt, ack_delay, now)
        }
    }

    #[test]
    fn simple_wasm() {
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/simple-wasm/simple_wasm.wasm".to_string();
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("simple_call");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
    }

    #[test]
    fn memory_allocation() {
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/memory-allocation/memory_allocation.wasm".to_string();
        let pcd_ptr = &pcd as *const _ as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("check_data");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let (po2, a2) = PluginOp::from_name("free_data");
        assert!(pcd.get_ph().provides(&po2, a2));
        let ph = pcd.get_ph();
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
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
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
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
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
        let mut pc = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let pc_ptr = &pc as *const _;
        let ok = pc.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        let (po, a) = PluginOp::from_name("process_frame_10");
        assert!(pc.get_ph().provides(&po, a));
        let pcd = pc
            .as_any()
            .downcast_ref::<PluginizableConnectionDummy>()
            .unwrap();
        let old_value = pcd.conn.max_tx_data;
        let new_value = old_value - 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pc.get_ph();
        let res = ph.call(&po, &[QVal::Frame(Frame::MaxData(md_frame)).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        assert_eq!(pcd.conn.max_tx_data, old_value);
        let new_value = old_value + 1000;
        let md_frame = MaxDataFrame {
            maximum_data: new_value,
        };
        let ph = pc.get_ph();
        let res = ph.call(&po, &[QVal::Frame(Frame::MaxData(md_frame)).into()]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), []);
        assert_eq!(pcd.conn.max_tx_data, new_value);
    }

    #[test]
    fn first_pluginop() {
        let mut pc = PluginizableConnectionDummy::new(exports_func_external_test);
        let pc_ptr = &mut pc as *mut _;
        // Default implementation is buggy.
        let pcd = pc
            .as_any_mut()
            .downcast_mut::<PluginizableConnectionDummy>()
            .unwrap();
        pcd.get_conn_mut().set_pluginizable_connection(pc_ptr);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
        assert_eq!(pcd.conn.max_tx_data, 2000);
        // Fix this with the plugin.
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let pc_ptr = &pc as *const _;
        let ok = pc.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        let pcd = pc
            .as_any_mut()
            .downcast_mut::<PluginizableConnectionDummy>()
            .unwrap();
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
        pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
        assert_eq!(pcd.conn.max_tx_data, 4000);
    }

    #[test]
    fn pluginop_macro_simple() {
        let mut pc = PluginizableConnectionDummy::new(exports_func_external_test);
        let pc_ptr = &mut pc as *mut _;
        // Default implementation is buggy.
        let pcd = pc
            .as_any_mut()
            .downcast_mut::<PluginizableConnectionDummy>()
            .unwrap();
        pcd.get_conn_mut().set_pluginizable_connection(pc_ptr);
        pcd.recv_pkt(
            Duration::from_millis(250),
            Duration::from_millis(10),
            Instant::now(),
        );
        let path = "../tests/macro-simple/macro_simple.wasm".to_string();
        let ok = pc.get_ph_mut().insert_plugin(&path.into(), pc_ptr);
        assert!(ok);
        let pcd = pc
            .as_any_mut()
            .downcast_mut::<PluginizableConnectionDummy>()
            .unwrap();
        pcd.recv_pkt(
            Duration::from_millis(125),
            Duration::from_millis(10),
            Instant::now(),
        );
        assert!(pcd.conn.max_tx_data == 12500);
        assert!(pcd.conn.srtt == Duration::from_millis(250));
    }
}

pub mod api;
pub mod handler;
pub mod plugin;
mod rawptr;
