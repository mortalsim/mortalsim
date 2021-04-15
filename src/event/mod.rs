use std::sync::Arc;
use std::any::TypeId;
use std::fmt::{Display, Debug};
use downcast_rs::DowncastSync;
use uuid::Uuid;

pub type EventHandler<T> = dyn FnMut(Arc<T>);

pub trait Event: DowncastSync + Debug {
    fn event_name(&self) -> &str;
}
impl_downcast!(sync Event);

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
