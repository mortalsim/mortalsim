use anyhow::{Result, Error};
use crate::core::sim::Time;
use crate::substance::Substance;
use crate::util::math::{bound_linear, bound_sigmoid};
use crate::util::BoundFn;
use super::MolarConcentration;

// Need to use the BASE UNIT for molar concentration
// otherwise the universe becomes unstable
use uom::si::molar_concentration::mole_per_cubic_meter;

// TODO: Make this an Event?!
#[derive(Debug, Clone)]
pub struct SubstanceChange {
    pub(crate) start_time: Option<Time>,
    pub(crate) substance: Substance,
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
    pub fn new(substance: Substance, amount: MolarConcentration, duration: Time, bound_fn: BoundFn) -> SubstanceChange {
        check_duration(duration);
        SubstanceChange {
            start_time: None,
            substance,
            amount,
            duration,
            bound_fn,
            previous_val: MolarConcentration::new::<mole_per_cubic_meter>(0.0),
        }
    }
    
    pub fn new_future(start_time: Time, substance: Substance, amount: MolarConcentration, duration: Time, bound_fn: BoundFn) -> SubstanceChange {
        check_duration(duration);
        SubstanceChange {
            start_time: Some(start_time),
            substance,
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
