mod bioconnector;
pub use bioconnector::BioConnector;

pub trait BioModule {
    fn init(connector: &mut BioConnector);
    fn trigger(connector: &mut BioConnector);
}
