use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use pluginop::{
    common::{
        quic::{Frame, FrameRegistration, MaxDataFrame, QVal},
        PluginOp, PluginVal,
    },
    octets::{Octets, OctetsMut},
    plugin::Env,
    Error,
};
use pluginop::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};
use pluginop_mock::{ConnectionDummy, PluginizableConnectionDummy};

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

// Normal user.
static BASE: &'static str = "..";
// Root user.
// static BASE: &'static str = "/Users/qdeconinck/code/pluginop";

fn memory_allocation_bench() {
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/memory-allocation/memory_allocation.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    let (po, a) = PluginOp::from_name("check_data");
    assert!(pcd.get_ph().provides(&po, a));
    let ph = pcd.get_ph_mut();
    let res = ph.call(&po, &[]);
    assert!(res.is_ok());
    assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
    let (po2, a2) = PluginOp::from_name("free_data");
    assert!(pcd.get_ph().provides(&po2, a2));
    let ph = pcd.get_ph_mut();
    let _ = ph.call(&po2, &[]);
    let res = ph.call(&po, &[]);
    assert!(res.is_err());
    if let Error::OperationError(e) = res.unwrap_err() {
        assert_eq!(e, -1);
    } else {
        assert!(false);
    }
}

fn static_memory(pcd: &mut PluginizableConnectionDummy) {
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
    let res = ph.call(&po, &[]);
    assert!(res.is_ok());
    assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
    let ph = pcd.get_ph_mut();
    let res = ph.call(&po2, &[(0 as i32).into(), (0 as i32).into()]);
    assert!(res.is_ok());
    let ph = pcd.get_ph_mut();
    let res = ph.call(&po, &[]);
    assert!(res.is_ok());
    assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
}

fn input_outputs(pcd: &mut PluginizableConnectionDummy) {
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

fn increase_max_data(pcd: &mut PluginizableConnectionDummy) {
    let (po, a) = PluginOp::from_name("process_frame_10");
    assert!(pcd.get_ph().provides(&po, a));
    // Reset to same state.
    pcd.conn.max_tx_data = 2000;
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

fn first_pluginop() {
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
    assert_eq!(pcd.conn.max_tx_data, 4000);
    pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
    assert_eq!(pcd.conn.max_tx_data, 2000);
    // Fix this with the plugin.
    let path = [BASE, "/tests/increase-max-data/increase_max_data.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 4000 }));
    assert_eq!(pcd.conn.max_tx_data, 4000);
    pcd.recv_frame(Frame::MaxData(MaxDataFrame { maximum_data: 2000 }));
    assert_eq!(pcd.conn.max_tx_data, 4000);
}

fn macro_simple() {
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    pcd.update_rtt(
        Duration::from_millis(250),
        Duration::from_millis(10),
        Instant::now(),
    );
    let path = [BASE, "/tests/macro-simple/macro_simple.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    pcd.update_rtt(
        Duration::from_millis(125),
        Duration::from_millis(10),
        Instant::now(),
    );
    assert!(pcd.conn.max_tx_data == 12500);
    assert!(pcd.conn.srtt == Duration::from_millis(250));
}

fn max_data(pcd: &mut PluginizableConnectionDummy, orig_buf: &mut [u8]) {
    let mut buf = OctetsMut::with_slice(orig_buf);
    let w = pcd.send_pkt(&mut buf, Some(false));
    assert_eq!(w, 3);
    assert_eq!(&[0x10, 0x60, 0x00], &orig_buf[..3]);
    let mut buf = Octets::with_slice(&mut orig_buf[..3]);
    let res = pcd.recv_pkt(&mut buf, Instant::now());
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);
}

fn super_frame(pcd: &mut PluginizableConnectionDummy, orig_buf: &mut [u8]) {
    let mut buf = OctetsMut::with_slice(orig_buf);
    let w = pcd.send_pkt(&mut buf, Some(false));
    assert_eq!(w, 3);
    // We cannot compare the last byte of orig_buf because it will change over time.
    assert_eq!(&[0x40, 0x42], &orig_buf[..2]);
    let mut buf = Octets::with_slice(&mut orig_buf[..3]);
    let res = pcd.recv_pkt(&mut buf, Instant::now());
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);
}

fn criterion_benchmark(c: &mut Criterion) {
    // First test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);

    let path = [BASE, "/tests/simple-wasm/simple_wasm.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    let (po, a) = PluginOp::from_name("simple_call");
    assert!(pcd.get_ph().provides(&po, a));
    let ph = pcd.get_ph_mut();
    c.bench_function("run and return", |b| b.iter(|| ph.call(&po, &[])));

    // Second test
    c.bench_function("memory allocation", |b| {
        b.iter(|| memory_allocation_bench())
    });

    // Third test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/static-memory/static_memory.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("static memory", |b| b.iter(|| static_memory(&mut pcd)));

    // Fourth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/inputs-support/inputs_support.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("inputs support", |b| b.iter(|| static_memory(&mut pcd)));

    // Fifth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/input-outputs/input_outputs.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("input outputs", |b| b.iter(|| input_outputs(&mut pcd)));

    // Sixth test
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/increase-max-data/increase_max_data.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("increase-max-data", |b| {
        b.iter(|| increase_max_data(&mut pcd))
    });

    // Seventh test
    c.bench_function("first pluginop", |b| b.iter(|| first_pluginop()));

    // Eigth test
    c.bench_function("macro simple", |b| b.iter(|| macro_simple()));

    // Ninth, tenth and eleventh tests.
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    pcd.get_ph_mut()
        .add_registration(pluginop::common::quic::Registration::Frame(
            FrameRegistration::new(
                0x10,
                pluginop::common::quic::FrameSendOrder::AfterACK,
                pluginop::common::quic::FrameSendKind::OncePerPacket,
                true,
                true,
            ),
        ));
    let mut orig_buf = [0; 1350];
    c.bench_function("max-data send and receive", |b| {
        b.iter(|| max_data(&mut pcd, &mut orig_buf))
    });
    // The exact same behavior, but with a WASM plugin.
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/max-data-frame/max_data_frame.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("max-data wasm send and receive", |b| {
        b.iter(|| max_data(&mut pcd, &mut orig_buf))
    });

    // Now insert super-frame as a plugin. Recreate a new pcd to discard the previous registration.
    let mut pcd =
        PluginizableConnectionDummy::new_pluginizable_connection(exports_func_external_test);
    let path = [BASE, "/tests/super-frame/super_frame.wasm"]
        .join("")
        .to_string();
    let ok = pcd.get_ph_mut().insert_plugin_testing(&path.into());
    assert!(ok.is_ok());
    c.bench_function("super-frame send and receive", |b| {
        b.iter(|| super_frame(&mut pcd, &mut orig_buf))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
