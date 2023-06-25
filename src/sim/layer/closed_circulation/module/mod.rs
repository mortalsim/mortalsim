mod initializer;
mod connector;
pub use initializer::{ClosedCircInitializer};
pub use connector::{ClosedCircConnector};
use crate::core::sim::SimModule;

use super::vessel::BloodVessel;

pub trait ClosedCircSimModule: SimModule {
    type VesselType: BloodVessel;

    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn init_cc(&mut self, cc_initializer: &mut ClosedCircInitializer<Self::VesselType>);
    
    // /// Retrieves the SimModule portion of this module
    // /// 
    // /// ### returns
    // /// this object as a SimModule
    // fn as_sim_module(&mut self) -> &mut dyn SimModule;
    
    // /// Used by the HumanSim to retrieve a mutable reference to this module's
    // /// ClosedCircConnector, which tracks module interactions
    // /// 
    // /// ### returns
    // /// SimConnector to interact with the rest of the simulation
    // fn cc_sim_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType>;
}

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;
    use std::sync::Arc;
    use crate::closed_circulation::VesselIter;
    use crate::core::sim::SimModule;
    use crate::event::{Event, BloodCompositionChange};
    use crate::event::test::{TestEventA, TestEventB};
    use crate::substance::{Substance, MolarConcentration};
    use crate::substance::Time;
    use super::BloodVessel;
    use super::ClosedCircSimModule;
    use super::{ClosedCircInitializer, ClosedCircConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;
    use uom::si::molar_concentration::millimole_per_liter;
    use uom::si::time::second;

    lazy_static! {
        static ref START_VESSELS: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::Aorta);
            vessel_list
        };
    }

    #[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
    pub enum TestBloodVessel {
        Aorta,
        ThoracicAorta,
        LeftSubclavianArtery,
        RightBraciocephalicArtery,
        LeftCommonCarotidArtery,
        SuperiorVenaCava,
        InferiorVenaCava,
        VenaCava,
    }

    impl BloodVessel for TestBloodVessel {
        fn start_vessels<'a>() -> VesselIter<'a, Self> {
            VesselIter { iter: START_VESSELS.iter() }
        }
    }

    pub struct TestCircModuleA {
    }

    impl TestCircModuleA {
        pub fn new() -> TestCircModuleA {
            TestCircModuleA {
            }
        }
        pub fn factory() -> Box<dyn ClosedCircSimModule<VesselType = TestBloodVessel>> {
            Box::new(TestCircModuleA::new())
        }
    }

    impl ClosedCircSimModule for TestCircModuleA {
        type VesselType = TestBloodVessel;

        fn init_cc(&mut self, cc_initializer: &mut ClosedCircInitializer<TestBloodVessel>) {

            let threshold = MolarConcentration::new::<millimole_per_liter>(0.1);
            cc_initializer.notify_composition_change(TestBloodVessel::ThoracicAorta, Substance::GLC, threshold);
            cc_initializer.attach_vessel(TestBloodVessel::InferiorVenaCava);
        }

        // fn cc_sim_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType> {
        //     &mut self.cc_sim_connector
        // }
    }

    impl SimModule for TestCircModuleA {
        fn run(&mut self) {
            // let change = BloodCompositionChange::<TestBloodVessel> {
            //     vessel: TestBloodVessel::InferiorVenaCava,
            //     substance: Substance::GLC,
            //     change: MolarConcentration::new::<millimole_per_liter>(0.1 / self.cc_sim_connector().depth() as f64),
            // };

            // self.cc_sim_connector.schedule_event(Time::new::<second>(1.0), change);
        }
    }
    
}