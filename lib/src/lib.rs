use handler::PluginHandler;
use pluginop_common::ProtoOp;
use wasmer::{Function, RuntimeError};

#[derive(Debug, Default)]
pub struct POCode {
    pre: Option<Function>,
    replace: Option<Function>,
    post: Option<Function>,
}
pub trait PluginizableConnection: Sized + Send + Sync + 'static {
    fn get_conn(&mut self) -> &mut dyn api::ConnectionToPlugin<Self>;
    fn get_ph(&self) -> &PluginHandler<Self>;
}

/// An error that may happen during the operations of this library.
#[derive(Clone, Debug)]
pub enum Error {
    RuntimeError(RuntimeError),
    NoDefault(ProtoOp),
    OutputConversionError(String),
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock, Weak};

    use pluginop_common::quic::{ConnectionField, RecoveryField};
    use wasmer::{Exports, FunctionEnv, FunctionEnvMut, Store, Value};

    use crate::{api::ConnectionToPlugin, handler::InternalArgs, plugin::Env};

    use super::*;

    /// Dummy object
    #[derive(Debug)]
    struct ConnectionDummy {
        pc: Option<Weak<RwLock<PluginizableConnectionDummy>>>,
    }

    impl api::ConnectionToPlugin<'_, PluginizableConnectionDummy> for ConnectionDummy {
        fn get_recovery(&self, _: &mut [u8], _: RecoveryField) -> bincode::Result<()> {
            todo!()
        }

        fn set_recovery(&mut self, _: RecoveryField, _: &[u8]) {
            todo!()
        }

        fn get_connection(&self, _: &mut [u8], _field: ConnectionField) -> bincode::Result<()> {
            todo!()
        }

        fn set_connection(&mut self, _field: ConnectionField, _value: &[u8]) {
            todo!()
        }

        fn set_pluginizable_conn(&mut self, pc: &Arc<RwLock<PluginizableConnectionDummy>>) {
            self.pc = Some(Arc::<RwLock<PluginizableConnectionDummy>>::downgrade(pc));
        }

        fn get_pluginizable_conn(&self) -> Option<&Weak<RwLock<PluginizableConnectionDummy>>> {
            self.pc.as_ref()
        }
    }

    #[derive(Debug)]
    struct PluginizableConnectionDummy {
        ph: Option<PluginHandler<PluginizableConnectionDummy>>,
        conn: ConnectionDummy,
    }

    impl PluginizableConnection for PluginizableConnectionDummy {
        fn get_conn(&mut self) -> &mut dyn api::ConnectionToPlugin<Self> {
            &mut self.conn
        }

        fn get_ph(&self) -> &PluginHandler<PluginizableConnectionDummy> {
            &self.ph.as_ref().unwrap()
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
        ) -> Arc<RwLock<Self>> {
            let ret = Arc::new(RwLock::new(PluginizableConnectionDummy {
                ph: None,
                conn: ConnectionDummy { pc: None },
            }));
            {
                let mut locked_ret = ret.write().unwrap();
                locked_ret.ph = Some(PluginHandler::new(exports_func));
                locked_ret.conn.set_pluginizable_conn(&ret);
            }
            ret
        }

        fn get_ph_mut(&mut self) -> &mut PluginHandler<PluginizableConnectionDummy> {
            self.ph.as_mut().unwrap()
        }
    }

    #[test]
    fn simple_wasm() {
        let pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/simple-wasm/simple_wasm.wasm".to_string();
        let mut locked_pcd = pcd.write().unwrap();
        let pcd_ptr = &*locked_pcd as *const _;
        let ok = locked_pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("simple_call");
        assert!(locked_pcd.get_ph().provides(&po, a));
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I64(42)]);
    }

    #[test]
    fn memory_allocation() {
        let pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/memory-allocation/memory_allocation.wasm".to_string();
        let mut locked_pcd = pcd.write().unwrap();
        let pcd_ptr = &*locked_pcd as *const _;
        let ok = locked_pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("check_data");
        assert!(locked_pcd.get_ph().provides(&po, a));
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I32(6)]);
        let (po2, a2) = ProtoOp::from_name("free_data");
        assert!(locked_pcd.get_ph().provides(&po2, a2));
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let _ = ph.call(&po2, &[], |_| {}, |_, r| r, internal_args);
        let internal_args = InternalArgs::default();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I32(-1)]);
    }

    #[test]
    fn static_memory() {
        let pcd = PluginizableConnectionDummy::new(exports_func_external_test);
        let path = "../tests/static-memory/static_memory.wasm".to_string();
        let mut locked_pcd = pcd.write().unwrap();
        let pcd_ptr = &*locked_pcd as *const _;
        let ok = locked_pcd.get_ph_mut().insert_plugin(&path.into(), pcd_ptr);
        assert!(ok);
        let (po, a) = ProtoOp::from_name("get_mult_value");
        assert!(locked_pcd.get_ph().provides(&po, a));
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I64(0)]);
        let (po2, a2) = ProtoOp::from_name("set_values");
        assert!(locked_pcd.get_ph().provides(&po2, a2));
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(
            &po2,
            &[(2 as i32).into(), (3 as i32).into()],
            |_| {},
            |_, r| r,
            internal_args,
        );
        assert!(res.is_ok());
        let internal_args = InternalArgs::default();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I64(6)]);
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(
            &po2,
            &[(0 as i32).into(), (0 as i32).into()],
            |_| {},
            |_, r| r,
            internal_args,
        );
        assert!(res.is_ok());
        let internal_args = InternalArgs::default();
        let ph = locked_pcd.get_ph();
        let res = ph.call(&po, &[], |_| {}, |_, r| r, internal_args);
        assert!(res.is_ok());
        assert_eq!(*res.unwrap(), [Value::I64(0)]);
    }
}

pub mod api;
pub mod handler;
pub mod plugin;
