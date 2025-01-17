//! Operations relative to a single loaded plugin.

use std::{
    cell::UnsafeCell,
    collections::BTreeSet,
    fmt::Debug,
    fs::File,
    io::Write,
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Weak},
    time::Instant,
};

use fnv::FnvHashMap;
use log::{error, warn};
use pluginop_common::{Anchor, PluginInputType, PluginOp, PluginOutputType, PluginVal};
use pluginop_rawptr::RawMutPtr;
use wasmer::{FunctionEnv, Instance, Module, Store, TypedFunction};

use crate::{
    api::{get_imports_with, CTPError, ConnectionToPlugin},
    handler::PluginHandler,
    Error, Permission,
};

pub type PluginFunction = TypedFunction<PluginInputType, PluginOutputType>;

#[derive(Default)]
struct POCode {
    before: Option<PluginFunction>,
    define: Option<PluginFunction>,
    after: Option<PluginFunction>,
}

impl Debug for POCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("POCode")
            .field("before", &self.before.is_some())
            .field("define", &self.define.is_some())
            .field("after", &self.after.is_some())
            .finish()
    }
}

impl POCode {
    /// Get the underlying PluginFunction associated to the provided `Anchor`.
    pub(crate) fn get(&self, a: Anchor) -> Option<&PluginFunction> {
        match a {
            Anchor::Before => self.before.as_ref(),
            Anchor::Define => self.define.as_ref(),
            Anchor::After => self.after.as_ref(),
        }
    }
}

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

/// A timer, requested by a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TimerEvent {
    /// When the timer event should take place.
    at: Instant,
    /// The internal identifier. This identifier is unique within the plugin.
    id: u64,
    /// The timer identifier.
    timer_id: u64,
}

impl TimerEvent {
    pub fn new(at: Instant, id: u64, timer_id: u64) -> Self {
        Self { at, id, timer_id }
    }
}

/// A companion structure to the plugin execution environment, containing plugin-specific
/// data allowing the bytecode to interact with the host implementation.
pub struct Env<CTP: ConnectionToPlugin> {
    /// The underlying plugin handler holding the plugin running this environment.
    ph: RawMutPtr<PluginHandler<CTP>>,
    /// The (weak) reference to the instance of the plugin. The value is set when
    /// initializing the plugin.
    instance: Weak<Pin<Box<Instance>>>,
    /// The set of internal field permissions granted to the plugin.
    permissions: BTreeSet<Permission>,
    /// Whether the associated plugin was initialized or not.
    initialized: bool,
    /// Initially, a plugin is only pre-loaded, i.e., only a very small subset of the
    /// functions are enabled. The API enables the plugin to be fully loaded by using
    /// a dedicated API call.
    enabled: bool,
    /// The next timeout events to fire.
    timer_events: Vec<TimerEvent>,
    /// Contains the inputs specific to the called operation.
    pub inputs: Pin<PluginValArray>,
    /// Enables a plugin to output more than one (serializable) value, as returning more than 1
    /// output in a function is not FFI safe.
    pub outputs: Pin<PluginValArray>,
    /// The files currently in use by the underlying plugin.
    files: Vec<UnsafeCell<File>>,
}

pub(crate) fn create_env<CTP: ConnectionToPlugin>(ph: RawMutPtr<PluginHandler<CTP>>) -> Env<CTP> {
    Env {
        ph,
        instance: Weak::new(),
        permissions: BTreeSet::new(),
        initialized: false,
        enabled: false,
        timer_events: Vec::new(),
        inputs: Pin::new(PluginValArray::default()),
        outputs: Pin::new(PluginValArray::default()),
        files: Vec::new(),
    }
}

impl<CTP: ConnectionToPlugin> Env<CTP> {
    fn sanitize(&mut self) {
        // Empty the inputs.
        self.inputs.clear();
        // And the outputs.
        self.outputs.clear();
    }

    pub(crate) fn get_instance(&self) -> Option<Arc<Pin<Box<Instance>>>> {
        self.instance.upgrade()
    }

    pub(crate) fn get_ph(&mut self) -> Option<&mut PluginHandler<CTP>> {
        if self.ph.is_null() {
            None
        } else {
            // SAFETY: The plugin handler has the `PhantomPinned` marker, but we need to take care
            // of the mutable calls on it.
            Some(unsafe { &mut **self.ph })
        }
    }

    pub(crate) fn get_bytes(
        &mut self,
        tag: usize,
        len: usize,
        mem: &mut [u8],
    ) -> Result<usize, CTPError> {
        let ph = self.get_ph().ok_or(CTPError::BadBytes)?;
        let bc = ph.get_mut_bytes_content(tag)?;
        if len > bc.read_len() {
            warn!(
                "Plugin requested {} bytes, but only {} left",
                len,
                bc.read_len()
            );
            return Err(CTPError::BadBytes);
        }
        bc.write_into(len, mem)
    }

    pub(crate) fn put_bytes(&mut self, tag: usize, mem: &[u8]) -> Result<usize, CTPError> {
        let ph = self.get_ph().ok_or(CTPError::BadBytes)?;
        let bc = ph.get_mut_bytes_content(tag)?;
        // TODO: limit the length that plugins should be able to write.
        bc.extend_from(mem)
    }

    fn timeout(&self) -> Option<Instant> {
        self.timer_events.first().map(|r| r.at)
    }

    /// Insert a timer event.
    pub(crate) fn insert_timer_event(&mut self, v: TimerEvent) {
        // If there is an element where the id is already there, update it.
        if let Some(te) = self.timer_events.iter_mut().find(|te| te.id == v.id) {
            *te = v;
        } else {
            self.timer_events.push(v);
        }
        // Always ensure the structure is sorted.
        self.timer_events.sort();
    }

    /// Pop a fired timer event, if any.
    pub(crate) fn pop_timer_event_if_earlier_than(&mut self, t: Instant) -> Option<TimerEvent> {
        if let Some(te) = self.timer_events.first() {
            if te.at <= t {
                // This is safe since we just checked that such an element exists.
                // Note that the Vec is still sorted.
                return Some(self.timer_events.remove(0));
            }
        }
        None
    }

    /// Cancel a timer.
    pub(crate) fn cancel_timer_event(&mut self, id: u64) -> Option<TimerEvent> {
        let mut cancelled = None;
        // This works, because we can only have a single id inside the timer events.
        self.timer_events.retain(|te| {
            if te.id == id {
                cancelled = Some(*te);
                false
            } else {
                true
            }
        });
        cancelled
    }

    pub(crate) fn create_file_with_path(&mut self, path: &Path) -> Result<i64, CTPError> {
        // TODO: we need to check whether we have the permisison to create the file.
        // TODO: secured path location (avoid /etc/passwd vulnerabilities)
        match File::create(path) {
            Ok(f) => {
                // Don't let the plugins directly handle files.
                let fd = self.files.len();
                self.files.push(UnsafeCell::new(f));
                Ok(fd as i64)
            }
            Err(e) => {
                error!("plugin: cannot create file: {:?}", e);
                Err(CTPError::FileError)
            }
        }
    }

    /// Write the content of the buffer in the provided file descriptor.
    pub(crate) fn write_to_file(&self, fd: i64, buf: &[u8]) -> Result<usize, CTPError> {
        if fd < 0 {
            return Err(CTPError::FileError);
        }
        match self.files.get(fd as usize) {
            Some(ucf) => {
                let f = unsafe {
                    // SAFETY: This is fine as long as plugins are single-threaded, no
                    // concurrent writes on a same file.
                    &mut *ucf.get()
                };
                match f.write(buf) {
                    Ok(w) => Ok(w),
                    Err(_) => Err(CTPError::FileError),
                }
            }
            None => Err(CTPError::FileError),
        }
    }

    /// Fully enable the plugin operations.
    pub(crate) fn enable(&mut self) {
        self.enabled = true;
    }
}

const KV_VEC_MAX_ELEMS: usize = 16;

#[derive(Debug)]
enum KeyValueCollectionInner<K, V> {
    Vec(Vec<(K, V)>),
    HashMap(FnvHashMap<K, V>),
}

impl<K, V> KeyValueCollectionInner<K, V>
where
    K: Eq + core::hash::Hash,
{
    fn is_vec(&self) -> bool {
        matches!(self, KeyValueCollectionInner::Vec(_))
    }

    fn len(&self) -> usize {
        match self {
            KeyValueCollectionInner::Vec(v) => v.len(),
            KeyValueCollectionInner::HashMap(hm) => hm.len(),
        }
    }

    fn get(&self, k: &K) -> Option<&V> {
        match self {
            KeyValueCollectionInner::Vec(v) => {
                v.iter()
                    .find_map(|(ek, ev)| if ek == k { Some(ev) } else { None })
            }
            KeyValueCollectionInner::HashMap(hm) => hm.get(k),
        }
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        match self {
            KeyValueCollectionInner::Vec(v) => {
                v.iter_mut()
                    .find_map(|(ek, ev)| if ek == k { Some(ev) } else { None })
            }
            KeyValueCollectionInner::HashMap(hm) => hm.get_mut(k),
        }
    }

    fn insert(&mut self, k: K, v: V) {
        // FIXME: in Vec mode, we should ideally check that the element is not already there.
        match self {
            KeyValueCollectionInner::Vec(vec) => vec.push((k, v)),
            KeyValueCollectionInner::HashMap(hm) => {
                hm.insert(k, v);
            }
        }
    }
}

#[derive(Debug)]
struct KeyValueCollection<K, V> {
    inner: KeyValueCollectionInner<K, V>,
    capacity: usize,
}

impl<K, V> KeyValueCollection<K, V>
where
    K: Eq + core::hash::Hash,
{
    fn new(capacity: usize) -> Self {
        let inner = if capacity > KV_VEC_MAX_ELEMS {
            KeyValueCollectionInner::HashMap(FnvHashMap::default())
        } else {
            KeyValueCollectionInner::Vec(Vec::with_capacity(capacity))
        };
        Self { inner, capacity }
    }

    fn insert(&mut self, k: K, v: V) {
        // We mostly care for the Vec variant, not for the HashMap.
        if self.inner.is_vec() && self.inner.len() > self.capacity {
            warn!("Added element will exceed original Vec capacity");
        }
        self.inner.insert(k, v)
    }

    fn get(&self, k: &K) -> Option<&V> {
        self.inner.get(k)
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.inner.get_mut(k)
    }
}

/// Structure holding the state of an inserted plugin. Because all the useful state is hold in the
/// `Env` structure, this structure does not need to be public anymore.
pub(crate) struct Plugin<CTP: ConnectionToPlugin> {
    /// The actual WASM instance.
    instance: Arc<Pin<Box<Instance>>>,
    /// The store in which the plugin operates.
    store: Store,
    // The environment accessible to plugins.
    env: FunctionEnv<Env<CTP>>,
    /// A collection holding the plugin functions contained in the instance.
    pocodes: Pin<Box<KeyValueCollection<PluginOp, POCode>>>,
    /// Cache indicating whether the plugin has the anchor or not (Pre, Replace, Post).
    has_anchor: [bool; 3],
    /// Opaque value provided as argument to the plugin.
    plugin_state: u32,
}

impl<CTP: ConnectionToPlugin> Plugin<CTP> {
    /// Creates a new `Plugin` instance.
    pub fn new(plugin_fname: &PathBuf, ph: &PluginHandler<CTP>) -> Result<Self, Error> {
        match std::fs::read(plugin_fname) {
            Ok(wasm) => {
                let ph_ptr = ph as *const _ as *mut _;
                let mut store = Store::new(ph.get_cloned_engine());
                let env = FunctionEnv::new(&mut store, create_env(RawMutPtr::new(ph_ptr)));
                let exports = (ph.get_export_func())(&mut store, &env);
                let imports = get_imports_with(exports, &mut store, &env);
                let module = match Module::from_binary(&store, &wasm) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("failed WASM compilation: {}", e);
                        return Err(Error::PluginLoadingError(e.to_string()));
                    }
                };

                match Instance::new(&mut store, &module, &imports) {
                    Ok(instance) => {
                        let mut plugin_state = [0u8; 4];

                        if let Err(e) = getrandom::getrandom(&mut plugin_state) {
                            warn!("cannot generate random plugin state: {}", e);
                        }

                        // XXX We could update the permissions later.
                        let permissions = &mut env.as_mut(&mut store).permissions;
                        permissions.insert(Permission::Output);
                        permissions.insert(Permission::Opaque);
                        permissions.insert(Permission::ConnectionAccess);
                        permissions.insert(Permission::WriteBuffer);
                        permissions.insert(Permission::ReadBuffer);

                        let (pocodes, has_anchor) =
                            Plugin::<CTP>::get_pocodes(&instance, &mut store);

                        Ok(Plugin {
                            instance: Arc::new(Box::pin(instance)),
                            store,
                            env,
                            pocodes: Box::pin(pocodes),
                            has_anchor,
                            plugin_state: u32::from_be_bytes(plugin_state),
                        })
                    }
                    Err(e) => {
                        error!("Cannot instantiate plugin: {}", e);
                        Err(Error::PluginLoadingError(e.to_string()))
                    }
                }
            }
            Err(e) => {
                error!("Cannot read plugin: {}", e);
                Err(Error::PluginLoadingError(e.to_string()))
            }
        }
    }

    fn get_pocodes(
        instance: &Instance,
        store: &mut Store,
    ) -> (KeyValueCollection<PluginOp, POCode>, [bool; 3]) {
        let mut pocodes: KeyValueCollection<PluginOp, POCode> =
            KeyValueCollection::new(KV_VEC_MAX_ELEMS);
        let mut has_anchor = [false; 3];

        for (name, _) in instance.exports.iter() {
            if let Ok(func) = instance.exports.get_typed_function(store, name) {
                let func = func.clone();

                let (po, a) = PluginOp::from_name(name);
                has_anchor[a.index()] = true;
                match pocodes.get_mut(&po) {
                    Some(poc) => match a {
                        Anchor::Before => poc.before = Some(func),
                        Anchor::Define => poc.define = Some(func),
                        Anchor::After => poc.after = Some(func),
                    },
                    None => {
                        let mut poc = POCode::default();
                        match a {
                            Anchor::Before => poc.before = Some(func),
                            Anchor::Define => poc.define = Some(func),
                            Anchor::After => poc.after = Some(func),
                        }
                        pocodes.insert(po, poc);
                    }
                };
            }
        }

        (pocodes, has_anchor)
    }

    /// Returns the first timer event related to this plugin.
    pub(crate) fn timeout(&self) -> Option<Instant> {
        self.env.as_ref(&self.store).timeout()
    }

    /// Process the timeout events related to this plugin.
    pub(crate) fn on_timeout(&mut self, t: Instant) -> Result<(), Error> {
        while let Some(te) = self
            .env
            .as_mut(&mut self.store)
            .pop_timer_event_if_earlier_than(t)
        {
            self.call(&PluginOp::OnPluginTimeout(te.timer_id), Anchor::Define, &[])?;
        }

        Ok(())
    }

    /// Returns an array indicating whether there is any provided bytecode
    /// serving each anchor.
    pub(crate) fn has_anchor(&self) -> [bool; 3] {
        self.has_anchor
    }

    /// Returns whether this plugin provides behavior for the requested
    /// `PluginOp` and `Anchor`.
    pub(crate) fn provides(&self, po: &PluginOp, anchor: Anchor) -> bool {
        self.has_anchor[anchor.index()]
            && (self.env.as_ref(&self.store).enabled || po.always_enabled())
            && self
                .pocodes
                .get(po)
                .and_then(|poc| poc.get(anchor))
                .is_some()
    }

    /// Initializes the plugin.
    pub(crate) fn initialize(&mut self) -> Result<(), Error> {
        let env_mut = self.env.as_mut(&mut self.store);
        env_mut.initialized = true;

        // Set now the instance backpointer.
        env_mut.instance = Arc::<Pin<Box<Instance>>>::downgrade(&self.instance);

        // And call a potential `init` method provided by the plugin.
        match self.call(&PluginOp::Init, Anchor::Define, &[]) {
            Ok(_) | Err(Error::NoPluginFunction) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Force-enable the plugin.
    pub(crate) fn force_enable(&mut self) {
        self.env.as_mut(&mut self.store).enable();
    }

    /// Invokes the function called `function_name` with provided `params`.
    pub fn call(
        &mut self,
        po: &PluginOp,
        anchor: Anchor,
        params: &[PluginVal],
    ) -> Result<Vec<PluginVal>, Error> {
        let env_mut = self.env.as_mut(&mut self.store);
        // Before launching any call, we should sanitize the running `env`.
        env_mut.sanitize();

        if !env_mut.enabled && !po.always_enabled() {
            return Err(Error::Disabled);
        }

        for p in params {
            env_mut.inputs.push(*p);
        }

        let func = match self.pocodes.get(po) {
            Some(poc) => match anchor {
                Anchor::Before => poc.before.as_ref(),
                Anchor::Define => poc.define.as_ref(),
                Anchor::After => poc.after.as_ref(),
            },
            None => None,
        };

        let func = func.ok_or(Error::NoPluginFunction)?;
        // debug!("Calling PO with param {:?}", params);
        match func.call(&mut self.store, self.plugin_state) {
            Ok(0) => Ok((*self.env.as_ref(&self.store).outputs).clone()),
            Ok(err) => Err(Error::OperationError(err)),
            Err(re) => Err(Error::RuntimeError(re)),
        }
    }
}
