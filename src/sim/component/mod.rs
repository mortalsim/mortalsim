

pub mod registry;
pub mod wrapper;

use self::registry::ComponentRegistry;

/// Trait to be used by any modules for Sim objects
pub trait SimComponent {
    /// Attaches the module to the ComponentKeeper
    fn attach(self, keeper: &mut ComponentRegistry);
    /// Runs an iteration of this module.
    fn run(&mut self);
}
