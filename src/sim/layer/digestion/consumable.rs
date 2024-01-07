use std::{fmt, cell::RefCell, sync::{Mutex, Arc}, rc::Rc};
use crate::{substance::SubstanceStore, util::{IdType, IdGenerator}};

lazy_static! {
    static ref CONSUMABLE_ID_GEN: Mutex<IdGenerator> = Mutex::new(IdGenerator::new());
}

pub struct Consumable {
    pub id: IdType,
    pub store: SubstanceStore,
    pub movement_multiplier: f64,
    pub position: f64,
}

impl fmt::Debug for Consumable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Consumable {{ store: {:?}, movement_multipier: {:?}, position: {:?}}}",
            self.store, self.movement_multiplier, self.position
        )
    }
}

impl Consumable {
    fn new(store: SubstanceStore) -> Consumable {
        Consumable {
            id: CONSUMABLE_ID_GEN.lock().unwrap().get_id(),
            store: store,
            movement_multiplier: 1.0,
            position: 0.0,
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::substance::SubstanceStore;

    use super::Consumable;


    #[test]
    fn test_new_consumable() {
        Consumable::new(SubstanceStore::new());
    }

}
