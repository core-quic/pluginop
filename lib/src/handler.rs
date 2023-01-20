use std::{
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use log::error;
use pluginop_common::{Anchor, PluginVal, ProtoOp};
use wasmer::{Engine, Exports, FunctionEnv, Store};
use wasmer_compiler_singlepass::Singlepass;

use crate::{
    api::get_imports_with,
    plugin::{create_env, Env, Plugin},
    rawptr::RawPtr,
    Error, PluginFunction, PluginizableConnection,
};

/// Get a store for plugins. Note that this function should be called once for a host.
fn create_store() -> Store {
    let compiler = Singlepass::new();
    let engine: Engine = compiler.into();
    Store::new(engine)
}

/// A pinned `Vec` of plugins.
struct PluginArray<P: PluginizableConnection> {
    /// The inner array.
    array: Vec<Plugin<P>>,
}

impl<P: PluginizableConnection> Deref for PluginArray<P> {
    type Target = Vec<Plugin<P>>;

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<P: PluginizableConnection> DerefMut for PluginArray<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl<P: PluginizableConnection> PluginArray<P> {
    /// Returns `true` iif one of the plugins provides an implementation for the requested `po`.
    fn provides(&self, po: &ProtoOp, anchor: Anchor) -> bool {
        self.iter().any(|p| p.get_func(po, anchor).is_some())
    }

    /// Returns the first plugin that provides an implementation for `po` with the implementing
    /// function, or `None` if there is not.
    fn get_first_plugin(&self, po: &ProtoOp) -> Option<(&Plugin<P>, &PluginFunction)> {
        for p in self.iter() {
            if let Some(func) = p.get_func(po, Anchor::Replace) {
                return Some((p, func));
            }
        }
        None
    }
}

pub struct PluginHandler<P: PluginizableConnection> {
    /// The store that served to instantiate plugins.
    store: Store,
    /// A pointer to the serving session. It can stay null if no plugin is inserted.
    conn: RawPtr<P>,
    /// Function creating an `Imports`.
    exports_func: fn(&mut Store, &FunctionEnv<Env<P>>) -> Exports,
    /// The actual container of the plugins.
    plugins: PluginArray<P>,
    /// Force this structure to be pinned.
    _pin: PhantomPinned,
}

impl<P: PluginizableConnection> std::fmt::Debug for PluginHandler<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHandler")
            .field("store", &self.store)
            .field("_pin", &self._pin)
            .finish()
    }
}

/// Permission that can be granted to plugins.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Permission {
    /// Permission to save output (should be always granted)
    Output,
    /// Permission to store opaque values (should be always granted)
    Opaque,
    /// Permission to access the Connection state
    ConnectionAccess,
    /// Permission to access the write byte buffer
    WriteBuffer,
    /// Permission to access the read byte buffer
    ReadBuffer,
}

/// A default implementation of a protocol operation proposed by the host implementation.
pub struct ProtocolOperationDefault {
    // po: ProtoOp,
    // default_fn: fn(InternalArgs, &[Value]) -> Box<dyn Any>,
    // return_type: POReturnType,
    // named_args: Vec<&'static str>,
    // named_refs: Vec<&'static str>,
    // named_muts: Vec<&'static str>,
    // use_transient: UseTransientStructs,
}

impl<P: PluginizableConnection> PluginHandler<P> {
    /// Creates a new `PluginHandler`, enabling the execution of `Plugin`s inserted on the fly to
    /// customize the behavior of a connection.
    pub fn new(exports_func: fn(&mut Store, &FunctionEnv<Env<P>>) -> Exports) -> Self {
        Self {
            store: create_store(),
            conn: RawPtr::null(),
            exports_func,
            plugins: PluginArray { array: Vec::new() },
            _pin: PhantomPinned,
        }
    }

    /// Attaches a new plugin whose bytecode is accessible through the provided path. Returns `true`
    /// if the insertion succeeded, `false` otherwise.
    ///
    /// If the insertion succeeds and the plugin provides an `init` function as a protocol
    /// operation, this function calls it. This can be useful to, e.g., initialize a plugin-specific
    /// structure or register new frames.
    ///
    /// When inserting the plugin, the caller provides the pointer to the connection context through
    /// `ptr`. **This pointer must be `Pin`**.
    pub fn insert_plugin(&mut self, plugin_fname: &PathBuf, conn: *const P) -> bool {
        if self.conn.is_null() {
            self.conn = RawPtr::new(conn);
        } else if !self.conn.ptr_eq(conn) {
            error!("Trying to attach a same PH to different connections");
            return false;
        }

        let self_ptr = self as *const _;
        let store = &mut self.store;
        let env = FunctionEnv::new(store, create_env(self_ptr));
        let exports = (self.exports_func)(store, &env);
        let imports = get_imports_with(exports, store, &env);
        match Plugin::new(plugin_fname, store, env, &imports) {
            Some(p) => {
                self.plugins.push(p);
                // Now the plugin is at its definitive area in memory, so we can initialize it.
                self.plugins
                    .last_mut()
                    .map(|p| p.initialize(store).is_ok())
                    .unwrap_or(false)
            }
            None => {
                error!("Failed to insert plugin with path {:?}", plugin_fname);
                false
            }
        }
    }

    pub fn provides(&self, po: &ProtoOp, anchor: Anchor) -> bool {
        self.plugins.provides(po, anchor)
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    fn call_internal(
        &mut self,
        pod: Option<&&ProtocolOperationDefault>,
        po: &ProtoOp,
        params: &[PluginVal],
    ) -> Result<Box<[PluginVal]>, Error> {
        // PRE part
        for p in self.plugins.iter() {
            if let Some(func) = p.get_func(po, Anchor::Pre) {
                p.call(&mut self.store, func, params)?;
            }
        }

        // REPLACE part
        let res = match self.plugins.get_first_plugin(po) {
            Some((p, func)) => p.call(&mut self.store, func, params)?,
            None => {
                match pod {
                    Some(_pod) => {
                        todo!()
                        // Gives back both transient arguments and named arguments.
                        // {
                        //     let mut transient_args = self.transient_args.lock().unwrap();
                        //     internal_args.transient = transient_args.take();
                        //     *transient_args = old_transient_args.take();
                        // }
                        // let ret = (pod.default_fn)(internal_args, params);
                        // match ret.downcast::<R>() {
                        //     Ok(r) => *r,
                        //     Err(e) => return Err(Error::OutputConversionError(format!("{:?} to TypeId {:?}", e, std::any::TypeId::of::<R>()))),
                        // }
                    }
                    None => return Err(Error::NoDefault(*po)),
                }
            }
        };

        // POST part
        for p in self.plugins.iter() {
            if let Some(func) = p.get_func(po, Anchor::Post) {
                p.call(&mut self.store, func, params)?;
            }
        }

        Ok(res)
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    pub fn call(&mut self, po: &ProtoOp, params: &[PluginVal]) -> Result<Box<[PluginVal]>, Error> {
        // trace!("Calling protocol operation {:?}", po);

        // TODO
        // let pod = self.default_protocol_operations.get(po);

        self.call_internal(None /* pod */, po, params)
    }
}
