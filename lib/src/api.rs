use std::sync::{Arc, Weak, RwLock};

use pluginop_common::{quic::{ConnectionField, RecoveryField}};

use crate::{PluginizableConnection};

/// A trait that needs to be implemented by the host implementation to provide
/// plugins information from the host.
pub trait ConnectionToPlugin<'a, P: PluginizableConnection>: Send {
    /// Gets the related `ConnectionField` and writes it as a serialized value in `w`.
    /// It is up to the plugin to correctly handle the value and perform the serialization.
    fn get_connection(&self, w: &mut [u8], field: ConnectionField) -> bincode::Result<()>;
    /// Sets the related `ConnectionField` to the provided value, that was serialized with content
    /// `value`. It is this function responsibility to correctly convert the
    /// input to the right type.
    fn set_connection(&mut self, field: ConnectionField, value: &[u8]);
    /// Gets the related `RecoveryField` and writes it as a serialized value in `w`. It is up to the
    /// plugin to correctly handle the value and perform the serialization.
    fn get_recovery(&self, w: &mut [u8], field: RecoveryField) -> bincode::Result<()>;
    /// Sets the related `RecoveryField` to the provided value, that was serialized with content
    /// `value`. It is this function responsibility to correctly convert the
    /// input to the right type.
    fn set_recovery(&mut self, field: RecoveryField, value: &[u8]);
    /// Sets the pluginizable connection.
    fn set_pluginizable_conn(&mut self, pc: &Arc<RwLock<P>>);
    /// Gets the pluginizable connection.
    fn get_pluginizable_conn(&self) -> Option<&Weak<RwLock<P>>>;
}