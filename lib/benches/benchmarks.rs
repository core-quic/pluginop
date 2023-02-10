use std::{any::Any, marker::PhantomPinned};

use criterion::{criterion_group, criterion_main, Criterion};
use pluginop::{
    api::{self, CTPError, ConnectionToPlugin},
    handler::PluginHandler,
    plugin::Env,
    Error, ParentReferencer, PluginizableConnection,
};
use pluginop_common::{
    quic::{self, ConnectionField, Frame, MaxDataFrame, QVal, RecoveryField},
    PluginOp, PluginVal,
};
use unix_time::Instant;
use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};

#[derive(Debug)]
struct ConnectionDummy {
    pc: Option<ParentReferencer<Box<dyn PluginizableConnection>>>,
    max_tx_data: u64,
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
        let pv: PluginVal = bincode::deserialize_from(r).map_err(|_| CTPError::SerializeError)?;
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
    fn process_frame_default(&mut self, f: quic::Frame) {
        match f {
            Frame::MaxData(mdf) => {
                // Voluntary buggy implementation.
                self.max_tx_data = mdf.maximum_data;
            }
            _ => todo!(),
        }
    }

    fn process_frame(
        &mut self,
        f: quic::Frame,
        hdr: &quic::Header,
        rcv_info: quic::RcvInfo,
        epoch: u64,
        now: Instant,
    ) {
        // TODO: define right expected signature for the function.
        // if self.pc
        // TODO: pre/post.
        let ph = self.get_pluginizable_connection().get_ph();
        let po = PluginOp::ProcessFrame(0x10);
        if ph.provides(&po, pluginop_common::Anchor::Replace) {
            ph.call(
                &po,
                &[
                    f.into(),
                    hdr.clone().into(),
                    rcv_info.into(),
                    epoch.into(),
                    now.into(),
                ],
            );
        } else {
            self.process_frame_default(f)
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
        self.process_frame(f, &hdr, rcv_info, epoch, now);
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
            }),
            _pin: PhantomPinned,
        })
    }

    fn recv_frame(&mut self, f: quic::Frame) {
        self.conn.recv_frame(f)
    }
}

fn memory_allocation_bench() {
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/memory-allocation/memory_allocation.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
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

fn static_memory(pcd: &mut Box<dyn PluginizableConnection>) {
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
    let res = ph.call(&po, &[]);
    assert!(res.is_ok());
    assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
    let ph = pcd.get_ph();
    let res = ph.call(&po2, &[(0 as i32).into(), (0 as i32).into()]);
    assert!(res.is_ok());
    let ph = pcd.get_ph();
    let res = ph.call(&po, &[]);
    assert!(res.is_ok());
    assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
}

fn input_outputs(pcd: &mut Box<dyn PluginizableConnection>) {
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

fn increase_max_data(pc: &mut Box<dyn PluginizableConnection>) {
    let (po, a) = PluginOp::from_name("process_frame_10");
    assert!(pc.get_ph().provides(&po, a));
    // Reset to same state.
    let pcd = pc
        .as_any_mut()
        .downcast_mut::<PluginizableConnectionDummy>()
        .unwrap();
    pcd.conn.max_tx_data = 2000;
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

fn criterion_benchmark(c: &mut Criterion) {
    // First test
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/simple-wasm/simple_wasm.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    let (po, a) = PluginOp::from_name("simple_call");
    assert!(pcd.get_ph().provides(&po, a));
    let ph = pcd.get_ph();
    c.bench_function("run and return", |b| b.iter(|| ph.call(&po, &[])));

    // Second test
    c.bench_function("memory allocation", |b| {
        b.iter(|| memory_allocation_bench())
    });

    // Third test
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/static-memory/static_memory.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("static memory", |b| b.iter(|| static_memory(&mut pcd)));

    // Fourth test
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/inputs-support/inputs_support.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("inputs support", |b| b.iter(|| static_memory(&mut pcd)));

    // Fifth test
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/input-outputs/input_outputs.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("input outputs", |b| b.iter(|| input_outputs(&mut pcd)));

    // Sixth test
    let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
    let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("increase-max-data", |b| {
        b.iter(|| increase_max_data(&mut pcd))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
