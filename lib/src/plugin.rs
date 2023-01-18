use std::{
    collections::BTreeSet,
    fmt::{Debug, Pointer},
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Weak},
};

use fnv::FnvHashMap;
use log::error;
use pluginop_common::{Anchor, Input, ProtoOp};
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

#[derive(Debug, Default)]
pub struct OutputArray {
    inner: Vec<Vec<u8>>,
    _pin: PhantomPinned,
}

impl Deref for OutputArray {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OutputArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Default)]
pub struct InputArray {
    inner: Vec<Input>,
    _pin: PhantomPinned,
}

impl Deref for InputArray {
    type Target = Vec<Input>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for InputArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct Env<P: PluginizableConnection> {
    /// The raw pointer to the plugin handler. Because `PluginHandler` is pinned,
    /// this is safe.
    _ph: RawPtr<PluginHandler<P>>,
    /// The (weak) reference to the instance of the plugin. The value is set when
    instance: Weak<Pin<Box<Instance>>>,
    /// The set of internal field permissions granted to the plugin.
    permissions: BTreeSet<Permission>,
    /// Whether the associated plugin was initialized or not.
    initialized: bool,
    /// Contains the inputs specific to the called operation.
    pub inputs: Pin<InputArray>,
    /// Enables a plugin to output more than one (serializable) value, as returning more than 1
    /// output in a function is not FFI safe.
    pub outputs: Pin<OutputArray>,
    /// Store for opaque values used by the plugin. Typically, it contains pointers, and WASM
    /// pointers are 32-bit values.
    pub opaque_values: Pin<Box<FnvHashMap<u64, u32>>>,
}

pub(crate) fn create_env<P: PluginizableConnection>(ph: *const PluginHandler<P>) -> Env<P> {
    Env {
        _ph: RawPtr { inner: ph },
        instance: Weak::new(),
        permissions: BTreeSet::new(),
        initialized: false,
        inputs: Pin::new(InputArray::default()),
        outputs: Pin::new(OutputArray::default()),
        opaque_values: Box::pin(FnvHashMap::default()),
    }
}

impl<P: PluginizableConnection> Env<P> {
    fn sanitize(&mut self) {
        // Empty the inputs.
        self.inputs.clear();
    }

    fn _get_ph(&self) -> &PluginHandler<P> {
        // SAFETY: This is valid since `PluginHandler` is pinned and cannot be moved.
        unsafe { &**self._ph }
    }

    pub fn get_instance(&self) -> Arc<Pin<Box<Instance>>> {
        self.instance.upgrade().unwrap()
    }
}

/// Structure holding the state of an inserted plugin. Because all the useful state is hold in the
/// `Env` structure, this structure does not need to be public anymore.
#[derive(Debug)]
pub(crate) struct Plugin<P: PluginizableConnection> {
    /// The actual WASM instance.
    instance: Arc<Pin<Box<Instance>>>,
    // The environment accessible to plugins.
    env: FunctionEnv<Env<P>>,
    /// A hash table to the functions contained in the instance.
    pocodes: Pin<Box<FnvHashMap<ProtoOp, POCode>>>,
    /// Opaque value provided as argument to the plugin.
    plugin_state: u32,
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
                        let mut plugin_state = [0u8; 4];
                        getrandom::getrandom(&mut plugin_state).expect("cannot generate random");

                        // XXX We could update the permissions later.
                        let permissions = &mut env.as_mut(store).permissions;
                        permissions.insert(Permission::Output);
                        permissions.insert(Permission::Opaque);
                        permissions.insert(Permission::ConnectionAccess);
                        permissions.insert(Permission::WriteBuffer);
                        permissions.insert(Permission::ReadBuffer);

                        let pocodes = Plugin::<P>::get_pocodes(&instance);

                        return Some(Plugin {
                            instance: Arc::new(Box::pin(instance)),
                            env,
                            pocodes: Box::pin(pocodes),
                            plugin_state: u32::from_be_bytes(plugin_state),
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

        // Set now the instance backpointer.
        env_mut.instance = Arc::<Pin<Box<Instance>>>::downgrade(&self.instance);

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
        params: &[Input],
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

        for p in params {
            env_mut.inputs.push(*p);
        }

        before_call(self.env.as_ref(store));
        // debug!("Calling PO with param {:?}", params);
        match func.call(store, &[self.plugin_state.into()]) {
            Ok(res) => Ok(after_call(self.env.as_ref(store), res)),
            Err(re) => Err(Error::RuntimeError(re)),
        }
    }
}
