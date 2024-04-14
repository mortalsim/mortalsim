use crate::sim::organism::Organism;
use crate::substance::{ConcentrationTracker, Substance, SubstanceConcentration};
use std::collections::{HashMap, HashSet};

pub struct CirculationInitializer<O: Organism> {
    /// BloodVessel connections for the associated component
    pub(crate) vessel_connections: HashSet<O::VesselType>,
    /// Notifications requested for the associated component
    pub(crate) substance_notifies: HashMap<O::VesselType, HashMap<Substance, ConcentrationTracker>>,
    /// Notify any changes to any vessel
    pub(crate) notify_any: bool,
    /// Attached all vessels to the component.
    pub(crate) attach_all: bool,
}

impl<O: Organism> CirculationInitializer<O> {
    pub fn new() -> CirculationInitializer<O> {
        CirculationInitializer {
            vessel_connections: HashSet::new(),
            substance_notifies: HashMap::new(),
            notify_any: false,
            attach_all: false,
        }
    }

    /// Registers the associated `CirculationComponent` to `run` whenever the
    /// provided `BloodVessel` is changed to the indicated threshold. Also
    /// automatically attaches the vessel for use by the component.
    ///
    /// ### Arguments
    /// * `vessel`    - `BloodVessel` to notify on changes
    /// * `substance` - `Substance` to notify on changes
    /// * `threshold` - Amount of change that should trigger a `run`
    pub fn notify_composition_change(
        &mut self,
        vessel: O::VesselType,
        substance: Substance,
        threshold: SubstanceConcentration,
    ) {
        self.vessel_connections.insert(vessel);
        let substance_map = self
            .substance_notifies
            .entry(vessel)
            .or_insert(HashMap::new());
        substance_map.insert(substance, ConcentrationTracker::new(threshold));
    }

    /// When called, ANY change on ANY vessel will trigger the `CirculationComponent`.
    /// Inherently attaches all vessels.
    pub fn notify_any_change(
        &mut self,
    ) {
        self.notify_any = true;
        self.attach_all = true;
    }

    /// Attaches a vessel for use by the associated `CirculationComponent`
    ///
    /// ### Arguments
    /// * `vessel` - `BloodVessel` this change should take place on
    pub fn attach_vessel(&mut self, vessel: O::VesselType) {
        self.vessel_connections.insert(vessel);
    }

    /// When called, ALL vessels will be attached to the associated
    /// `CirculationComponent`.
    pub fn attach_all_vessels(&mut self) {
        self.attach_all = true;
    }
}

#[cfg(test)]
pub mod test {
    use crate::sim::organism::test::{TestBloodVessel, TestOrganism, TestSim};
    use crate::substance::Substance;
    use crate::mmol_per_L;

    use super::CirculationInitializer;

    #[test]
    fn test_attach_vessel() {
        let mut circulation_init = CirculationInitializer::<TestOrganism>::new();
        circulation_init.attach_vessel(TestBloodVessel::Aorta);
        assert!(circulation_init
            .vessel_connections
            .contains(&TestBloodVessel::Aorta));
    }

    #[test]
    fn test_attach_all() {
        let mut circulation_init = CirculationInitializer::<TestOrganism>::new();
        circulation_init.attach_all_vessels();
        assert!(circulation_init.attach_all == true);
    }

    #[test]
    fn test_notify() {
        let mut circulation_init = CirculationInitializer::<TestOrganism>::new();
        circulation_init.notify_composition_change(
            TestBloodVessel::Aorta,
            Substance::CO2,
            mmol_per_L!(1.0),
        );
        assert!(circulation_init
            .substance_notifies
            .contains_key(&TestBloodVessel::Aorta));
        assert!(!circulation_init
            .substance_notifies
            .contains_key(&TestBloodVessel::VenaCava));
    }
}
