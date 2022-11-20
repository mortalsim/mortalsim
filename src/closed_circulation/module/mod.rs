mod initializer;
mod connector;
pub use initializer::{ClosedCircInitializer};
pub use connector::{ClosedCircConnector};
use crate::core::sim::{SimModule, SimModuleInitializer, SimConnector};
use super::vessel::BloodVessel;

pub trait ClosedCircSimModule: SimModule {
    type VesselType: BloodVessel;

    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn init_cc(&mut self, cc_initializer: &mut ClosedCircInitializer<Self::VesselType>);
    
    /// Retrieves the SimModule portion of this module
    /// 
    /// ### returns
    /// this object as a SimModule
    fn as_sim_module(&mut self) -> &mut dyn SimModule;
    
    /// Used by the HumanSim to retrieve a mutable reference to this module's
    /// ClosedCircConnector, which tracks module interactions
    /// 
    /// ### returns
    /// SimConnector to interact with the rest of the simulation
    fn get_cc_sim_connector(&mut self) -> &mut ClosedCircConnector<Self::VesselType>;
}

#[cfg(test)]
pub mod test {
    use std::sync::Arc;
    use crate::event::{Event, BloodCompositionChange};
    use crate::event::test::{TestEventA, TestEventB};
    use crate::substance::{Substance, MolarConcentration};
    use crate::substance::Time;
    use super::BloodVessel;
    use super::ClosedCircSimModule;
    use super::{SimModuleInitializer, SimConnector, ClosedCircInitializer, ClosedCircConnector};
    use uom::si::f64::{Length, AmountOfSubstance};
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;
    use uom::si::molar_concentration::millimole_per_liter;
    use uom::si::time::second;

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

    impl BloodVessel for TestBloodVessel {}

    pub struct TestCircModuleA {}
    impl TestCircModuleA {
        pub fn factory() -> Box<dyn ClosedCircSimModule<VesselType = TestBloodVessel>> {
            Box::new(TestCircModuleA {})
        }
    }

    impl ClosedCircSimModule for TestCircModuleA {
        type VesselType = TestBloodVessel;

        fn init(&mut self, initializer: &mut SimModuleInitializer, cc_initializer: &mut ClosedCircInitializer<TestBloodVessel>) {
            initializer.notify(TestEventA::new(Length::new::<meter>(1.0)));
            initializer.notify(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Length::new::<meter>(3.0);
            });

            let threshold = MolarConcentration::new::<millimole_per_liter>(0.1);
            cc_initializer.notify_composition_change(TestBloodVessel::ThoracicAorta, Substance::GLC, threshold);
            cc_initializer.attach_vessel(TestBloodVessel::InferiorVenaCava);
        }
        fn run(&mut self, connector: &mut SimConnector, circ_connector: &mut ClosedCircConnector<TestBloodVessel>) {
            let evt_a = connector.get::<TestEventA>().unwrap();
            assert_eq!(evt_a.len, Length::new::<meter>(3.0));

            log::debug!("Trigger Events: {:?}", connector.trigger_events().collect::<Vec<&Arc<dyn Event>>>());

            let change = BloodCompositionChange::<TestBloodVessel> {
                vessel: TestBloodVessel::InferiorVenaCava,
                substance: Substance::GLC,
                change: MolarConcentration::new::<millimole_per_liter>(0.1 / circ_connector.depth() as f64),
            };

            connector.schedule_event(Time::new::<second>(1.0), change);
        }
    }
    
}
