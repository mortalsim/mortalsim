use super::MolarConcentration;
use crate::core::sim::Time;
use crate::substance::Substance;
use crate::util::math::{bound_linear, bound_sigmoid};
use crate::util::BoundFn;
use anyhow::{Error, Result};

// Need to use the BASE UNIT for molar concentration
// otherwise the universe becomes unstable
use uom::si::molar_concentration::mole_per_cubic_meter;

#[derive(Debug, Clone)]
pub struct SubstanceChange {
    pub start_time: Time,
    pub amount: MolarConcentration,
    pub duration: Time,
    pub bound_fn: BoundFn,
    pub previous_val: MolarConcentration,
}

fn check_duration(duration: Time) {
    if duration.value <= 0.0 {
        panic!("SubstanceChange duration must be greater than 0!")
    }
}

impl SubstanceChange {
    /// Constructs a new SubstanceChange with the given parameters
    ///
    /// ### Arguments
    /// * `start_time` - simulation time to start the change
    /// * `substance`  - the substance to change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns a new SubstanceChange starting at 0.0
    pub fn new(
        start_time: Time,
        amount: MolarConcentration,
        duration: Time,
        bound_fn: BoundFn,
    ) -> SubstanceChange {
        check_duration(duration);
        SubstanceChange {
            start_time,
            amount,
            duration,
            bound_fn,
            previous_val: MolarConcentration::new::<mole_per_cubic_meter>(0.0),
        }
    }

    /// Retrieves the delta of the concentration change from the previous call, or
    /// the change from 0.0 if this hasn't been called before.
    ///
    /// ### Arguments
    /// * `sim_time` - simulation time to evaluate the change at
    pub fn next_amount(&mut self, sim_time: Time) -> MolarConcentration {
        let new_val = MolarConcentration::new::<mole_per_cubic_meter>(self.bound_fn.call(
            (sim_time - self.start_time).value,
            self.duration.value,
            self.amount.value,
        ));

        let result = new_val - self.previous_val;
        println!(
            "new_val: {:?}, previous_val: {:?}, result: {:?}",
            new_val, self.previous_val, result
        );
        self.previous_val = new_val;
        result
    }
}

#[cfg(test)]
mod tests {

    use super::{BoundFn, MolarConcentration, SubstanceChange, Time};
    use crate::util::{mmol_per_L, secs};
    use uom::si::molar_concentration::millimole_per_liter;
    use uom::si::time::second;

    #[test]
    fn new_change() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), secs!(1.0), BoundFn::Sigmoid);
    }

    #[test]
    #[should_panic(expected = "SubstanceChange duration must be greater than 0")]
    fn new_change_neg_duration() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), secs!(-1.0), BoundFn::Sigmoid);
    }

    #[test]
    #[should_panic(expected = "SubstanceChange duration must be greater than 0")]
    fn new_change_zero_duration() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), secs!(0.0), BoundFn::Sigmoid);
    }

    #[test]
    fn change_call() {
        let amt = mmol_per_L!(1.0);
        let duration = secs!(1.0);
        let mut change = SubstanceChange::new(secs!(0.0), amt, duration, BoundFn::Sigmoid);

        // Should be at about half amplitude
        let sim_time = duration / 2.0;
        let result = change.next_amount(sim_time);
        let diff = result - (amt / 2.0);
        assert!(
            diff.value.abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.value,
            result.value,
            diff.value
        );

        // Should be the other half
        let result = change.next_amount(duration);
        let diff = result - (amt / 2.0);
        assert!(
            diff.value.abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.value,
            result.value,
            diff.value
        );
    }
}
