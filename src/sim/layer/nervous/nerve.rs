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

static ID_GEN: OnceLock<Mutex<IdGenerator>> = OnceLock::new();

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

pub struct NerveSignal<O: Organism> {
    id: IdType,
    path: Vec<O::NerveType>,
    message: Box<dyn Event>,
    send_time: SimTime,
    blocked: bool,
}

impl<O: Organism> NerveSignal<O> {
    pub fn new<T: Event>(
        message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<Self> {
        if neural_path.is_empty() {
            return Err(anyhow!("Neural path cannot be empty!"));
        }
        for idx in 0..(neural_path.len() - 1) {
            let cur_nerve = neural_path.get(idx).unwrap();
            let next_nerve = neural_path.get(idx + 1).unwrap();
            // Ensure each section of the path is valid
            if !cur_nerve.downlink().any(|d| d == *next_nerve) {
                return Err(anyhow!("Invalid link from {} to {}", cur_nerve, next_nerve));
            }
        }

        Ok(Self {
            id: ID_GEN
                .get_or_init(|| Mutex::new(IdGenerator::new()))
                .lock()
                .unwrap()
                .get_id(),
            path: neural_path,
            message: Box::new(message),
            send_time,
            blocked: false,
        })
    }

    pub fn id(&self) -> IdType {
        self.id
    }

    pub fn is_blocked(&self) -> bool {
        self.blocked
    }

    pub fn block(&mut self) {
        self.blocked = true;
    }

    pub fn unblock(&mut self) {
        self.blocked = false;
    }

    pub fn neural_path(&self) -> NerveIter<O::NerveType> {
        NerveIter(self.path.iter())
    }

    pub fn send_time(&self) -> SimTime {
        self.send_time
    }

    pub fn type_id(&self) -> TypeId {
        self.message.type_id()
    }

    pub fn message<T: Event>(&self) -> &'_ T {
        self.message
            .downcast_ref::<T>()
            .expect("Invalid message type")
    }

    pub fn message_mut<T: Event>(&mut self) -> &'_ mut T {
        self.message
            .downcast_mut::<T>()
            .expect("Invalid message type")
    }
}

impl<O: Organism> Drop for NerveSignal<O> {
    fn drop(&mut self) {
        if let Some(gen) = ID_GEN.get() {
            gen.lock().unwrap().return_id(self.id()).unwrap();
        }
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
