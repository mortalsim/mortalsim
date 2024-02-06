
pub mod digestion_layer;
pub mod consumable;
pub mod component;

pub use digestion_layer::*;
pub use consumable::*;
pub use component::*;

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
