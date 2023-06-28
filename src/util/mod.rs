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
        Time::new::<second>($x)
    };
}

macro_rules! mmol_per_L {
    ( $x:expr ) => {
        MolarConcentration::new::<millimole_per_liter>($x)
    };
}

pub(crate) use mmol_per_L;
pub(crate) use secs;
