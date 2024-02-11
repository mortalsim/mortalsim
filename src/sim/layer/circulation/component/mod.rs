mod connector;
mod initializer;
pub use connector::BloodStore;
pub use connector::CirculationConnector;
pub use initializer::CirculationInitializer;

use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;

pub trait CirculationComponent<O: Organism>: SimComponent<O> {
    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn circulation_init(&mut self, circulation_initializer: &mut CirculationInitializer<O>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// CirculationConnector, which tracks module interactions
    ///
    /// ### returns
    /// TimeManager to interact with the rest of the simulation
    fn circulation_connector(&mut self) -> &mut CirculationConnector<O>;
}

#[cfg(test)]
pub mod test {
    use super::CirculationComponent;
    use super::{CirculationConnector, CirculationInitializer};
    use crate::sim::component::registry::ComponentRegistry;
    use crate::sim::component::SimComponent;
    use crate::sim::layer::circulation::component::connector::BloodStore;
    use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
    use crate::sim::organism::test::TestSim;
    use crate::sim::test::TestOrganism;
    use crate::sim::SimTime;
    use crate::substance::Substance;
    use crate::util::mmol_per_L;
    use simple_si_units::chemical::Concentration;

    pub struct TestCircComponentA {
        cc_sim_connector: CirculationConnector<TestOrganism>,
    }

    impl TestCircComponentA {
        pub fn new() -> TestCircComponentA {
            TestCircComponentA {
                cc_sim_connector: CirculationConnector::new(),
            }
        }
    }

    impl CirculationComponent<TestOrganism> for TestCircComponentA {
        fn circulation_init(
            &mut self,
            circulation_initializer: &mut CirculationInitializer<TestOrganism>,
        ) {
            circulation_initializer.notify_composition_change(
                TestBloodVessel::Aorta,
                Substance::GLC,
                Concentration::from_mM(0.1),
            );
            circulation_initializer.attach_vessel(TestBloodVessel::VenaCava);
        }

        fn circulation_connector(&mut self) -> &mut CirculationConnector<TestOrganism> {
            &mut self.cc_sim_connector
        }
    }

    impl SimComponent<TestOrganism> for TestCircComponentA {
        /// The unique id of the component
        fn id(&self) -> &'static str {
            "TestCircComponentA"
        }

        /// Attaches the module to the ComponentKeeper
        fn attach(self, registry: &mut ComponentRegistry<TestOrganism>) {
            registry.add_circulation_component(self)
        }

        /// Runs an iteration of this module.
        fn run(&mut self) {
            self.cc_sim_connector
                .blood_store(&TestBloodVessel::VenaCava)
                .unwrap()
                .schedule_change(Substance::GLC, mmol_per_L!(1.0), SimTime::from_s(1.0));
        }
    }

    #[test]
    fn test_component() {
        let mut component = TestCircComponentA::new();

        let mut circulation_initializer = CirculationInitializer::new();

        component.circulation_init(&mut circulation_initializer);

        assert_eq!(
            circulation_initializer
                .substance_notifies
                .get(&TestBloodVessel::Aorta)
                .unwrap()
                .get(&Substance::GLC)
                .unwrap()
                .threshold,
            mmol_per_L!(0.1)
        );

        component
            .circulation_connector()
            .vessel_map
            .insert(TestBloodVessel::VenaCava, BloodStore::new());

        component.run();

        component
            .circulation_connector()
            .vessel_map
            .get_mut(&TestBloodVessel::VenaCava)
            .unwrap()
            .advance(SimTime::from_s(2.0));

        let glc = component
            .circulation_connector()
            .blood_store(&TestBloodVessel::VenaCava)
            .unwrap()
            .concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(
            glc > expected - threshold && glc < expected + threshold,
            "GLC not within {} of {}",
            threshold,
            expected
        );

        component
            .circulation_connector()
            .vessel_map
            .get_mut(&TestBloodVessel::VenaCava)
            .unwrap()
            .advance(SimTime::from_s(2.0));

        let glc = component
            .circulation_connector()
            .blood_store(&TestBloodVessel::VenaCava)
            .unwrap()
            .concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(
            glc > expected - threshold && glc < expected + threshold,
            "GLC not within {} of {}",
            threshold,
            expected
        );
    }
}
