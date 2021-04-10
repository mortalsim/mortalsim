
/// Type of a blood vessel
#[derive(Debug, Clone, Copy)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

/// A blood vessel of the circulatory system
trait BloodVessel {
    /// Retrieves a list of BloodVessel objects which are immediately upstream
    fn upstream(&self) -> Vec<&'static dyn BloodVessel>;
    
    /// Retrieves a list of mutable BloodVessel objects which are immediately downstream
    fn upstream_mut(&mut self) -> Vec<&'static mut dyn BloodVessel>;
    
    /// Retrieves a list of BloodVessel objects which are immediately downstream
    fn downstream(&self) -> Vec<&'static dyn BloodVessel>;
    
    /// Retrieves a list of mutable BloodVessel objects which are immediately downstream
    fn downstream_mut(&mut self) -> Vec<&'static mut dyn BloodVessel>;
}
