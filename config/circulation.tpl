use crate::closed_circulation::{
    BloodVessel, ClosedCirculationSim, ClosedCirculatorySystem, VesselIter,
};
use std::collections::HashSet;

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum ${namespaceCapitalized}BloodVessel {
    ${vesselNames}
}

impl BloodVessel for HumanBloodVessel {
    fn start_vessels<'a>() -> VesselIter<'a, Self> {
        VesselIter {
            iter: START_VESSELS.iter(),
        }
    }
}
