use handler::PluginHandler;
use pluginop_common::{PluginInputType, PluginOutputType, ProtoOp};
use wasmer::{RuntimeError, TypedFunction};

pub type PluginFunction = TypedFunction<PluginInputType, PluginOutputType>;

#[derive(Default)]
pub struct POCode {
    pre: Option<PluginFunction>,
    replace: Option<PluginFunction>,
    post: Option<PluginFunction>,
}

pub trait PluginizableConnection: Sized + Send + 'static {
    fn get_conn(&self) -> &dyn api::ConnectionToPlugin<Self>;
    fn get_conn_mut(&mut self) -> &mut dyn api::ConnectionToPlugin<Self>;
    fn get_ph(&self) -> &PluginHandler<Self>;
    fn get_ph_mut(&mut self) -> &mut PluginHandler<Self>;
}

/// An error that may happen during the operations of this library.
#[derive(Clone, Debug)]
pub enum Error {
    RuntimeError(RuntimeError),
    NoDefault(ProtoOp),
    OutputConversionError(String),
    OperationError(i64),
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomPinned;

    use pluginop_common::{
        quic::{ConnectionField, Frame, MaxDataFrame, QVal, RecoveryField},
        PluginVal,
    };
    use wasmer::{Exports, Function, FunctionEnv, FunctionEnvMut, Store};

    use crate::{api::CTPError, plugin::Env};

    use super::*;

    /// Dummy object
    #[derive(Debug)]
    struct ConnectionDummy {
        max_tx_data: u64,
    }

    impl api::ConnectionToPlugin<'_, PluginizableConnectionDummy> for ConnectionDummy {
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
    }

    #[derive(Debug)]
    struct PluginizableConnectionDummy {
        ph: PluginHandler<PluginizableConnectionDummy>,
        conn: Box<ConnectionDummy>,
        _pin: PhantomPinned,
    }

    impl PluginizableConnection for PluginizableConnectionDummy {
        fn get_conn(&self) -> &dyn api::ConnectionToPlugin<Self> {
            &*self.conn
        }

        fn get_conn_mut(&mut self) -> &mut dyn api::ConnectionToPlugin<Self> {
            // SAFETY: only valid as long as we are single-thread.
            &mut *self.conn
        }

        fn get_ph(&self) -> &PluginHandler<PluginizableConnectionDummy> {
            &self.ph
        }

        fn get_ph_mut(&mut self) -> &mut PluginHandler<Self> {
            // SAFETY: only valid as loing as we are single-thread.
            &mut self.ph
        }
    }

    fn add_one<P: PluginizableConnection>(_: FunctionEnvMut<Env<P>>, x: u64) -> u64 {
        x + 1
    }

    fn exports_func_external_test<P: PluginizableConnection>(
        store: &mut Store,
        env: &FunctionEnv<Env<P>>,
    ) -> Exports {
        let mut exports = Exports::new();
        exports.insert("add_one", Function::new_typed_with_env(store, env, add_one));
        exports
    }

    impl PluginizableConnectionDummy {
        fn new(
            exports_func: fn(&mut Store, &FunctionEnv<Env<Self>>) -> Exports,
        ) -> Box<PluginizableConnectionDummy> {
            Box::new(PluginizableConnectionDummy {
                ph: PluginHandler::new(exports_func),
                conn: Box::new(ConnectionDummy { max_tx_data: 2000 }),
                _pin: PhantomPinned,
            })
        }
    }

    #[test]
    fn simple_wasm() {
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/simple-wasm/simple_wasm.wasm".to_string();
        let pcd_ptr = &pcd as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("simple_call");
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
        let pcd_ptr = &pcd as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("check_data");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(6)]);
        let (po2, a2) = ProtoOp::from_name("free_data");
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
        let pcd_ptr = &pcd as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("get_mult_value");
        assert!(pcd.get_ph().provides(&po, a));
        let ph = pcd.get_ph();
        let res = ph.call(&po, &[]);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [PluginVal::I64(0)]);
        let (po2, a2) = ProtoOp::from_name("set_values");
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
        let pcd_ptr = &pcd as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("get_calc_value");
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
        let (po2, a2) = ProtoOp::from_name("set_values");
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
        let mut pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/increase-max-data/increase_max_data.wasm".to_string();
        let pcd_ptr = &pcd as *const _;
        let ok = pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("process_frame_10");
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
}

pub mod api;
pub mod handler;
pub mod plugin;
mod rawptr;
