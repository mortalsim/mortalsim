use std::any::{Any, TypeId};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

use crate::event::Event;
use crate::sim::layer::AnatomicalRegionIter;
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::util::IdGenerator;
use crate::IdType;


pub trait Nerve:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Into<&'static str>
{
    type AnatomyType: Clone;
    fn terminal_nerves<'a>() -> NerveIter<'a, Self>;
    fn uplink<'a>(&self) -> NerveIter<'a, Self>;
    fn downlink<'a>(&self) -> NerveIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

pub struct NerveIter<'a, N: Nerve>(pub core::slice::Iter<'a, N>);

impl<'a, N: Nerve> Iterator for NerveIter<'a, N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        Some(self.0.next()?.clone())
    }
}

impl<'a, N: Nerve> ExactSizeIterator for NerveIter<'a, N> {
    fn len(&self) -> usize {
        self.0.len()
    }
}


#[cfg(test)]
pub mod test {
    use crate::sim::{layer::AnatomicalRegionIter, organism::test::TestAnatomicalRegion};
    use std::{collections::HashSet, sync::OnceLock};

    use super::{Nerve, NerveIter};

    #[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
    pub enum TestNerve {
        Brain,
        SpinalCord,
    }

    static BRAIN_LIST: OnceLock<Vec<TestNerve>> = OnceLock::new();
    static SPINALCORD_LIST: OnceLock<Vec<TestNerve>> = OnceLock::new();
    static EMPTY_LIST: OnceLock<Vec<TestNerve>> = OnceLock::new();

    static BRAIN_REGIONS: OnceLock<HashSet<TestAnatomicalRegion>> = OnceLock::new();
    static SPINALCORD_REGIONS: OnceLock<HashSet<TestAnatomicalRegion>> = OnceLock::new();

    impl TestNerve {
        fn empty() -> &'static Vec<TestNerve> {
            EMPTY_LIST.get_or_init(|| Vec::new())
        }
        fn brain_list() -> &'static Vec<TestNerve> {
            BRAIN_LIST.get_or_init(|| {
                let mut nerve_list = Vec::new();
                nerve_list.push(TestNerve::Brain);
                nerve_list
            })
        }
        fn spinalcord_list() -> &'static Vec<TestNerve> {
            SPINALCORD_LIST.get_or_init(|| {
                let mut nerve_list = Vec::new();
                nerve_list.push(TestNerve::SpinalCord);
                nerve_list
            })
        }
    }

    impl Nerve for TestNerve {
        type AnatomyType = TestAnatomicalRegion;

        fn terminal_nerves<'a>() -> NerveIter<'a, Self> {
            NerveIter(Self::spinalcord_list().iter())
        }

        fn uplink<'a>(&self) -> NerveIter<'a, Self> {
            match self {
                TestNerve::Brain => NerveIter(Self::empty().iter()),
                TestNerve::SpinalCord => NerveIter(Self::brain_list().iter()),
            }
        }

        fn downlink<'a>(&self) -> NerveIter<'a, Self> {
            match self {
                TestNerve::Brain => NerveIter(Self::spinalcord_list().iter()),
                TestNerve::SpinalCord => NerveIter(Self::empty().iter()),
            }
        }

        fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
            match self {
                TestNerve::Brain => AnatomicalRegionIter(
                    BRAIN_REGIONS
                        .get_or_init(|| {
                            let mut region_list = HashSet::new();
                            region_list.insert(TestAnatomicalRegion::Head);
                            region_list
                        })
                        .iter(),
                ),
                TestNerve::SpinalCord => AnatomicalRegionIter(
                    SPINALCORD_REGIONS
                        .get_or_init(|| {
                            let mut region_list = HashSet::new();
                            region_list.insert(TestAnatomicalRegion::Torso);
                            region_list
                        })
                        .iter(),
                ),
            }
        }
    }
}
