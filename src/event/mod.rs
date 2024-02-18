use downcast_rs::{Downcast, DowncastSync};
use std::fmt::Debug;
use std::sync::Arc;

mod vital;
mod infection;
mod wound;

pub use vital::*;
pub use infection::*;
pub use wound::*;

// Numeric type to use for all built-in Events
type NumType = f64;

pub trait Event: Debug + Send + DowncastSync {}

impl_downcast!(sync Event);

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
            TestEventA { len: len }
        }
    }

    impl Event for TestEventA {}

    #[derive(Debug, Clone, Copy)]
    pub struct TestEventB {
        pub amt: Amount<f64>,
    }

    impl TestEventB {
        pub fn new(amt: Amount<f64>) -> TestEventB {
            TestEventB { amt: amt }
        }
    }

    impl Event for TestEventB {}
}
