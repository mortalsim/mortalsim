
pub mod digestion_layer;
pub mod component;
pub mod consumable;

pub use digestion_layer::DigestionLayer;

use crate::event::Event;

use self::consumable::Consumable;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DigestionDirection {
    FORWARD,
    BACK,
    EXHAUSTED,
}

#[derive(Debug)]
pub struct ConsumeEvent(Consumable);

impl Event for ConsumeEvent {
    fn event_name(&self) -> &str {
        "ConsumeEvent"
    }
}

#[derive(Debug)]
pub struct EliminateEvent {
    excrement: Consumable,
    direction: DigestionDirection,
}

impl EliminateEvent {
    pub fn new(excrement: Consumable, direction: DigestionDirection) -> Self {
        Self {
            excrement,
            direction,
        }
    }
}

impl Event for EliminateEvent {
    fn event_name(&self) -> &str {
        "EliminateEvent"
    }
}