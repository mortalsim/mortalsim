use super::SubstanceConcentration;
use crate::sim::SimTime;
use crate::math::BoundFn;

#[derive(Debug, Clone)]
pub struct SubstanceChange {
    pub start_time: SimTime,
    pub amount: SubstanceConcentration,
    pub duration: SimTime,
    pub bound_fn: BoundFn,
    pub previous_val: SubstanceConcentration,
}

fn check_duration(duration: SimTime) {
    if duration.to_s() <= 0.0 {
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
        start_time: SimTime,
        amount: SubstanceConcentration,
        duration: SimTime,
        bound_fn: BoundFn,
    ) -> SubstanceChange {
        check_duration(duration);
        SubstanceChange {
            start_time,
            amount,
            duration,
            bound_fn,
            previous_val: SubstanceConcentration::from_mM(0.0),
        }
    }

    /// Retrieves the delta of the concentration change from the previous call, or
    /// the change from 0.0 if this hasn't been called before.
    ///
    /// ### Arguments
    /// * `sim_time` - simulation time to evaluate the change at
    pub fn next_amount(&mut self, sim_time: SimTime) -> SubstanceConcentration {
        let new_val = SubstanceConcentration::from_mM(self.bound_fn.call(
            (sim_time - self.start_time).to_s(),
            self.duration.to_s(),
            self.amount.to_mM(),
        ));

        let result = new_val - self.previous_val;
        log::debug!(
            "new_val: {:?}, previous_val: {:?}, result: {:?}",
            new_val,
            self.previous_val,
            result
        );
        self.previous_val = new_val;
        result
    }
}

#[cfg(test)]
mod tests {

    use super::{BoundFn, SubstanceChange};
    use crate::{mmol_per_L, secs};

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
            diff.to_mM().abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.to_s(),
            result.to_mM(),
            diff.to_mM()
        );

        // Should be the other half
        let result = change.next_amount(duration);
        let diff = result - (amt / 2.0);
        assert!(
            diff.to_mM().abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.to_s(),
            result.to_mM(),
            diff.to_mM()
        );
    }
}
