use mopa::Any;
use uom::si::f64::Length;
use uom::si::f64::AmountOfSubstance;
use uom::si::length::meter;
use uom::si::amount_of_substance::mole;
pub mod event_hub;
pub mod time_manager;
pub mod event_listener;

pub type EventHandler<T> = dyn FnMut(Box<T>);

pub trait Event: Any {}
mopafy!(Event);

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
