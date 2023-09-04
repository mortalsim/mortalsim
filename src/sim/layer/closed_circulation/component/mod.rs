mod connector;
mod initializer;
pub use connector::ClosedCircConnector;
pub use initializer::ClosedCircInitializer;

use crate::sim::component::SimComponent;

use super::vessel::BloodVessel;

pub trait ClosedCircComponent: SimComponent {
    type VesselType: BloodVessel;

    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn cc_init(&mut self, cc_initializer: &mut ClosedCircInitializer<Self::VesselType>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// ClosedCircConnector, which tracks module interactions
    ///
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn cc_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType>;
}

#[cfg(test)]
pub mod test {
    use simple_si_units::chemical::Concentration;
    use simple_si_units::chemical::Molality;
    use simple_si_units::base::Time;

    use super::BloodVessel;
    use super::ClosedCircComponent;
    use super::{ClosedCircConnector, ClosedCircInitializer};
    use crate::sim::SimTime;
    use crate::sim::component::registry::ComponentRegistry;
    use crate::sim::layer::closed_circulation::VesselIter;
    use crate::sim::component::SimComponent;
    use crate::event::test::{TestEventA, TestEventB};
    use crate::event::Event;
    use crate::sim::layer::closed_circulation::vessel::test::TestBloodVessel;
    use crate::substance::Substance;
    use crate::substance::SubstanceStore;
    use crate::util::mmol_per_L;
    use crate::util::BoundFn;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;
    
     pub struct TestCircComponentA {
         cc_sim_connector: ClosedCircConnector<TestBloodVessel>
     }

     impl TestCircComponentA {
         pub fn new() -> TestCircComponentA {
             TestCircComponentA {
                 cc_sim_connector: ClosedCircConnector::new()
             }
         }
     }

    impl ClosedCircComponent for TestCircComponentA {
        type VesselType = TestBloodVessel;

        fn cc_init(&mut self, cc_initializer: &mut ClosedCircInitializer<TestBloodVessel>) {
            cc_initializer.notify_composition_change(
                TestBloodVessel::Aorta,
                Substance::GLC,
                Concentration::from_mM(0.1),
            );
            cc_initializer.attach_vessel(TestBloodVessel::VenaCava);
        }

        fn cc_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType> {
            &mut self.cc_sim_connector
        }
    }

    impl SimComponent for TestCircComponentA {

        /// The unique id of the component
        fn id(&self) -> &'static str {
            "TestCircComponentA"
        }

        /// Attaches the module to the ComponentKeeper
        fn attach(self, registry: &mut ComponentRegistry) {
            registry.add_closed_circulation_component(self)
        }

        /// Runs an iteration of this module.
        fn run(&mut self) {
            assert!(self.cc_sim_connector.schedule_change(
                    TestBloodVessel::VenaCava,
                    Substance::GLC,
                    mmol_per_L!(1.0),
                    SimTime::from_s(1.0),
                ).is_ok());
        }
    }

    #[test]
    fn test_component() {
        let mut component = TestCircComponentA::new();
        let mut vessel_map = HashSet::new();
        vessel_map.insert(TestBloodVessel::VenaCava);
        component.cc_connector().vessel_connections = vessel_map;

        let mut cc_initializer = ClosedCircInitializer::new();

        component.cc_init(&mut cc_initializer);

        component.cc_connector().vessel_connections = cc_initializer.vessel_connections;
        component.cc_connector().substance_notifies = cc_initializer.substance_notifies;

        assert_eq!(component.cc_connector()
            .substance_notifies.get(&TestBloodVessel::Aorta)
            .unwrap()
            .get(&Substance::GLC).unwrap(), &mmol_per_L!(0.1));

        component.run();

        let mut store = SubstanceStore::new();

        for (_vessel, mut changes)  in component.cc_connector().pending_changes.drain() {
            for (_id, (substance, change)) in changes.drain() {
                store.schedule_substance_change(substance, change);
            }
        }

        store.advance(SimTime::from_s(2.0));

        let glc = store.concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);
        
        store.advance(SimTime::from_s(2.0));

        let glc = store.concentration_of(&Substance::GLC);
        let expected = mmol_per_L!(1.0);
        let threshold = mmol_per_L!(0.0001);
        assert!(glc > expected - threshold && glc < expected + threshold, "GLC not within {} of {}", threshold, expected);

    }
}