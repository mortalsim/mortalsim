use std::sync::Arc;
use std::any::TypeId;
use std::fmt::{Display, Debug};
use downcast_rs::DowncastSync;
use uuid::Uuid;

// mod blood_events;
// pub use blood_events::*;

pub type EventHandler<T> = dyn FnMut(Arc<T>);

pub trait Event: DowncastSync + Debug {
    fn event_name(&self) -> &str;
}
impl_downcast!(sync Event);

pub struct EventIterator<'a, E: Event> {
    evt_list: Option<Vec::<Arc<E>>>,
    iter_ref: Option<&'a Vec::<Arc<E>>>,
}

// impl<'a, E: Event> Iterator for EventIterator<'a, E> {
//     type Item = Arc<E>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.evt_list.is_some() {
//             self.
//         }
//     }
// }


#[cfg(test)]
pub mod test {

    use super::Event;
    use uuid::Uuid;
    use uom::si::f64::Length;
    use uom::si::f64::AmountOfSubstance;
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventA {
        pub len: Length,
        event_id: Uuid,
    }

    impl TestEventA {
        pub fn new(len: Length) -> TestEventA {
            TestEventA {
                len: len,
                event_id: Uuid::new_v4(),
            }
        }
    }


    impl Event for TestEventA {
        fn event_name(&self) -> &str {"TestEventA"}
    }

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventB {
        pub amt: AmountOfSubstance,
        event_id: Uuid,
    }
    
    impl TestEventB {
        pub fn new(amt: AmountOfSubstance) -> TestEventB {
            TestEventB {
                amt: amt,
                event_id: Uuid::new_v4(),
            }
        }
    }
    
    impl Event for TestEventB {
        fn event_name(&self) -> &str {"TestEventB"}
    }
}
