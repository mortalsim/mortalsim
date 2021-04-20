use std::fmt;
use std::str::FromStr;
use std::rc::Rc;
use std::borrow::Borrow;
use std::hash::Hash;

pub trait BloodVessel: FromStr + Hash + Clone + Copy + PartialEq + Eq + fmt::Debug + fmt::Display {}


/// Type of a blood vessel
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum BloodVesselType {
    Vein,
    Artery,
}

/// Custom type containing a reference counted
/// string instance to avoid unnecessary copying.
/// New type is needed to allow borrows to compare
/// with &str type, which is not available in
/// the standard library unfortunately...
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct BloodVesselId(Rc<String>);

impl fmt::Display for BloodVesselId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<str> for BloodVesselId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<String> for BloodVesselId {
    fn borrow(&self) -> &String {
        &self.0
    }
}

impl Borrow<Rc<String>> for BloodVesselId {
    fn borrow(&self) -> &Rc<String> {
        &self.0
    }
}

impl From<&str> for BloodVesselId {
    fn from(val: &str) -> Self {
        Self(Rc::new(String::from(val)))
    }
}

impl Into<String> for BloodVesselId {
    fn into(self) -> String {
        self.0.to_string()
    }
}