use downcast_rs::{Downcast, DowncastSync};
use std::fmt::Debug;
use std::sync::Arc;
use std::vec::Drain;

mod vital;
mod infection;
mod wound;

pub use vital::*;
pub use infection::*;
pub use wound::*;

// Numeric type to use for all built-in Events
type NumType = f64;

pub trait Event: Debug + Send + DowncastSync {
    // Indicates whether the event should be considered
    // transient, in which case it will not remain on
    // SimState after emission. Default is true.
    fn transient(&self) -> bool {
        true
    }
}

impl_downcast!(sync Event);

pub struct EventDrainIterator<'a>(pub Drain<'a, Arc<dyn Event>>);

impl<'a> Iterator for EventDrainIterator<'a> {
    type Item = Arc<dyn Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
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

    impl Event for TestEventA {
        fn transient(&self) -> bool {
            false
        }
    }

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
