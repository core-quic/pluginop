//! The handling of pluginization operations.

use std::{
    marker::PhantomPinned,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::Instant,
};

use log::error;
use pluginop_common::{quic::Registration, Anchor, Bytes, PluginOp, PluginVal};
use unix_time::Instant as UnixInstant;
use wasmer::{Engine, Exports, FunctionEnv, Store};
use wasmer_compiler_singlepass::Singlepass;

use crate::{
    api::{CTPError, ConnectionToPlugin},
    plugin::{Env, Plugin},
    BytesContent, Error, PluginizableConnection,
};

use pluginop_rawptr::RawMutPtr;

/// Get a store for plugins. Note that this function should be called once for a host.
fn create_engine() -> Engine {
    let compiler = Singlepass::new();
    compiler.into()
}

/// A pinned `Vec` of plugins.
struct PluginArray<CTP: ConnectionToPlugin> {
    /// The inner array.
    array: Vec<Plugin<CTP>>,
}

impl<CTP: ConnectionToPlugin> Deref for PluginArray<CTP> {
    type Target = Vec<Plugin<CTP>>;

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<CTP: ConnectionToPlugin> DerefMut for PluginArray<CTP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl<CTP: ConnectionToPlugin> PluginArray<CTP> {
    /// Returns `true` iif one of the plugins provides an implementation for the requested `po`.
    fn provides(&self, po: &PluginOp, anchor: Anchor) -> bool {
        self.iter().any(|p| p.provides(po, anchor))
    }

    /// Returns the first plugin that provides an implementation for `po` with the implementing
    /// function, or `None` if there is not.
    fn get_first_plugin(&mut self, po: &PluginOp) -> Option<&mut Plugin<CTP>> {
        self.iter_mut().find(|p| p.provides(po, Anchor::Define))
    }
}

/// The core structure handling the pluginization of connections.
pub struct PluginHandler<CTP: ConnectionToPlugin> {
    /// The engine used to instantiate plugins.
    engine: Engine,
    /// A pointer to the serving session. It can stay null if no plugin is inserted.
    conn: RawMutPtr<PluginizableConnection<CTP>>,
    /// Function creating an `Imports`.
    exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports,
    /// The actual container of the plugins.
    plugins: PluginArray<CTP>,
    /// Bytes contents that will be passed to potential plugins.
    bytes_contents: Vec<BytesContent>,
    /// Registrations made by the plugins.
    registrations: Vec<Registration>,
    /// A reference time used to make conversions between `Duration` at plugin side
    /// and `Instant` at host side.
    reference_instant: Instant,
    /// A reference UNIX-based time used to make conversions between `Duration` at
    /// plugin side and `Instant` at host side.
    reference_unix_instant: UnixInstant,
    /// Whether the anchor is provided by any of the plugins.
    has_anchor: [bool; 3],
    /// Force this structure to be pinned.
    _pin: PhantomPinned,
}

impl<CTP: ConnectionToPlugin> std::fmt::Debug for PluginHandler<CTP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHandler")
            .field("_pin", &self._pin)
            .finish()
    }
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

impl<CTP: ConnectionToPlugin> PluginHandler<CTP> {
    /// Create a new [`PluginHandler`], enabling the execution of plugins inserted on the fly to
    /// customize the behavior of a connection.
    pub fn new(exports_func: fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports) -> Self {
        Self {
            engine: create_engine(),
            conn: RawMutPtr::null(),
            exports_func,
            plugins: PluginArray { array: Vec::new() },
            bytes_contents: Vec::new(),
            registrations: Vec::new(),
            reference_instant: Instant::now(),
            reference_unix_instant: UnixInstant::now(),
            has_anchor: [false; 3],
            _pin: PhantomPinned,
        }
    }

    /// Set the pointer to the connection context. **This pointer must be pinned**.
    pub fn set_pluginizable_connection(
        &mut self,
        conn: *const PluginizableConnection<CTP>,
    ) -> bool {
        if self.conn.is_null() {
            self.conn = RawMutPtr::new(conn as *mut _);
        } else if !self.conn.ptr_eq(conn as *mut _) {
            error!("Trying to attach a same PH to different connections");
            return false;
        }
        true
    }

    pub(crate) fn insert_plugin_internal(
        &mut self,
        plugin_fname: &PathBuf,
        force_enable: bool,
    ) -> Result<(), Error> {
        let mut plugin = Plugin::new(plugin_fname, self)?;
        // Cache whether anchors are provided.
        self.has_anchor
            .iter_mut()
            .zip(plugin.has_anchor())
            .for_each(|(i, e)| *i |= e);
        if force_enable {
            plugin.force_enable();
        }
        self.plugins.push(plugin);
        // Now the plugin is at its definitive area in memory, so we can initialize it.
        self.plugins
            .last_mut()
            .ok_or(Error::PluginLoadingError("PluginNotInserted".to_string()))?
            .initialize()
            .map_err(|e| Error::PluginLoadingError(format!("{:?}", e)))
    }

    /// Attach a new plugin whose bytecode is accessible through the provided path. Return whether
    /// the insertion succeeded, or the related [`Error`] otherwise.
    ///
    /// If the insertion succeeds and the plugin provides an `init` function as a protocol
    /// operation, this function calls it. This can be useful to, e.g., initialize a plugin-specific
    /// structure or register new frames.
    pub fn insert_plugin(&mut self, plugin_fname: &PathBuf) -> Result<(), Error> {
        self.insert_plugin_internal(plugin_fname, false)
    }

    /// To be used in testing code only.
    #[doc(hidden)]
    pub fn insert_plugin_testing(&mut self, plugin_fname: &PathBuf) -> Result<(), Error> {
        self.insert_plugin_internal(plugin_fname, true)
    }

    /// Return whether there is a bytecode providing the plugin operation
    /// at the requested anchor.
    pub fn provides(&self, po: &PluginOp, anchor: Anchor) -> bool {
        self.has_anchor[anchor.index()] && self.plugins.provides(po, anchor)
    }

    /// Return the first timeout event required by a plugin.
    pub fn timeout(&self) -> Option<Instant> {
        self.plugins.iter().filter_map(|p| p.timeout()).min()
    }

    /// Call potential timeouts that fired since the provided time.
    ///
    /// If there were not firing timers, this method does nothing.
    pub fn on_timeout(&mut self, t: Instant) -> Result<(), Error> {
        for p in self.plugins.iter_mut() {
            p.on_timeout(t)?;
        }
        Ok(())
    }

    /// Get an immutable reference to the serving connection.
    pub fn get_conn(&self) -> Option<&PluginizableConnection<CTP>> {
        if self.conn.is_null() {
            None
        } else {
            // SAFETY: The pluginizable conn is pinned and implements `!Unpin`.
            Some(unsafe { &**self.conn })
        }
    }

    /// Get an mutable reference to the serving connection.
    pub fn get_conn_mut(&mut self) -> Option<&mut PluginizableConnection<CTP>> {
        if self.conn.is_null() {
            None
        } else {
            // SAFETY: The pluginizable conn is pinned and implements `!Unpin`.
            Some(unsafe { &mut **self.conn })
        }
    }

    /// Set bytes content, to be available through a [`Bytes`] value by the plugin.
    pub fn add_bytes_content(&mut self, bc: BytesContent) -> Bytes {
        let tag = self.bytes_contents.len() as u64;
        let max_read_len = bc.read_len() as u64;
        let max_write_len = bc.write_len() as u64;
        self.bytes_contents.push(bc);
        Bytes {
            tag,
            max_read_len,
            max_write_len,
        }
    }

    /// Clear the content made available to the plugin.
    ///
    /// This method must always be called once the plugin operation call completes.
    /// This is automatically done by the helping macros.
    pub fn clear_bytes_content(&mut self) {
        self.bytes_contents.clear();
    }

    /// Get a mutable reference on the [`BytesContent`] with the associtated `tag`.
    pub(crate) fn get_mut_bytes_content(
        &mut self,
        tag: usize,
    ) -> Result<&mut BytesContent, CTPError> {
        self.bytes_contents.get_mut(tag).ok_or(CTPError::BadBytes)
    }

    /// Register some plugin [`Registration`].
    pub fn add_registration(&mut self, r: Registration) {
        self.registrations.push(r);
    }

    /// Return all the [`Registration`]s that are known by the pluginization handler.
    pub fn get_registrations(&self) -> &[Registration] {
        &self.registrations
    }

    /// Clone the plugin environment engine.
    pub(crate) fn get_cloned_engine(&self) -> Engine {
        self.engine.clone()
    }

    /// Return the set of exported functions.
    pub(crate) fn get_export_func(&self) -> fn(&mut Store, &FunctionEnv<Env<CTP>>) -> Exports {
        self.exports_func
    }

    /// Gets a UNIX-based `Instant` usable by the plugin side from a host-side `Instant`.
    pub(crate) fn get_unix_instant_from_instant(&self, i: Instant) -> UnixInstant {
        let d = i.duration_since(self.reference_instant);
        self.reference_unix_instant + d
    }

    /// Gets a `Instant` usable by the host side from a plugin-side UNIX-based `Instant`.
    pub(crate) fn get_instant_from_unix_instant(&self, i: UnixInstant) -> Instant {
        let d = i.duration_since(self.reference_unix_instant);
        self.reference_instant + d
    }

    /// Invokes the protocol operation `po` and runs its anchors.
    fn call_internal(
        &mut self,
        pod: Option<&&ProtocolOperationDefault>,
        po: &PluginOp,
        params: &[PluginVal],
    ) -> Result<Vec<PluginVal>, Error> {
        // PRE part
        for p in self
            .plugins
            .iter_mut()
            .filter(|p| p.provides(po, Anchor::Before))
        {
            p.call(po, Anchor::Before, params)?;
        }

        // REPLACE part
        let res = match self.plugins.get_first_plugin(po) {
            Some(p) => p.call(po, Anchor::Define, params)?,
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
        for p in self
            .plugins
            .iter_mut()
            .filter(|p| p.provides(po, Anchor::After))
        {
            p.call(po, Anchor::After, params)?;
        }

        Ok(res)
    }

    /// Only for BEFORE or AFTER calls.
    pub fn call_direct(
        &mut self,
        po: &PluginOp,
        anchor: Anchor,
        params: &[PluginVal],
    ) -> Result<(), Error> {
        assert_ne!(anchor, Anchor::Define);
        if anchor == Anchor::Define {
            return Err(Error::InternalError(
                "call_direct only available for Before or After anchors.".to_string(),
            ));
        }
        for p in self.plugins.iter_mut().filter(|p| p.provides(po, anchor)) {
            p.call(po, anchor, params)?;
        }

        Ok(())
    }

    /// Invokes the plugin operation `po` and runs its anchors.
    pub fn call(&mut self, po: &PluginOp, params: &[PluginVal]) -> Result<Vec<PluginVal>, Error> {
        // trace!("Calling protocol operation {:?}", po);

        // TODO
        // let pod = self.default_protocol_operations.get(po);

        self.call_internal(None /* pod */, po, params)
    }

    /// Invokes a plugin operation control operation.
    pub fn poctl(&mut self, id: u64, params: &[PluginVal]) -> Result<Vec<PluginVal>, Error> {
        self.call(&PluginOp::PluginControl(id), params)
    }
}
