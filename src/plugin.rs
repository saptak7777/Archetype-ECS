use crate::app::App;

/// Plugin trait for modular application architecture
pub trait Plugin {
    /// Build the plugin into the app
    fn build(&self, app: &mut App);
}
