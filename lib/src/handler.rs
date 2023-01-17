use std::{pin::Pin, marker::PhantomPinned, path::PathBuf, sync::{Arc, Mutex}, collections::BTreeSet, any::Any};

use log::error;
use pluginop_common::{ProtoOp, Anchor};
use wasmer::{Store, Engine, Imports, FunctionEnv, Value, Function};
use wasmer_compiler_singlepass::Singlepass;

use crate::{PluginizableConnection, plugin::{Plugin, Env, create_env, RawPtr}, Error};

/// Get a store for plugins. Note that this function should be called once for a host.
fn create_store() -> Store {
    let compiler = Singlepass::new();
    let engine:Engine = compiler.into();
    Store::new(engine)
}

pub struct PluginHandler<P: PluginizableConnection> {
    /// The store that served to instantiate plugins.
    store: Arc<Mutex<Store>>,
    /// A pointer to the serving session. It can stay null if no plugin is inserted.
    conn: RawPtr<P>,
    /// Function creating an `Imports`.
    imports_func: fn(&mut Store, &FunctionEnv<Env<P>>) -> Imports,
    /// The actual container of the plugins.
    plugins: Pin<Box<Vec<Plugin<P>>>>,
    /// Opaque value provided as argument to the plugin.
    plugin_state: u32,
    /// Force this structure to be pinned.
    _pin: PhantomPinned,
}

impl<P: PluginizableConnection> std::fmt::Debug for PluginHandler<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHandler").field("store", &self.store).field("_pin", &self._pin).finish()
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

/// Structure containing protocol operation arguments that are hidden from plugins.
#[derive(Default)]
pub struct InternalArgs;

/// A default implementation of a protocol operation proposed by the host implementation.
pub struct ProtocolOperationDefault {
    po: ProtoOp,
    default_fn: fn(InternalArgs, &[Value]) -> Box<dyn Any>,
    // return_type: POReturnType,
    named_args: Vec<&'static str>,
    named_refs: Vec<&'static str>,
    named_muts: Vec<&'static str>,
    // use_transient: UseTransientStructs,
}

impl<P: PluginizableConnection> PluginHandler<P> where {
    pub fn new(imports_func: fn(&mut Store, &FunctionEnv<Env<P>>) -> Imports) -> Self {
        let mut plugin_state = [0u8; 4];
        getrandom::getrandom(&mut plugin_state).expect("cannot generate random");
        Self {
            store: Arc::new(Mutex::new(create_store())),
            conn: RawPtr::null(),
            imports_func,
            plugins: Box::pin(Vec::new()),
            plugin_state: u32::from_be_bytes(plugin_state),
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
        &mut self, plugin_fname: &PathBuf, conn: *const P,
    ) -> bool {
        if self.conn.is_null() {
            self.conn = RawPtr::new(conn);
        } else if !self.conn.ptr_eq(conn) {
            error!("Trying to attach a same PH to different connections");
            return false;
        }

        let store = &mut *self.store.lock().unwrap();
        let env = FunctionEnv::new(
            store,
            create_env(self)
        );
        let imports = (self.imports_func)(store, &env);
        match Plugin::new(plugin_fname, store, env, &imports) {
            Some(p) => {
                self.plugins.push(p);
                // Now the plugin is at its definitive area in memory, so we can initialize it.
                self.plugins.last_mut().unwrap().initialize(store);
                true
            },
            None => {
                error!("Failed to insert plugin with path {:?}", plugin_fname);
                false
            },
        }
    }

    /// Returns `true` iif one of the plugins provides an implementation for the requested `po`.
    pub fn provides(&self, po: &ProtoOp, anchor: Anchor) -> bool {
        for p in self.plugins.iter() {
            if let Some(_) = p.get_func(po, anchor) {
                return true;
            }
        }
        false
    }

    /// Returns the first plugin that provides an implementation for `po` with the implementing
    /// function, or `None` if there is not.
    fn get_first_plugin(&self, po: &ProtoOp) -> Option<(&Plugin<P>, &Function)> {
        for p in self.plugins.iter() {
            match p.get_func(po, Anchor::Replace) {
                Some(func) => return Some((p, func)),
                None => {},
            }
        }
        None
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    fn call_internal<R: 'static, B, A>(
        &self, pod: Option<&&ProtocolOperationDefault>, po: &ProtoOp, params: &[Value],
        mut before_call: B, after_call: A, mut internal_args: InternalArgs,
    ) -> Result<R, Error>
    where
        B: FnMut(&Env<P>),
        A: FnMut(&Env<P>, Box<[Value]>) -> R,
    {
        // We have to handle transient arguments.
        // let mut old_transient_args = {
        //     let mut transient_args = self.transient_args.lock().unwrap();
        //     let tmp = transient_args.take();
        //     *transient_args = internal_args.transient.take();
        //     tmp
        // };

        let mut store = self.store.lock().unwrap();
        let store = &mut *store;

        // PRE part
        for p in self.plugins.iter() {
            match p.get_func(po, Anchor::Pre) {
                Some(func) => p.call(store, func, params, &mut before_call, |_, _| {})?,
                None => {},
            }
        }

        // REPLACE part
        let res = match self.get_first_plugin(po) {
            Some((p, func)) => p.call(store, func, params, &mut before_call, after_call)?,
            None => {
                match pod {
                    Some(pod) => {
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
                    },
                    None => return Err(Error::NoDefault(*po)),
                }
            },
        };

        // POST part
        for p in self.plugins.iter() {
            match p.get_func(po, Anchor::Post) {
                Some(func) => p.call(store, func, params, &mut before_call, |_, _| {})?,
                None => {},
            }
        }

        // Finally, we have to clean up the transient arguments. We must remove all the previously
        // added transient arguments that were consumed by this protocol operation. The only ones
        // that are consumed are plain arguments. This happens only if a plugin was inserted at the
        // REPLACE anchor.
        // if let Some(otas) = &mut old_transient_args {
        //     if let Some(po) = pod {
        //         if let Some(op) = &mut otas.others_plain {
        //             op.retain(|na| !po.named_args.contains(&na.name));
        //         }
        //     }
        // }
        // {
        //     let mut transient_args = self.transient_args.lock().unwrap();
        //     *transient_args = old_transient_args.take();
        // }

        Ok(res)
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    pub fn call<R: 'static, B, A>(
        &self, po: &ProtoOp, params: &[Value], before_call: B, after_call: A,
        internal_args: InternalArgs,
    ) -> Result<R, Error>
    where
        B: FnMut(&Env<P>),
        A: FnMut(&Env<P>, Box<[Value]>) -> R,
    {
        // trace!("Calling protocol operation {:?}", po);

        // TODO
        // let pod = self.default_protocol_operations.get(po);

        if params.len() > 0 { todo!("handle params") }
        println!("Using state {}", self.plugin_state);

        self.call_internal(None /* pod */, po, &[self.plugin_state.into()], before_call, after_call, internal_args)
    }
}