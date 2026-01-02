use crate::app::App;

/// Plugin trait for modular application architecture
pub trait Plugin {
    /// Get the name of this plugin (for logging and debugging)
    fn plugin_name(&self) -> &'static str;
    
    /// Build the plugin into the app
    fn build(&self, app: &mut App);
}
