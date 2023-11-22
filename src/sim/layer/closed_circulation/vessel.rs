use std::collections::hash_set;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

pub trait BloodVessel:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Sync + Into<&'static str>
{
    type AnatomyType: Clone;
    fn max_arterial_depth() -> u32;
    fn max_venous_depth() -> u32;
    fn max_cycle() -> u32;
    fn start_vessels<'a>() -> VesselIter<'a, Self>;
    fn arteries<'a>() -> VesselIter<'a, Self>;
    fn veins<'a>() -> VesselIter<'a, Self>;
    fn pre_capillaries<'a>() -> VesselIter<'a, Self>;
    fn post_capillaries<'a>() -> VesselIter<'a, Self>;
    fn vessel_type(&self) -> BloodVesselType;
    fn upstream<'a>(&self) -> VesselIter<'a, Self>;
    fn downstream<'a>(&self) -> VesselIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

pub struct VesselIter<'a, V: BloodVessel>(pub hash_set::Iter<'a, V>);

impl<'a, V: BloodVessel> Iterator for VesselIter<'a, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        Some(self.0.next()?.clone())
    }
}

impl<'a, V: BloodVessel> ExactSizeIterator for VesselIter<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum DummyVessel {}

impl BloodVessel for DummyVessel {
    type AnatomyType = i8;
    fn start_vessels<'a>() -> VesselIter<'a, Self> { panic!() }
    fn arteries<'a>() -> VesselIter<'a, Self> { panic!() }
    fn veins<'a>() -> VesselIter<'a, Self> { panic!() }
    fn pre_capillaries<'a>() -> VesselIter<'a, Self> { panic!() }
    fn post_capillaries<'a>() -> VesselIter<'a, Self> { panic!() }
    fn max_arterial_depth() -> u32 { panic!() }
    fn max_venous_depth() -> u32 { panic!() }
    fn max_cycle() -> u32 { panic!() }
    fn vessel_type(&self) -> BloodVesselType { panic!() }
    fn upstream<'a>(&self) -> VesselIter<'a, Self> { panic!() }
    fn downstream<'a>(&self) -> VesselIter<'a, Self> { panic!() }
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> { panic!() }
}

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;

    use super::{BloodVessel, VesselIter, BloodVesselType, AnatomicalRegionIter};


    lazy_static! {
        static ref START_VESSELS: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::Aorta);
            vessel_list
        };
    }

    lazy_static! {
        static ref ARTERIES: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::Aorta);
            vessel_list
        };
    }

    lazy_static! {
        static ref VEINS: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::VenaCava);
            vessel_list
        };
    }

    lazy_static! {
        static ref PRE_CAPILLARIES: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::Aorta);
            vessel_list
        };
    }
    
    lazy_static! {
        static ref POST_CAPILLARIES: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::VenaCava);
            vessel_list
        };
    }

    lazy_static! {
        static ref AORTA_UPSTREAM: HashSet<TestBloodVessel> = {
            HashSet::new()
        };
    }

    lazy_static! {
        static ref VENACAVA_UPSTREAM: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::Aorta);
            vessel_list
        };
    }
    
    lazy_static! {
        static ref AORTA_DOWNSTREAM: HashSet<TestBloodVessel> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestBloodVessel::VenaCava);
            vessel_list
        };
    }

    lazy_static! {
        static ref VENACAVA_DOWNSTREAM: HashSet<TestBloodVessel> = {
            HashSet::new()
        };
    }

    lazy_static! {
        static ref AORTA_REGIONS: HashSet<TestAnatomicalRegion> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestAnatomicalRegion::Torso);
            vessel_list
        };
    }

    lazy_static! {
        static ref VENACAVA_REGIONS: HashSet<TestAnatomicalRegion> = {
            let mut vessel_list = HashSet::new();
            vessel_list.insert(TestAnatomicalRegion::Torso);
            vessel_list
        };
    }
    
    #[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
    pub enum TestAnatomicalRegion {
        Head,
        Torso,
        LeftArm,
        RightArm,
        LeftLeg,
        RightLeg,
    }

    #[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
    pub enum TestBloodVessel {
        Aorta,
        VenaCava,
    }

    impl BloodVessel for TestBloodVessel {
        type AnatomyType = TestAnatomicalRegion;

        fn max_arterial_depth() -> u32 { 1 }
        fn max_venous_depth() -> u32 { 1 }
        fn max_cycle() -> u32 { 2 }
        fn start_vessels<'a>() -> VesselIter<'a, Self> {
            VesselIter(START_VESSELS.iter())
        }
        fn arteries<'a>() -> VesselIter<'a, Self> {
            VesselIter(ARTERIES.iter())
        }
        fn veins<'a>() -> VesselIter<'a, Self> {
            VesselIter(VEINS.iter())
        }
        fn pre_capillaries<'a>() -> VesselIter<'a, Self> {
            VesselIter(PRE_CAPILLARIES.iter())
        }
        fn post_capillaries<'a>() -> VesselIter<'a, Self> {
            VesselIter(POST_CAPILLARIES.iter())
        }
        fn vessel_type(&self) -> BloodVesselType {
            match self {
                TestBloodVessel::Aorta => BloodVesselType::Artery,
                TestBloodVessel::VenaCava => BloodVesselType::Vein,
            }
        }
        fn upstream<'a>(&self) -> VesselIter<'a, Self> {
            match self {
                TestBloodVessel::Aorta => VesselIter(AORTA_UPSTREAM.iter()),
                TestBloodVessel::VenaCava => VesselIter(VENACAVA_UPSTREAM.iter()),
            }
        }
        fn downstream<'a>(&self) -> VesselIter<'a, Self> {
            match self {
                TestBloodVessel::Aorta => VesselIter(AORTA_DOWNSTREAM.iter()),
                TestBloodVessel::VenaCava => VesselIter(VENACAVA_DOWNSTREAM.iter()),
            }
        }
        fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
            match self {
                TestBloodVessel::Aorta => AnatomicalRegionIter(AORTA_REGIONS.iter()),
                TestBloodVessel::VenaCava => AnatomicalRegionIter(VENACAVA_REGIONS.iter()),
            }
        }
    }

     fn test_depths() {
        assert_eq!(TestBloodVessel::max_arterial_depth(), 1);
        assert_eq!(TestBloodVessel::max_venous_depth(), 1);
        assert_eq!(TestBloodVessel::max_cycle(), 2);
     }

     #[test]
     fn test_start_vessels() {
        assert_eq!(TestBloodVessel::start_vessels().len(), 1);
     }

     #[test]
     fn test_arteries() {
        assert_eq!(TestBloodVessel::arteries().len(), 1);
     }

     #[test]
     fn test_veins() {
        assert_eq!(TestBloodVessel::veins().len(), 1);
     }

     #[test]
     fn test_pre_capillaries() {
        assert_eq!(TestBloodVessel::pre_capillaries().len(), 1);
     }

     #[test]
     fn test_post_capillaries() {
        assert_eq!(TestBloodVessel::post_capillaries().len(), 1);
     }

     #[test]
     fn test_vessel_type() {
        assert_eq!(TestBloodVessel::Aorta.vessel_type(), BloodVesselType::Artery);
        assert_eq!(TestBloodVessel::VenaCava.vessel_type(), BloodVesselType::Vein);
     }

     #[test]
     fn test_upstream() {
        assert_eq!(TestBloodVessel::Aorta.upstream().len(), 0);
        assert_eq!(TestBloodVessel::VenaCava.upstream().len(), 1);
     }

     #[test]
     fn test_downstream() {
        assert_eq!(TestBloodVessel::Aorta.downstream().len(), 1);
        assert_eq!(TestBloodVessel::VenaCava.downstream().len(), 0);
     }

     #[test]
     fn test_regions() {
        let mut expected_regions = HashSet::new();
        expected_regions.insert(TestAnatomicalRegion::Torso);
        assert_eq!(HashSet::from_iter(TestBloodVessel::Aorta.regions()), expected_regions);
        assert_eq!(HashSet::from_iter(TestBloodVessel::VenaCava.regions()), expected_regions);
     }

}
