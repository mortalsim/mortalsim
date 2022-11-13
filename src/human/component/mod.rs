mod initializer;
mod connector;
pub use initializer::HumanComponentInitializer;
pub use connector::HumanSimConnector;
use crate::core::sim::{SimComponent, SimComponentInitializer, SimConnector};


pub trait HumanSimComponent: SimComponent {

    /// Initializes the component. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the component
    fn init_human(&mut self, initializer: &mut HumanComponentInitializer);

    /// Retrieves the SimComponent portion of this component
    /// 
    /// ### returns
    /// this object as a SimComponent
    fn as_sim_component(&mut self) -> &mut dyn SimComponent;
    
    /// Used by the HumanSim to retrieve a mutable reference to this component's
    /// HumanSimConnector, which tracks component interactions
    /// 
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn get_human_sim_connector(&mut self) -> &mut HumanSimConnector;

}

#[cfg(test)]
mod tests {
    use super::{HumanSimComponent, SimConnector, SimComponent, SimComponentInitializer, HumanSimConnector};
    use super::super::sim::register_component;
    use crate::closed_circulation::{ClosedCircConnector, ClosedCircInitializer};
    use crate::event::Event;
    use crate::event::test::{TestEventA, TestEventB};
    use std::sync::Arc;
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    pub struct TestComponentA {
        connector: SimConnector,
        human_connector: HumanSimConnector,
    }
    impl TestComponentA {
        pub fn factory() -> Box<dyn HumanSimComponent> {
            Box::new(TestComponentA {
                connector: SimConnector::new(),
                human_connector: HumanSimConnector::new(SimConnector::new(), ClosedCircConnector::new(ClosedCircInitializer::new())),
            })
        }
    }
    impl SimComponent for TestComponentA {
        fn get_sim_connector(&mut self) -> &mut SimConnector {
            &mut self.connector
        }
        fn init(&mut self, initializer: &mut SimComponentInitializer) {
            initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Length::new::<meter>(3.0);
            });
        }
        fn run(&mut self) {
            let evt_a = self.connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
        }
    }

    impl HumanSimComponent for TestComponentA {
        fn as_sim_component(&mut self) -> &mut dyn SimComponent {
            self
        }
        fn init_human(&mut self, initializer: &mut super::HumanComponentInitializer) {

        }

        fn get_human_sim_connector(&mut self) -> &mut super::HumanSimConnector {
            &mut self.human_connector 
        }
    }

    #[test]
    fn test_human_sim_component() {
        register_component("TestComponentA", TestComponentA::factory);

        let component = TestComponentA::factory();
        let component_ref: Box<&mut dyn SimComponent> = Box::new(component.as_sim_component());

        // assert_eq!(sim.get_time(), Time::new::<second>(0.0));
        // sim.advance_by(Time::new::<second>(1.0));
        // assert_eq!(sim.get_time(), Time::new::<second>(1.0));
    }
}

// impl SimComponent for dyn HumanSimComponent {
//     /// Initializes the component. Should register any `Event` objects to listen for
//     /// and set initial state.
//     /// 
//     /// ### Arguments
//     /// * `initializer` - Helper object for initializing the component
//     fn init(&mut self, initializer: &mut SimComponentInitializer) {}

//     /// Note that all `Event`s previously scheduled by this component which have not yet
//     /// occurred will be unscheduled before `run` is executed.
//     /// 
//     /// ### returns
//     /// SimConnector to interact with the rest of the simulation
//     fn get_sim_connector(&mut self) -> &mut SimConnector {
//         HumanSimComponent::get_sim_connector(self)
//     }
    
//     /// Runs an iteration of this component. Will be called anytime a `notify` registered
//     /// `Event` changes on `Sim` state. All module logic should ideally occur within this
//     /// call and all resulting `Event` objects scheduled for future emission.
//     /// 
//     /// Note that all `Event`s previously scheduled by this component which have not yet
//     /// occurred will be unscheduled before `run` is executed.
//     /// 
//     /// ### Arguments
//     /// * `connector` - Helper object for the component to interact with the rest of
//     ///                 the simulation
//     fn run(&mut self) {
//         HumanSimComponent::run(self)
//     }
// }
