use std::{
    collections::BTreeSet,
    fmt::{Debug, Pointer},
    ops::Deref,
    path::PathBuf,
    pin::Pin,
};

use fnv::FnvHashMap;
use log::error;
use pluginop_common::{Anchor, ProtoOp};
use wasmer::{Function, FunctionEnv, Imports, Instance, Module, Store, Value};

use crate::{
    handler::{Permission, PluginHandler},
    Error, POCode, PluginizableConnection,
};

pub struct RawPtr<T: ?Sized> {
    inner: *const T,
}

impl<T: ?Sized> RawPtr<T> {
    pub fn new(ptr: *const T) -> Self {
        Self { inner: ptr }
    }

    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

impl<T: ?Sized> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<T: ?Sized> Copy for RawPtr<T> {}

impl<T: ?Sized> Deref for RawPtr<T> {
    type Target = *const T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> PartialEq for RawPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Sized> RawPtr<T> {
    pub fn null() -> Self {
        Self {
            inner: std::ptr::null(),
        }
    }

    pub fn ptr_eq(&self, ptr: *const T) -> bool {
        std::ptr::eq(self.inner, ptr)
    }
}

impl<T: ?Sized> Debug for RawPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pointer::fmt(&self.inner, f)
        //f.debug_struct("RawPtr").field("inner", &self.inner).finish()
    }
}

// SAFETY: Only true if T is pinned.
unsafe impl<T: ?Sized> Send for RawPtr<T> {}

// SAFETY: Only true if T is pinned.
unsafe impl<T: ?Sized> Sync for RawPtr<T> {}

#[derive(Debug)]
pub struct Env<P: PluginizableConnection> {
    /// The raw pointer to the plugin handler. Because `PluginHandler` is pinned,
    /// this is safe.
    _ph: RawPtr<PluginHandler<P>>,
    /// The set of internal field permissions granted to the plugin.
    permissions: BTreeSet<Permission>,
    /// Whether the associated plugin was initialized or not.
    initialized: bool,
}

pub(crate) fn create_env<P: PluginizableConnection>(ph: *const PluginHandler<P>) -> Env<P> {
    Env {
        _ph: RawPtr { inner: ph },
        permissions: BTreeSet::new(),
        initialized: false,
    }
}

impl<P: PluginizableConnection> Env<P> {
    fn sanitize(&mut self) { /* Placeholder */
    }
}

/// Structure holding the state of an inserted plugin. Because all the useful state is hold in the
/// `Env` structure, this structure does not need to be public anymore.
#[derive(Debug)]
pub(crate) struct Plugin<P: PluginizableConnection> {
    /// The actual WASM instance.
    _instance: Pin<Box<Instance>>,
    // The environment accessible to plugins.
    env: FunctionEnv<Env<P>>,
    /// A hash table to the functions contained in the instance.
    pocodes: Pin<Box<FnvHashMap<ProtoOp, POCode>>>,
}

impl<P: PluginizableConnection> Plugin<P> {
    /// Creates a new `Plugin` instance.
    pub fn new(
        plugin_fname: &PathBuf,
        store: &mut Store,
        env: FunctionEnv<Env<P>>,
        imports: &Imports,
    ) -> Option<Self> {
        match std::fs::read(plugin_fname) {
            Ok(wasm) => {
                let module = Module::from_binary(store, &wasm).expect("wasm compilation");

                match Instance::new(store, &module, imports) {
                    Ok(instance) => {
                        // XXX We could update the permissions later.
                        let permissions = &mut env.as_mut(store).permissions;
                        permissions.insert(Permission::Output);
                        permissions.insert(Permission::Opaque);
                        permissions.insert(Permission::ConnectionAccess);
                        permissions.insert(Permission::WriteBuffer);
                        permissions.insert(Permission::ReadBuffer);

                        let pocodes = Plugin::<P>::get_pocodes(&instance);

                        return Some(Plugin {
                            _instance: Box::pin(instance),
                            env,
                            pocodes: Box::pin(pocodes),
                        });
                    }
                    Err(e) => {
                        error!("Cannot instantiate plugin: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Cannot read plugin: {}", e);
            }
        }
        None
    }

    fn get_pocodes(instance: &Instance) -> FnvHashMap<ProtoOp, POCode> {
        let mut pocodes: FnvHashMap<ProtoOp, POCode> = FnvHashMap::default();

        for (name, _) in instance.exports.iter() {
            if let Ok(func) = instance.exports.get_function(name) {
                let func = func.clone();

                let (po, a) = ProtoOp::from_name(name);
                match pocodes.get_mut(&po) {
                    Some(poc) => match a {
                        Anchor::Pre => poc.pre = Some(func),
                        Anchor::Replace => poc.replace = Some(func),
                        Anchor::Post => poc.post = Some(func),
                    },
                    None => {
                        let mut poc = POCode::default();
                        match a {
                            Anchor::Pre => poc.pre = Some(func),
                            Anchor::Replace => poc.replace = Some(func),
                            Anchor::Post => poc.post = Some(func),
                        }
                        pocodes.insert(po, poc);
                    }
                };
            }
        }

        pocodes
    }

    /// Returns the function providing code for the requested protocol operation and anchor.
    pub(crate) fn get_func(&self, po: &ProtoOp, anchor: Anchor) -> Option<&Function> {
        match self.pocodes.get(po) {
            Some(poc) => match anchor {
                Anchor::Pre => poc.pre.as_ref(),
                Anchor::Replace => poc.replace.as_ref(),
                Anchor::Post => poc.post.as_ref(),
            },
            None => None,
        }
    }

    /// Initializes the plugin.
    pub(crate) fn initialize(&self, store: &mut Store) {
        let env_mut = self.env.as_mut(store);
        env_mut.initialized = true;

        // And call a potential `init` method provided by the plugin.
        let po = ProtoOp::Init;
        if let Some(func) = self.get_func(&po, Anchor::Replace) {
            self.call(store, func, &[], &mut |_| {}, |_, _| {})
                .expect("error in init");
        }
    }

    /// Invokes the function called `function_name` with provided `params`.
    pub fn call<B, A, R>(
        &self,
        store: &mut Store,
        func: &Function,
        params: &[Value],
        before_call: &mut B,
        mut after_call: A,
    ) -> Result<R, Error>
    where
        B: FnMut(&Env<P>),
        A: FnMut(&Env<P>, Box<[Value]>) -> R,
    {
        let env_mut = self.env.as_mut(store);
        // Before launching any call, we should sanitize the running `env`.
        env_mut.sanitize();

        before_call(self.env.as_ref(store));
        // debug!("Calling PO with param {:?}", params);
        match func.call(store, params) {
            Ok(res) => Ok(after_call(self.env.as_ref(store), res)),
            Err(re) => Err(Error::RuntimeError(re)),
        }
    }
}
