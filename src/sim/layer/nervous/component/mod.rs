use crate::sim::organism::Organism;
use crate::sim::component::SimComponent;

mod connector;
mod initializer;

pub use connector::{NervousConnector, NerveSignalTransformer};
pub use initializer::NervousInitializer;


pub trait NervousComponent<O: Organism>: SimComponent<O> {

    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn nervous_init(&mut self, nervous_initializer: &mut NervousInitializer<O>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// CirculationConnector, which tracks module interactions
    ///
    /// ### returns
    /// TimeManager to interact with the rest of the simulation
    fn nervous_connector(&mut self) -> &mut NervousConnector<O>;
}

#[cfg(test)]
pub mod test {

}
