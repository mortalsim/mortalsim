use downcast_rs::DowncastSync;
use uom::si::f64::Length;
use uom::si::f64::AmountOfSubstance;
use uom::si::length::meter;
use uom::si::amount_of_substance::mole;

pub type EventHandler<T> = dyn FnMut(Box<T>);

pub trait Event: DowncastSync {}
impl_downcast!(Event);

#[cfg(test)]
#[derive(Debug)]
pub struct TestEventA {
    pub len: Length
}

#[cfg(test)]
impl Event for TestEventA {}

#[cfg(test)]
#[derive(Debug)]
pub struct TestEventB {
    pub amt: AmountOfSubstance
}

#[cfg(test)]
impl Event for TestEventB {}
