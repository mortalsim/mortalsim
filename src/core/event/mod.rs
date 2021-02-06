use downcast_rs::DowncastSync;
use uom::si::f64::Length;
use uom::si::f64::AmountOfSubstance;
use uom::si::length::meter;
use uom::si::amount_of_substance::mole;
pub mod event_hub;
pub mod time_manager;
pub mod event_listener;

pub type EventHandler<T> = dyn FnMut(Box<T>);

pub trait Event: DowncastSync {}
impl_downcast!(Event);

#[cfg(test)]
#[derive(Debug)]
struct TestEventA {
    len: Length
}

#[cfg(test)]
impl Event for TestEventA {}

#[cfg(test)]
#[derive(Debug)]
struct TestEventB {
    amt: AmountOfSubstance
}

#[cfg(test)]
impl Event for TestEventB {}
