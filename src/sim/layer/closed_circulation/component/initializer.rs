use super::super::vessel::BloodVessel;
use crate::event::Event;
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore, ConcentrationTracker};
use std::collections::{HashMap, HashSet};

pub struct ClosedCircInitializer<V: BloodVessel> {
    /// BloodVessel connections for the associated component
    pub(crate) vessel_connections: HashSet<V>,
    /// Notifications requested for the associated component
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, ConcentrationTracker>>,
    /// Attached all vessels to the component.
    pub(crate) attach_all: bool,
}

impl<V: BloodVessel> ClosedCircInitializer<V> {
    pub fn new() -> ClosedCircInitializer<V> {
        ClosedCircInitializer {
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
            attach_all: false,
        }
    }

    /// Registers the associated `ClosedCircComponent` to `run` whenever the
    /// provided `BloodVessel` is changed to the indicated threshold. Also
    /// automatically attaches the vessel for use by the component.
    ///
    /// ### Arguments
    /// * `vessel`    - `BloodVessel` to notify on changes
    /// * `substance` - `Substance` to notify on changes
    /// * `threshold` - Amount of change that should trigger a `run`
    pub fn notify_composition_change(
        &mut self,
        vessel: V,
        substance: Substance,
        threshold: SubstanceConcentration,
    ) {
        self.vessel_connections
            .insert(vessel);
        let substance_map = self
            .substance_notifies
            .entry(vessel)
            .or_insert(HashMap::new());
        substance_map.insert(substance, ConcentrationTracker::new(threshold));
    }

    /// Attaches a vessel for use by the associated `ClosedCircComponent`
    ///
    /// ### Arguments
    /// * `vessel` - `BloodVessel` this change should take place on
    pub fn attach_vessel(&mut self, vessel: V) {
        self.vessel_connections
            .insert(vessel);
    }

    /// When called, ALL vessels will be attached to the associated
    /// `ClosedCircComponent`. Note that this will cause the component
    /// to `run` at every simulation step
    pub fn attach_all_vessels(&mut self) {
        self.attach_all = true;
    }
}

#[cfg(test)]
pub mod test {
    use crate::{sim::layer::closed_circulation::vessel::test::TestBloodVessel, substance::Substance, util::mmol_per_L};

    use super::ClosedCircInitializer;
    
    #[test]
    fn test_attach_vessel() {
        let mut cc_init = ClosedCircInitializer::<TestBloodVessel>::new();
        cc_init.attach_vessel(TestBloodVessel::Aorta);
        assert!(cc_init.vessel_connections.contains(&TestBloodVessel::Aorta));
    }

    #[test]
    fn test_attach_all() {
        let mut cc_init = ClosedCircInitializer::<TestBloodVessel>::new();
        cc_init.attach_all_vessels();
        assert!(cc_init.attach_all == true);
    }

    #[test]
    fn test_notify() {
        let mut cc_init = ClosedCircInitializer::<TestBloodVessel>::new();
        cc_init.notify_composition_change(TestBloodVessel::Aorta, Substance::CO2, mmol_per_L!(1.0));
        assert!(cc_init.substance_notifies.contains_key(&TestBloodVessel::Aorta));
        assert!(!cc_init.substance_notifies.contains_key(&TestBloodVessel::VenaCava));
    }
}
