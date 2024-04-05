pub mod id_gen;
pub mod math;
pub mod quantity_wrapper;
pub mod version;

pub use id_gen::{IdGenerator, IdType};
pub use math::BoundFn;
pub use quantity_wrapper::*;
pub use version::Version;

macro_rules! secs {
    ( $x:expr ) => {
        crate::units::base::Time::from_s($x)
    };
}

macro_rules! mmol_per_L {
    ( $x:expr ) => {
        crate::units::chemical::Concentration::from_mM($x)
    };
}

pub(crate) use mmol_per_L;
pub(crate) use secs;
