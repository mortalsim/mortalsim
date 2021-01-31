use mopa::Any;
pub mod event_hub;
pub mod time_manager;
pub mod event_listener;

pub type EventHandler<T> = dyn FnMut(Box<T>);

pub trait Event: Any {}
mopafy!(Event);


#[cfg(test)]
pub struct TestEventA {
    value: i32
}

#[cfg(test)]
impl Event for TestEventA {}

#[cfg(test)]
pub struct TestEventB {
    value: String
}

#[cfg(test)]
impl Event for TestEventB {}