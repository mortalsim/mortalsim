mod bioconnector;
pub use bioconnector::BioConnector;

pub trait BioModule {
    fn trigger();
    fn init(connector: BioConnector);
}
