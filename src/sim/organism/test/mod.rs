use std::cell::Cell;
use std::collections::HashSet;
use std::path::Component;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::sim::component::{ComponentFactory, SimComponent};
pub use crate::sim::layer::circulation::vessel::test::TestBloodVessel;
use crate::sim::layer::core::test::TestComponentA;
pub use crate::sim::layer::nervous::nerve::test::TestNerve;
use crate::sim::layer::LayerManager;
use crate::sim::SimConnector;
use crate::util::{IdGenerator, IdType};

use super::Organism;

static DEFAULT_ID_GEN: OnceLock<Mutex<IdGenerator>> = OnceLock::new();
static DEFAULT_FACTORIES: OnceLock<Mutex<Vec<(IdType, ComponentFactory<'_, TestSim>)>>> = OnceLock::new();

pub struct TestSim {
    connector: SimConnector,
    layer_manager: LayerManager<Self>,
    id_gen: IdGenerator,
}

impl TestSim {

    fn default_id_gen() -> MutexGuard<'static, IdGenerator> {
        DEFAULT_ID_GEN.get_or_init(|| {
            Mutex::new(IdGenerator::new())
        }).lock().unwrap()
    }
    
    fn default_factories() -> MutexGuard<'static, Vec<(u32, ComponentFactory<'static, TestSim>)>> {
        DEFAULT_FACTORIES.get_or_init(|| {
            Mutex::new(Vec::new())
        }).lock().unwrap()
    }

    /// WARNING: when setting defaults, it is essential that two different factories
    /// do NOT produce components with the same id() value. In such a scenario,
    /// initialization of a Sim instance will fail since component ids MUST be unique
    /// for each instance.
    pub fn set_default<T: SimComponent<Self>>(factory: impl FnMut() -> T + 'static + Send + Sync) -> IdType {
        let factory_id = Self::default_id_gen().get_id();
        Self::default_factories().push((factory_id, ComponentFactory::new(factory)));
        factory_id
    }

    pub fn remove_default<T: SimComponent<Self>>(factory_id: &IdType) -> anyhow::Result<()> {
        if let Some((idx, _)) = Self::default_factories()
            .iter().enumerate().find(|(_, (id, _f))| id == factory_id) {
            Self::default_factories().remove(idx);

            return Self::default_id_gen().return_id(*factory_id)
        };
        Err(anyhow!("Invalid factory_id provided"))
    }
}

impl Organism for TestSim {
    type VesselType = TestBloodVessel;
    type NerveType = TestNerve;
    type AnatomyType = TestAnatomicalRegion;
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

#[test]
fn test_default() {
    TestSim::set_default(|| {
        TestComponentA::new()
    });
}