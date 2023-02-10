use std::{
    cell::UnsafeCell,
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use log::error;
use pluginop_common::{Anchor, PluginOp, PluginVal};
use wasmer::{Engine, Exports, FunctionEnv, Store};
use wasmer_compiler_singlepass::Singlepass;

use crate::{
    api::get_imports_with,
    plugin::{create_env, Env, Plugin},
    rawptr::RawMutPtr,
    Error, PluginFunction, PluginizableConnection,
};

/// Get a store for plugins. Note that this function should be called once for a host.
fn create_store() -> Store {
    let compiler = Singlepass::new();
    let engine: Engine = compiler.into();
    Store::new(engine)
}

/// A pinned `Vec` of plugins.
struct PluginArray {
    /// The inner array.
    array: Vec<Plugin>,
}

impl Deref for PluginArray {
    type Target = Vec<Plugin>;

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl DerefMut for PluginArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl PluginArray {
    /// Returns `true` iif one of the plugins provides an implementation for the requested `po`.
    fn provides(&self, po: &PluginOp, anchor: Anchor) -> bool {
        self.iter().any(|p| p.get_func(po, anchor).is_some())
    }

    /// Returns the first plugin that provides an implementation for `po` with the implementing
    /// function, or `None` if there is not.
    fn get_first_plugin(&self, po: &PluginOp) -> Option<(&Plugin, &PluginFunction)> {
        for p in self.iter() {
            if let Some(func) = p.get_func(po, Anchor::Replace) {
                return Some((p, func));
            }
        }
        None
    }
}

pub struct PluginHandler {
    /// The store that served to instantiate plugins.
    store: UnsafeCell<Store>,
    /// A pointer to the serving session. It can stay null if no plugin is inserted.
    conn: RawMutPtr<Box<dyn PluginizableConnection>>,
    /// Function creating an `Imports`.
    exports_func: fn(&mut Store, &FunctionEnv<Env>) -> Exports,
    /// The actual container of the plugins.
    plugins: PluginArray,
    /// Force this structure to be pinned.
    _pin: PhantomPinned,
}

impl std::fmt::Debug for PluginHandler {
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

impl PluginHandler {
    /// Creates a new `PluginHandler`, enabling the execution of `Plugin`s inserted on the fly to
    /// customize the behavior of a connection.
    pub fn new(exports_func: fn(&mut Store, &FunctionEnv<Env>) -> Exports) -> Self {
        Self {
            store: UnsafeCell::new(create_store()),
            conn: RawMutPtr::null(),
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
    pub fn insert_plugin(
        &mut self,
        plugin_fname: &PathBuf,
        conn: *const Box<dyn PluginizableConnection>,
    ) -> bool {
        if self.conn.is_null() {
            self.conn = RawMutPtr::new(conn as *const _ as *mut _)
        } else if !self.conn.ptr_eq(conn as *const _ as *mut _) {
            error!("Trying to attach a same PH to different connections");
            return false;
        }
        let ph_ptr = self as *mut _;
        let store = unsafe { &mut *self.store.get() };
        let env = FunctionEnv::new(store, create_env(RawMutPtr::new(ph_ptr)));
        let exports = (self.exports_func)(store, &env);
        let imports = get_imports_with(exports, store, &env);
        match Plugin::new(plugin_fname, store, env, &imports) {
            Some(p) => {
                self.plugins.push(p);
                // Now the plugin is at its definitive area in memory, so we can initialize it.
                self.plugins
                    .last_mut()
                    .map(|p| p.initialize(unsafe { &mut *self.store.get() }).is_ok())
                    .unwrap_or(false)
            }
            None => {
                error!("Failed to insert plugin with path {:?}", plugin_fname);
                false
            }
        }
    }

    pub fn provides(&self, po: &PluginOp, anchor: Anchor) -> bool {
        self.plugins.provides(po, anchor)
    }

    /// Gets an immutable reference to the serving connection.
    pub fn get_conn(&self) -> Option<&dyn PluginizableConnection> {
        if self.conn.is_null() {
            None
        } else {
            // SAFETY: The pluginizable conn is pinned and implements `!Unpin`.
            Some(unsafe { &***self.conn })
        }
    }

    /// Gets an mutable reference to the serving connection.
    pub fn get_conn_mut(&mut self) -> Option<&mut dyn PluginizableConnection> {
        if self.conn.is_null() {
            None
        } else {
            // SAFETY: The pluginizable conn is pinned and implements `!Unpin`.
            Some(unsafe { &mut ***self.conn })
        }
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    fn call_internal(
        &self,
        pod: Option<&&ProtocolOperationDefault>,
        po: &PluginOp,
        params: &[PluginVal],
    ) -> Result<Box<[PluginVal]>, Error> {
        // PRE part
        for p in self.plugins.iter() {
            if let Some(func) = p.get_func(po, Anchor::Pre) {
                p.call(unsafe { &mut *self.store.get() }, func, params)?;
            }
        }

        // REPLACE part
        let res = match self.plugins.get_first_plugin(po) {
            Some((p, func)) => p.call(unsafe { &mut *self.store.get() }, func, params)?,
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
                p.call(unsafe { &mut *self.store.get() }, func, params)?;
            }
        }

        Ok(res)
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    pub fn call(&self, po: &PluginOp, params: &[PluginVal]) -> Result<Box<[PluginVal]>, Error> {
        // trace!("Calling protocol operation {:?}", po);

        // TODO
        // let pod = self.default_protocol_operations.get(po);

        self.call_internal(None /* pod */, po, params)
    }
}
