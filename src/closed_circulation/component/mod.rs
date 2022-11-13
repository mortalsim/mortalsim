mod initializer;
mod connector;
pub use initializer::{ClosedCircInitializer};
pub use connector::{ClosedCircConnector};
use crate::core::sim::{SimComponent, SimComponentInitializer, SimConnector};
use super::vessel::BloodVessel;

pub trait ClosedCircSimComponent: SimComponent {
    type VesselType: BloodVessel;

    /// Initializes the component. Should register any `Event` objects to listen for
    /// and set initial state.
    /// 
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the component
    fn init_cc(&mut self, cc_initializer: &mut ClosedCircInitializer<Self::VesselType>);
    
    /// Retrieves the SimComponent portion of this component
    /// 
    /// ### returns
    /// this object as a SimComponent
    fn as_sim_component(&mut self) -> &mut dyn SimComponent;
    
    /// Used by the HumanSim to retrieve a mutable reference to this component's
    /// ClosedCircConnector, which tracks component interactions
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
    use super::ClosedCircSimComponent;
    use super::{SimComponentInitializer, SimConnector, ClosedCircInitializer, ClosedCircConnector};
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

    pub struct TestCircComponentA {}
    impl TestCircComponentA {
        pub fn factory() -> Box<dyn ClosedCircSimComponent<VesselType = TestBloodVessel>> {
            Box::new(TestCircComponentA {})
        }
    }

    impl ClosedCircSimComponent for TestCircComponentA {
        type VesselType = TestBloodVessel;

        fn init(&mut self, initializer: &mut SimComponentInitializer, cc_initializer: &mut ClosedCircInitializer<TestBloodVessel>) {
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
