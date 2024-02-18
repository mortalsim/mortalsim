
use crate::sim::Organism;
use crate::units::base::{Distance, Mass, Temperature};
use crate::units::mechanical::{Frequency, Force, Pressure};

use super::{Event, NumType};

macro_rules! unit_wrapper {
    ($name:ident, $unit:ty) => {
        impl Event for $name {}

        impl AsRef<$unit> for $name {
            fn as_ref(&self) -> &$unit {
                &self.0
            }
        }

        impl AsMut<$unit> for $name {
            fn as_mut(&mut self) -> &mut $unit {
                &mut self.0
            }
        }

        impl Into<$unit> for $name {
            fn into(self) -> $unit {
                self.0
            }
        }
    };
}

/// Event indicating a change of heart contraction rate or pulse
#[derive(Debug, Clone, Copy, PartialEq)]
struct HeartRate(Frequency<NumType>);
unit_wrapper!(HeartRate, Frequency<NumType>);

/// Event indicating a change of core body temperature
#[derive(Debug, Clone, Copy, PartialEq)]
struct CoreBodyTemp(Temperature<NumType>);
unit_wrapper!(CoreBodyTemp, Temperature<NumType>);

/// Event indicating a change of aortic blood pressure
#[derive(Debug, Clone, Copy, PartialEq)]
struct AorticBloodPressure(Pressure<NumType>);
unit_wrapper!(AorticBloodPressure, Pressure<NumType>);

/// Event indicating a change of respiration rate
#[derive(Debug, Clone, Copy, PartialEq)]
struct RespiratoryRate(Frequency<NumType>);
unit_wrapper!(RespiratoryRate, Frequency<NumType>);

/// Event indicating a change in height
#[derive(Debug, Clone, Copy, PartialEq)]
struct Height(Distance<NumType>);
unit_wrapper!(Height, Distance<NumType>);

/// Event indicating a change in total body mass
#[derive(Debug, Clone, Copy, PartialEq)]
struct BodyMass(Mass<NumType>);
unit_wrapper!(BodyMass, Mass<NumType>);

/// Event indicating a change in level of consciousness
/// See https://www.ncbi.nlm.nih.gov/books/NBK380/
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount, EnumIs, EnumIter)]
enum ConciousLevel {
    Alert,
    Clouded,
    Confused,
    Lethargic,
    Obtundated,
    Stupor,
    Coma,
}

impl Event for ConciousLevel {}
