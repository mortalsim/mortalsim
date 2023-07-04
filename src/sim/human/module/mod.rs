mod connector;
mod initializer;
use crate::core::sim::{CoreSimModule, SimConnector, SimModule, SimModuleInitializer};
pub use connector::HumanSimConnector;
pub use initializer::HumanModuleInitializer;

pub trait HumanSimModule: CoreSimModule {
    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn init_human(&mut self, initializer: &mut HumanModuleInitializer);

    /// Retrieves the SimModule portion of this module
    ///
    /// ### returns
    /// this object as a SimModule
    fn as_core_module(&mut self) -> &mut dyn CoreSimModule;

    /// Used by the HumanSim to retrieve a mutable reference to this module's
    /// HumanSimConnector, which tracks module interactions
    ///
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn get_human_sim_connector(&mut self) -> &mut HumanSimConnector;
}

#[cfg(test)]
mod tests {
    use super::super::sim::register_module;
    use super::{HumanSimConnector, HumanSimModule, SimConnector, SimModule, SimModuleInitializer};
    use crate::closed_circulation::{ClosedCircConnector, ClosedCircInitializer};
    use crate::event::test::{TestEventA, TestEventB};
    use crate::event::Event;
    use std::sync::Arc;
    use uom::si::amount_of_substance::mole;
    use uom::si::f64::{AmountOfSubstance, Length};
    use uom::si::length::meter;

    // pub struct TestModuleA {
    //     connector: SimConnector,
    //     human_connector: HumanSimConnector,
    // }
    // impl TestModuleA {
    //     pub fn factory() -> Box<dyn HumanSimModule> {
    //         Box::new(TestModuleA {
    //             connector: SimConnector::new(),
    //             human_connector: HumanSimConnector::new(SimConnector::new(), ClosedCircConnector::new(Rc::new(HumanCirculatorySystem), ClosedCircInitializer::new())),
    //         })
    //     }
    // }
    // impl SimModule for TestModuleA {
    //     fn get_sim_connector(&mut self) -> &mut SimConnector {
    //         &mut self.connector
    //     }
    //     fn init(&mut self, initializer: &mut SimModuleInitializer) {
    //         initializer.notify(TestEventA::new(Distance::from_m(1.0)));
    //         initializer.notify(TestEventB::new(Amount::from_mol(1.0)));
    //         initializer.transform(|evt: &mut TestEventA| {
    //             evt.len = Distance::from_m(3.0);
    //         });
    //     }
    //     fn run(&mut self) {
    //         let evt_a = self.connector.get::<TestEventA>().unwrap();
    //         assert_eq!(evt_a.len, Distance::from_m(3.0));

    //         log::debug!("Trigger Events: {:?}", self.connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());
    //     }
    // }

    // impl HumanSimModule for TestModuleA {
    //     fn as_sim_module(&mut self) -> &mut dyn SimModule {
    //         self
    //     }
    //     fn init_human(&mut self, initializer: &mut super::HumanModuleInitializer) {

    //     }

    //     fn get_human_sim_connector(&mut self) -> &mut super::HumanSimConnector {
    //         &mut self.human_connector
    //     }
    // }

    // #[test]
    // fn test_human_sim_module() {
    //     register_module("TestModuleA", TestModuleA::factory);

    //     let module = TestModuleA::factory();
    //     let module_ref: Box<&mut dyn SimModule> = Box::new(module.as_sim_module());

    //     // assert_eq!(sim.get_time(), Time::from_s(0.0));
    //     // sim.advance_by(Time::from_s(1.0));
    //     // assert_eq!(sim.get_time(), Time::from_s(1.0));
    // }
}

// impl SimModule for dyn HumanSimModule {
//     /// Initializes the module. Should register any `Event` objects to listen for
//     /// and set initial state.
//     ///
//     /// ### Arguments
//     /// * `initializer` - Helper object for initializing the module
//     fn init(&mut self, initializer: &mut SimModuleInitializer) {}

//     /// Note that all `Event`s previously scheduled by this module which have not yet
//     /// occurred will be unscheduled before `run` is executed.
//     ///
//     /// ### returns
//     /// SimConnector to interact with the rest of the simulation
//     fn get_sim_connector(&mut self) -> &mut SimConnector {
//         HumanSimModule::get_sim_connector(self)
//     }

//     /// Runs an iteration of this module. Will be called anytime a `notify` registered
//     /// `Event` changes on `Sim` state. All module logic should ideally occur within this
//     /// call and all resulting `Event` objects scheduled for future emission.
//     ///
//     /// Note that all `Event`s previously scheduled by this module which have not yet
//     /// occurred will be unscheduled before `run` is executed.
//     ///
//     /// ### Arguments
//     /// * `connector` - Helper object for the module to interact with the rest of
//     ///                 the simulation
//     fn run(&mut self) {
//         HumanSimModule::run(self)
//     }
// }
