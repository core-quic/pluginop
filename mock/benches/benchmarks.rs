use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use pluginop::{
    common::{
        quic::{Frame, MaxDataFrame, QVal},
        PluginOp, PluginVal,
    },
    plugin::Env,
    Error, PluginizableConnection,
};
use pluginop_mock::PluginizableConnectionDummy;
use unix_time::Instant;
use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};

fn add_one(_: FunctionEnvMut<Env>, x: u64) -> u64 {
    x + 1
}

fn exports_func_external_test(store: &mut Store, env: &FunctionEnv<Env>) -> Exports {
    let mut exports = Exports::new();
    exports.insert("add_one", Function::new_typed_with_env(store, env, add_one));
    exports
}

fn memory_allocation_bench() {
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
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

fn first_pluginop() {
    let mut pc =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
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

fn macro_simple() {
    let mut pc =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
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

fn criterion_benchmark(c: &mut Criterion) {
    // First test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
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
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = "../tests/static-memory/static_memory.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("static memory", |b| b.iter(|| static_memory(&mut pcd)));

    // Fourth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = "../tests/inputs-support/inputs_support.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("inputs support", |b| b.iter(|| static_memory(&mut pcd)));

    // Fifth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = "../tests/input-outputs/input_outputs.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("input outputs", |b| b.iter(|| input_outputs(&mut pcd)));

    // Sixth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
    let pcd_ptr = &pcd as *const _;
    let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
    assert!(ok);
    c.bench_function("increase-max-data", |b| {
        b.iter(|| increase_max_data(&mut pcd))
    });

    // Seventh test
    c.bench_function("first pluginop", |b| b.iter(|| first_pluginop()));

    // Eigth test
    c.bench_function("macro simple", |b| b.iter(|| macro_simple()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
