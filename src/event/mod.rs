use downcast_rs::Downcast;
use dyn_clone::DynClone;
use std::fmt::Debug;
use std::sync::Arc;

pub type EventHandler<T> = dyn FnMut(Box<T>);

pub trait Event: Debug + Send + Downcast + DynClone {}

dyn_clone::clone_trait_object!(Event);
impl_downcast!(Event);

pub struct EventIterator<'a, E: Event> {
    evt_list: Option<Vec<Box<E>>>,
    iter_ref: Option<&'a Vec<Box<E>>>,
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

    impl Event for TestEventA {}

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

    impl Event for TestEventB {}
}
