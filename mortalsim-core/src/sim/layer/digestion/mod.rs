pub(crate) mod component;
pub(crate) mod consumable;
pub(crate) mod digestion_layer;

mod consumed;
use consumed::Consumed;

pub use component::{DigestionComponent, DigestionConnector, DigestionInitializer};
pub use consumable::Consumable;
pub use digestion_layer::DigestionLayer;

use crate::event::Event;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DigestionDirection {
    FORWARD,
    BACK,
    EXHAUSTED,
}

#[derive(Debug, Clone)]
pub struct ConsumeEvent(pub Consumable);

impl Event for ConsumeEvent {}

#[derive(Debug, Clone)]
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

impl Event for EliminateEvent {}
