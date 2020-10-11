use mopa::Any;
pub mod event_hub;
pub mod time_manager;

pub trait Event: Any {}
mopafy!(Event);