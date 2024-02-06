use crate::sim::SimTime;
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::{Substance, SubstanceStore};
use crate::units::geometry::Volume;
use crate::util::IdType;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Consumable {
    name: String,
    pub(super) store: SubstanceStore,
    volume: Volume<f64>,
    change_map: HashMap<Substance, Vec<IdType>>,
}

impl Consumable {
    substance_store_wrapper!(store, change_map);

    pub fn new(name: String, store: SubstanceStore, volume: Volume<f64>) -> Consumable {
        Consumable {
            name: name,
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
            return Err(anyhow!(
                "Consumable volume cannot be less than zero (set to {:?})",
                volume
            ));
        }
        self.volume = volume;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::substance::{Substance, SubstanceStore};
    use crate::units::geometry::Volume;
    use crate::util::{mmol_per_L, secs};

    use super::Consumable;

    #[test]
    fn test_new_consumable() {
        Consumable::new(String::new(), SubstanceStore::new(), Volume::from_L(0.5));
    }

    #[test]
    fn test_advance() {
        let mut consumable =
            Consumable::new(String::new(), SubstanceStore::new(), Volume::from_L(0.5));
        consumable.schedule_change(Substance::O2, mmol_per_L!(1.0), secs!(1.0));
    }
}
