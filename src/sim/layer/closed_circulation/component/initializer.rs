use super::super::vessel::BloodVessel;
use crate::event::Event;
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore};
use std::collections::{HashMap, HashSet};

pub struct ClosedCircInitializer<V: BloodVessel> {
    pub(crate) vessel_connections: HashMap<V, SubstanceStore>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, SubstanceConcentration>>,
    pub(crate) any_substance_notifies: HashSet<V>,
    pub(crate) attach_all: bool,
}

impl<V: BloodVessel> ClosedCircInitializer<V> {
    pub fn new() -> ClosedCircInitializer<V> {
        ClosedCircInitializer {
            vessel_connections: HashMap::new(),
            substance_notifies: HashMap::new(),
            any_substance_notifies: HashSet::new(),
            attach_all: false,
        }
    }

    pub fn notify_composition_change(
        &mut self,
        vessel: V,
        substance: Substance,
        threshold: SubstanceConcentration,
    ) {
        self.vessel_connections
            .insert(vessel, SubstanceStore::new());
        let substance_map = self
            .substance_notifies
            .entry(vessel)
            .or_insert(HashMap::new());
        substance_map.insert(substance, threshold);
    }

    pub fn attach_vessel(&mut self, vessel: V) {
        self.vessel_connections
            .insert(vessel, SubstanceStore::new());
    }

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
        assert!(cc_init.vessel_connections.contains_key(&TestBloodVessel::Aorta));
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
