mod connector;
mod initializer;
pub use connector::ClosedCircConnector;
pub use connector::BloodStore;
pub use initializer::ClosedCircInitializer;

use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;

pub trait ClosedCircComponent<O: Organism>: SimComponent<O> {

    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn cc_init(&mut self, cc_initializer: &mut ClosedCircInitializer<O>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// ClosedCircConnector, which tracks module interactions
    ///
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn cc_connector(&mut self) -> &mut ClosedCircConnector<O>;
}

#[cfg(test)]
pub mod test {
    use simple_si_units::chemical::Concentration;
    use crate::sim::layer::closed_circulation::component::connector::BloodStore;
    use crate::sim::layer::closed_circulation::vessel::test::TestBloodVessel;
    use crate::sim::organism::test::TestSim;
    use super::ClosedCircComponent;
    use super::{ClosedCircConnector, ClosedCircInitializer};
    use crate::sim::SimTime;
    use crate::sim::component::registry::ComponentRegistry;
    use crate::sim::component::SimComponent;
    use crate::substance::Substance;
    use crate::util::mmol_per_L;
    
    pub struct TestCircComponentA {
        cc_sim_connector: ClosedCircConnector<TestSim>
    }

    impl TestCircComponentA {
        pub fn new() -> TestCircComponentA {
            TestCircComponentA {
                cc_sim_connector: ClosedCircConnector::new()
            }
        }
    }

    impl ClosedCircComponent<TestSim> for TestCircComponentA {

        fn cc_init(&mut self, cc_initializer: &mut ClosedCircInitializer<TestSim>) {
            cc_initializer.notify_composition_change(
                TestBloodVessel::Aorta,
                Substance::GLC,
                Concentration::from_mM(0.1),
            );
            cc_initializer.attach_vessel(TestBloodVessel::VenaCava);
        }

        fn cc_connector(&mut self) -> &mut ClosedCircConnector<TestSim> {
            &mut self.cc_sim_connector
        }
    }

    impl SimComponent<TestSim> for TestCircComponentA {

        /// The unique id of the component
        fn id(&self) -> &'static str {
            "TestCircComponentA"
        }

        /// Attaches the module to the ComponentKeeper
        fn attach(self, registry: &mut ComponentRegistry<TestSim>) {
            registry.add_component(self)
        }

        /// Runs an iteration of this module.
        fn run(&mut self) {
            self.cc_sim_connector.blood_store(&TestBloodVessel::VenaCava).unwrap().schedule_change(
                Substance::GLC,
                mmol_per_L!(1.0),
                SimTime::from_s(1.0),
            );
        }
    }

    #[test]
    fn test_component() {
        let mut component = TestCircComponentA::new();

        let mut cc_initializer = ClosedCircInitializer::new();

        component.cc_init(&mut cc_initializer);

        assert_eq!(cc_initializer.substance_notifies.get(&TestBloodVessel::Aorta)
            .unwrap()
            .get(&Substance::GLC).unwrap().threshold, mmol_per_L!(0.1));
        
        component.cc_connector().vessel_map.insert(TestBloodVessel::VenaCava, BloodStore::new());

        component.run();

        component.cc_connector().vessel_map.get_mut(&TestBloodVessel::VenaCava).unwrap().advance(SimTime::from_s(2.0));

        let glc = component.cc_connector().blood_store(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
        
        component.cc_connector().vessel_map.get_mut(&TestBloodVessel::VenaCava).unwrap().advance(SimTime::from_s(2.0));

        let glc = component.cc_connector().blood_store(&TestBloodVessel::VenaCava).unwrap().concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);

    }
}
