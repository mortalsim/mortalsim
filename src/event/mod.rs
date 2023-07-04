use downcast_rs::DowncastSync;
use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use uuid::Uuid;

// mod blood_events;
// pub use blood_events::*;

pub type EventHandler<T> = dyn FnMut(Arc<T>);

pub trait Event: DowncastSync + Debug {
    fn event_name(&self) -> &str;
}
impl_downcast!(sync Event);

pub struct EventIterator<'a, E: Event> {
    evt_list: Option<Vec<Arc<E>>>,
    iter_ref: Option<&'a Vec<Arc<E>>>,
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
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use uuid::Uuid;

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventA {
        pub len: Distance<f64>,
        event_id: Uuid,
    }

    impl TestEventA {
        pub fn new(len: Distance<f64>) -> TestEventA {
            TestEventA {
                len: len,
                event_id: Uuid::new_v4(),
            }
        }
    }

    impl Event for TestEventA {
        fn event_name(&self) -> &str {
            "TestEventA"
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventB {
        pub amt: Amount<f64>,
        event_id: Uuid,
    }

    impl TestEventB {
        pub fn new(amt: Amount<f64>) -> TestEventB {
            TestEventB {
                amt: amt,
                event_id: Uuid::new_v4(),
            }
        }
    }

    impl Event for TestEventB {
        fn event_name(&self) -> &str {
            "TestEventB"
        }
    }
}
