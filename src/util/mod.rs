pub mod quantity_wrapper;
pub mod id_gen;
pub mod version;
pub mod math;

pub use quantity_wrapper::*;
pub use id_gen::{IdGenerator, IdType};
pub use version::Version;
pub use math::BoundFn;

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

pub(crate) use secs;
pub(crate) use mmol_per_L;
