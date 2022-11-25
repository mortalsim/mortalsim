use anyhow::{Result, Error};
use crate::core::sim::Time;
use crate::substance::Substance;
use crate::util::math::{bound_linear, bound_sigmoid};
use crate::util::BoundFn;
use super::MolarConcentration;

// Need to use the BASE UNIT for molar concentration
// otherwise the universe becomes unstable
use uom::si::molar_concentration::mole_per_cubic_meter;

#[derive(Debug, Clone)]
pub struct SubstanceChange {
    pub(crate) start_time: Time,
    pub(crate) amount: MolarConcentration,
    pub(crate) duration: Time,
    pub(crate) bound_fn: BoundFn,
    pub(crate) previous_val: MolarConcentration,
}

fn check_duration(duration: Time) {
    if duration.value <= 0.0 {
        panic!("SubstanceChange duration must be greater than 0!")
    }
}

impl SubstanceChange {
    pub fn new(start_time: Time, amount: MolarConcentration, duration: Time, bound_fn: BoundFn) -> SubstanceChange {
        check_duration(duration);
        SubstanceChange {
            start_time: start_time,
            amount,
            duration,
            bound_fn,
            previous_val: MolarConcentration::new::<mole_per_cubic_meter>(0.0),
        }
    }

    pub fn call(&self, sim_time: Time) -> MolarConcentration {
        MolarConcentration::new::<mole_per_cubic_meter>(
            self.bound_fn.call(&sim_time.value, &self.duration.value, &self.amount.value)
        )
    }
}
