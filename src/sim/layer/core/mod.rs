pub mod component;
pub mod core_layer;

pub use component::{CoreComponent, CoreComponentInitializer, CoreConnector};
pub use core_layer::CoreLayer;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    ClosedCirculation,
}
