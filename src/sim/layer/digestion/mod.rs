pub mod component;
pub mod consumable;
pub mod digestion_layer;

pub use component::*;
pub use consumable::*;
pub use digestion_layer::*;

use crate::event::Event;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DigestionDirection {
    FORWARD,
    BACK,
    EXHAUSTED,
}

#[derive(Debug, Clone)]
pub struct ConsumeEvent(Consumable);

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
