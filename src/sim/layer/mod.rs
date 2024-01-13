pub mod core;
pub mod closed_circulation;
pub mod digestion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    ClosedCirculation,
    Digestion,
}

pub struct LayerManager {

}
