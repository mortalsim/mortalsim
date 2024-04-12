macro_rules! secs {
    ( $x:expr ) => {
        $crate::SimTime($crate::units::base::Time::<f64>{ s: $x })
    };
}

macro_rules! mmol_per_L {
    ( $x:expr ) => {
        crate::units::chemical::Concentration::from_mM($x)
    };
}

pub(crate) use mmol_per_L;
pub(crate) use secs;
