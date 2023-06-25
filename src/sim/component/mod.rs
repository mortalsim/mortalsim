

pub mod registry;
pub mod wrapper;

use self::registry::ComponentRegistry;

/// Trait to be used by any modules for Sim objects
pub trait SimComponent {
  /// Attaches the module to the ComponentKeeper
  fn attach(self, registry: &mut ComponentRegistry);
  /// Runs an iteration of this module.
  fn run(&mut self);
}

/// Trait to outline common methods for all systems that
/// process `SimComponent`s
pub trait SimComponentProcessor<T: SimComponent> {
  /// Prepare a component for their run
  fn prepare_component(&self, component: &mut T);
  /// Process a component after their run
  fn process_component(&mut self, component: &mut T);
}
