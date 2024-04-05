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
    use rand::{distributions::Alphanumeric, Rng};
    use simple_si_units::geometry::Volume;

    use crate::{sim::{component::{ComponentRegistry, SimComponent}, layer::digestion::{consumable::test::{test_ammonia, test_fiber, test_food}, consumed::Consumed, DigestionDirection, DigestionInitializer}, organism::test::TestOrganism, Consumable, Organism, SimTime}, substance::Substance, util::{mmol_per_L, secs}};

    use super::{DigestionComponent, DigestionConnector};

    pub struct TestDigestionComponent<O: Organism> {
        connector: DigestionConnector<O>,
        id: &'static str,
    }
    impl<O: Organism> TestDigestionComponent<O> {
        pub fn new() -> Self {
            // Generate a unique id each time so we can add multiple
            let s: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect();
            let cid = format!("{}{}", "TestDigestionComponent", s).leak();

            Self {
                connector: DigestionConnector::new(),
                id: cid,
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
            self.id
        }
        fn attach(self, registry: &mut ComponentRegistry<O>) {
            registry.add_digestion_component(self)
        }
        fn run(&mut self) {
            for cons in self.connector.consumed() {
                if cons.concentration_of(&Substance::NH3) > mmol_per_L!(1.0) {
                    cons.set_exit(cons.entry_time + secs!(5.0), DigestionDirection::BACK).unwrap();
                }
                else if cons.concentration_of(&Substance::GLC) > mmol_per_L!(0.0) {
                    // Mmmm sugar!
                    cons.schedule_change(Substance::GLC, -cons.concentration_of(&Substance::GLC), SimTime::from_min(5.0));
                    cons.set_exit(cons.entry_time + SimTime::from_min(5.0), DigestionDirection::EXHAUSTED).unwrap();
                }
                else {
                    cons.set_exit(cons.entry_time + SimTime::from_min(1.0), DigestionDirection::FORWARD).unwrap();
                }
            }
        }
    }


    #[test]
    fn test_component() {
        let mut component: TestDigestionComponent<TestOrganism> = TestDigestionComponent::new();
        component.digestion_connector().consumed_list.push(Consumed::new(test_food(250.0)));
        component.digestion_connector().consumed_list.push(Consumed::new(test_ammonia(50.0)));
        component.digestion_connector().consumed_list.push(Consumed::new(test_fiber(100.0)));

        component.run();

        let fiber = component.digestion_connector().consumed_list.pop().unwrap();
        let ammonia = component.digestion_connector().consumed_list.pop().unwrap();
        let mut food = component.digestion_connector().consumed_list.pop().unwrap();

        assert_eq!(food.exit_direction, DigestionDirection::EXHAUSTED);
        assert_eq!(ammonia.exit_direction, DigestionDirection::BACK);
        assert_eq!(fiber.exit_direction, DigestionDirection::FORWARD);

        let orig_conc = food.concentration_of(&Substance::GLC);
        food.advance(SimTime::from_min(1.0));

        assert!(food.concentration_of(&Substance::GLC) < orig_conc);

        food.advance(SimTime::from_min(10.0));
        assert!(food.concentration_of(&Substance::GLC) < mmol_per_L!(0.1));
    }
}
