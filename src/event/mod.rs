use downcast_rs::DowncastSync;
use std::fmt::Debug;
use std::sync::Arc;

pub type EventHandler<T> = dyn FnMut(Arc<T>);

pub trait Event: DowncastSync + Debug {
    fn event_name(&self) -> &str;
}
impl_downcast!(sync Event);

pub struct EventIterator<'a, E: Event> {
    evt_list: Option<Vec<Arc<E>>>,
    iter_ref: Option<&'a Vec<Arc<E>>>,
}

#[cfg(test)]
pub mod test {

    use super::Event;
    use crate::units::base::Amount;
    use crate::units::base::Distance;

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventA {
        pub len: Distance<f64>,
    }

    impl TestEventA {
        pub fn new(len: Distance<f64>) -> TestEventA {
            TestEventA {
                len: len,
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
    }

    impl TestEventB {
        pub fn new(amt: Amount<f64>) -> TestEventB {
            TestEventB {
                amt: amt,
            }
        }
    }

    impl Event for TestEventB {
        fn event_name(&self) -> &str {
            "TestEventB"
        }
    }
}
