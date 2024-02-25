use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;

pub(crate) mod connector;
pub(crate) mod initializer;

pub use connector::DigestionConnector;
pub use initializer::DigestionInitializer;

pub trait DigestionComponent<O: Organism>: SimComponent<O> {
    /// Initializes the module. Currently not used.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn digestion_init(&mut self, _initializer: &mut DigestionInitializer<O>) {}

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// DigestionConnector, which tracks module interactions
    ///
    /// ### returns
    /// DigestionConnector to interact with the digestion layer
    fn digestion_connector(&mut self) -> &mut DigestionConnector<O>;
}

#[cfg(test)]
pub mod test {
    use crate::{sim::{component::{ComponentRegistry, SimComponent}, layer::digestion::DigestionDirection, Organism}, substance::Substance, util::{mmol_per_L, secs}};

    use super::{DigestionComponent, DigestionConnector};

    pub struct TestDigestionComponent<O: Organism> {
        connector: DigestionConnector<O>,
    }
    impl<O: Organism> TestDigestionComponent<O> {
        fn new() -> Self {
            Self {
                connector: DigestionConnector::new(),
            }
        }
    }
    impl<O: Organism> DigestionComponent<O> for TestDigestionComponent<O> {
        fn digestion_connector(&mut self) -> &mut DigestionConnector<O> {
            &mut self.connector
        }
    }

    impl<O: Organism> SimComponent<O> for TestDigestionComponent<O> {
        fn id(&self) -> &'static str {
            "TestDigestionComponent"
        }
        fn attach(self, registry: &mut ComponentRegistry<O>) {
            registry.add_digestion_component(self)
        }
        fn run(&mut self) {
            for food in self.connector.consumed() {
                if food.concentration_of(&Substance::NH3) > mmol_per_L!(1.0) {
                    food.set_exit(secs!(5.0), DigestionDirection::BACK);
                }
            }
        }
    }


    #[test]
    fn test_component() {
        // let mut component = TestComponentA::new();
        // let mut initializer = CoreInitializer::new();
        // component.core_init(&mut initializer);

        // assert!(initializer.pending_notifies.len() == 2);
        // assert!(initializer.pending_transforms.len() == 1);
    }
}
