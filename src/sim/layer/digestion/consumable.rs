use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::cell::RefCell;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use crate::sim::SimTime;
use crate::substance::{SubstanceStore, Substance};
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::units::geometry::Volume;
use crate::util::IdType;

pub struct Consumable {
    pub(super) store: SubstanceStore,
    volume: Volume<f64>,
    change_map: HashMap<Substance, Vec<IdType>>,
}

impl fmt::Debug for Consumable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Consumable {{ store: {:?}, volume: {:?} }}",
            self.store, self.volume
        )
    }
}

impl Consumable {
    substance_store_wrapper!(store, change_map);

    pub fn new(store: SubstanceStore, volume: Volume<f64>) -> Consumable {
        Consumable {
            store: store,
            volume: volume,
            change_map: HashMap::new(),
        }
    }
    pub fn advance(&mut self, sim_time: SimTime) {
        self.store.advance(sim_time)
    }

    pub fn volume(&self) -> Volume<f64> {
        self.volume
    }

    pub fn set_volume(&mut self, volume: Volume<f64>) -> anyhow::Result<()> {
        if volume <= Volume::from_L(0.0) {
            return Err(anyhow!("Consumable volume cannot be less than zero (set to {:?})", volume))
        }
        self.volume = volume;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::units::geometry::Volume;
    use crate::substance::{SubstanceStore, Substance};
    use crate::util::{mmol_per_L, secs, BoundFn};

    use super::Consumable;


    #[test]
    fn test_new_consumable() {
        Consumable::new(SubstanceStore::new(), Volume::from_L(0.5));
    }

    #[test]
    fn test_advance() {
        let mut consumable = Consumable::new(SubstanceStore::new(), Volume::from_L(0.5));
        consumable.schedule_change(Substance::O2, mmol_per_L!(1.0), secs!(0.0));

    }

}
