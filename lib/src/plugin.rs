use std::{
    collections::BTreeSet,
    fmt::Debug,
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Weak},
};

use fnv::FnvHashMap;
use log::{error, warn};
use pluginop_common::{Anchor, PluginVal, ProtoOp};
use wasmer::{FunctionEnv, Imports, Instance, Module, Store};

use crate::{
    handler::{Permission, PluginHandler},
    rawptr::RawMutPtr,
    Error, POCode, PluginFunction, PluginizableConnection,
};

/// An array of plugin-compatible values.
#[derive(Debug, Default)]
pub struct PluginValArray {
    inner: Vec<PluginVal>,
    _pin: PhantomPinned,
}

impl Deref for PluginValArray {
    type Target = Vec<PluginVal>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PluginValArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct Env<P: PluginizableConnection> {
    /// The underlying plugin handler holding the plugin running this environment.
    ph: RawMutPtr<PluginHandler<P>>,
    /// The (weak) reference to the instance of the plugin. The value is set when
    instance: Weak<Pin<Box<Instance>>>,
    /// The set of internal field permissions granted to the plugin.
    permissions: BTreeSet<Permission>,
    /// Whether the associated plugin was initialized or not.
    initialized: bool,
    /// Contains the inputs specific to the called operation.
    pub inputs: Pin<PluginValArray>,
    /// Enables a plugin to output more than one (serializable) value, as returning more than 1
    /// output in a function is not FFI safe.
    pub outputs: Pin<PluginValArray>,
    /// Store for opaque values used by the plugin. Typically, it contains pointers, and WASM
    /// pointers are 32-bit values.
    pub opaque_values: Pin<Box<FnvHashMap<u64, u32>>>,
}

pub(crate) fn create_env<P: PluginizableConnection>(ph: RawMutPtr<PluginHandler<P>>) -> Env<P> {
    Env {
        ph,
        instance: Weak::new(),
        permissions: BTreeSet::new(),
        initialized: false,
        inputs: Pin::new(PluginValArray::default()),
        outputs: Pin::new(PluginValArray::default()),
        opaque_values: Box::pin(FnvHashMap::default()),
    }
}

impl<P: PluginizableConnection> Env<P> {
    fn sanitize(&mut self) {
        // Empty the inputs.
        self.inputs.clear();
        // And the outputs.
        self.outputs.clear();
    }

    pub fn get_instance(&self) -> Option<Arc<Pin<Box<Instance>>>> {
        self.instance.upgrade()
    }

    pub fn get_ph(&mut self) -> Option<&mut PluginHandler<P>> {
        if self.ph.is_null() {
            None
        } else {
            // SAFETY: The plugin handler has the `PhantomPinned` marker, but we need to take care
            // of the mutable calls on it.
            Some(unsafe { &mut **self.ph })
        }
    }
}

/// Structure holding the state of an inserted plugin. Because all the useful state is hold in the
/// `Env` structure, this structure does not need to be public anymore.
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
                let module = match Module::from_binary(store, &wasm) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("failed WASM compilation: {}", e);
                        return None;
                    }
                };

                match Instance::new(store, &module, imports) {
                    Ok(instance) => {
                        let mut plugin_state = [0u8; 4];

                        if let Err(e) = getrandom::getrandom(&mut plugin_state) {
                            warn!("cannot generate random plugin state: {}", e);
                        }

                        // XXX We could update the permissions later.
                        let permissions = &mut env.as_mut(store).permissions;
                        permissions.insert(Permission::Output);
                        permissions.insert(Permission::Opaque);
                        permissions.insert(Permission::ConnectionAccess);
                        permissions.insert(Permission::WriteBuffer);
                        permissions.insert(Permission::ReadBuffer);

                        let pocodes = Plugin::<P>::get_pocodes(&instance, store);

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

    fn get_pocodes(instance: &Instance, store: &mut Store) -> FnvHashMap<ProtoOp, POCode> {
        let mut pocodes: FnvHashMap<ProtoOp, POCode> = FnvHashMap::default();

        for (name, _) in instance.exports.iter() {
            if let Ok(func) = instance.exports.get_typed_function(store, name) {
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
    pub(crate) fn get_func(&self, po: &ProtoOp, anchor: Anchor) -> Option<&PluginFunction> {
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
    pub(crate) fn initialize(&self, store: &mut Store) -> Result<(), Error> {
        let env_mut = self.env.as_mut(store);
        env_mut.initialized = true;

        // Set now the instance backpointer.
        env_mut.instance = Arc::<Pin<Box<Instance>>>::downgrade(&self.instance);

        // And call a potential `init` method provided by the plugin.
        let po = ProtoOp::Init;
        if let Some(func) = self.get_func(&po, Anchor::Replace) {
            self.call(store, func, &[])?;
        }
        Ok(())
    }

    /// Invokes the function called `function_name` with provided `params`.
    pub fn call(
        &self,
        store: &mut Store,
        func: &PluginFunction,
        params: &[PluginVal],
    ) -> Result<Box<[PluginVal]>, Error> {
        let env_mut = self.env.as_mut(store);
        // Before launching any call, we should sanitize the running `env`.
        env_mut.sanitize();

        for p in params {
            env_mut.inputs.push(*p);
        }

        // debug!("Calling PO with param {:?}", params);
        match func.call(store, self.plugin_state) {
            Ok(res) if res == 0 => Ok((*self.env.as_ref(store).outputs).clone().into_boxed_slice()),
            Ok(err) => Err(Error::OperationError(err)),
            Err(re) => Err(Error::RuntimeError(re)),
        }
    }
}
