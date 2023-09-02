use super::super::vessel::{BloodVessel, BloodVesselType, VesselIter};
use crate::substance::{SubstanceConcentration, Substance, SubstanceStore};
use super::ClosedCircInitializer;
use anyhow::Result;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct ClosedCircConnector<V: BloodVessel> {
    pub(crate) vessel_connections: HashMap<V, SubstanceStore>,
    pub(crate) substance_notifies: HashMap<V, HashMap<Substance, SubstanceConcentration>>,
}

impl<V: BloodVessel> ClosedCircConnector<V> {
    pub fn new() -> ClosedCircConnector<V> {
        ClosedCircConnector {
            vessel_connections: HashMap::new(),
            substance_notifies: HashMap::new(),
        }
    }

    pub fn blood_store(&self, vessel: &V) -> Option<&SubstanceStore> {
        self.vessel_connections.get(vessel)
    }
}

#[cfg(test)]
pub mod test {


    use crate::{sim::layer::closed_circulation::vessel::test::TestBloodVessel, substance::SubstanceStore};

    use super::ClosedCircConnector;

    #[test]
    fn test_blood_store() {
        let mut ccc = ClosedCircConnector::<TestBloodVessel>::new();
        ccc.vessel_connections.insert(TestBloodVessel::Aorta, SubstanceStore::new());

        assert!(ccc.blood_store(&TestBloodVessel::Aorta).is_some());
        assert!(ccc.blood_store(&TestBloodVessel::VenaCava).is_none());
    }

}
