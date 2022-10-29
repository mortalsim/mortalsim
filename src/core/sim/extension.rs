use std::any::TypeId;
use super::SimConnector;

pub trait SimExtension {
    fn notify_events(&self) -> Vec<TypeId>;
    fn connectors(&mut self) -> Vec<(&'static str, &mut SimConnector)>;
}
