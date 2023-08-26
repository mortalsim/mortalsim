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

// #[cfg(test)]
// pub mod test {
//     use simple_si_units::chemical::Concentration;
//     use simple_si_units::chemical::Molality;

//     use super::BloodVessel;
//     use super::ClosedCircComponent;
//     use super::{ClosedCircConnector, ClosedCircInitializer};
//     use crate::sim::layer::closed_circulation::VesselIter;
//     use crate::sim::component::SimComponent;
//     use crate::event::test::{TestEventA, TestEventB};
//     use crate::event::Event;
//     use crate::substance::Substance;
//     use std::collections::HashSet;
//     use std::sync::Arc;

//     lazy_static! {
//         static ref START_VESSELS: HashSet<TestBloodVessel> = {
//             let mut vessel_list = HashSet::new();
//             vessel_list.insert(TestBloodVessel::Aorta);
//             vessel_list
//         };
//     }

//     #[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
//     pub enum TestBloodVessel {
//         Aorta,
//         ThoracicAorta,
//         LeftSubclavianArtery,
//         RightBraciocephalicArtery,
//         LeftCommonCarotidArtery,
//         SuperiorVenaCava,
//         InferiorVenaCava,
//         VenaCava,
//     }

//     impl BloodVessel for TestBloodVessel {
//         fn start_vessels<'a>() -> VesselIter<'a, Self> {
//             VesselIter(START_VESSELS.iter())
//         }
//     }

//     pub struct TestCircComponentA {
//         cc_sim_connector: ClosedCircConnector<TestBloodVessel>
//     }

//     impl TestCircComponentA {
//         pub fn new() -> TestCircComponentA {
//             TestCircComponentA {
//                 cc_sim_connector: ClosedCircConnector::new()
//             }
//         }
//     }

//     impl ClosedCircComponent for TestCircComponentA {
//         type VesselType = TestBloodVessel;

//         fn init_cc(&mut self, cc_initializer: &mut ClosedCircInitializer<TestBloodVessel>) {
//             let threshold = Concentration::from_mM(0.1);
//             cc_initializer.notify_composition_change(
//                 TestBloodVessel::ThoracicAorta,
//                 Substance::GLC,
//                 threshold,
//             );
//             cc_initializer.attach_vessel(TestBloodVessel::InferiorVenaCava);
//         }

//         fn cc_sim_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType> {
//             &mut self.cc_sim_connector
//         }
//     }

//     impl SimComponent for TestCircComponentA {

//         /// The unique id of the component
//         fn id(&self) -> &'static str {
//             "TestCircComponentA"
//         }

//         /// Attaches the module to the ComponentKeeper
//         fn attach(self, registry: &mut ComponentRegistry) {
//             registry.add_
//         }

//         /// Runs an iteration of this module.
//         fn run(&mut self) {
//             // let change = BloodCompositionChange::<TestBloodVessel> {
//             //     vessel: TestBloodVessel::InferiorVenaCava,
//             //     substance: Substance::GLC,
//             //     change: MolarConcentration::new::<millimole_per_liter>(0.1 / self.cc_sim_connector().depth() as f64),
//             // };

//             // self.cc_sim_connector.schedule_event(Time::from_s(1.0), change);
//         }
//     }
// }
