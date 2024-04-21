use std::sync::{Arc, Mutex, RwLock, Weak};

use super::SubstanceConcentration;
use crate::sim::SimTime;
use crate::math::BoundFn;
use crate::SimTimeSpan;

pub trait SubstanceChangeItem {
    /// Retrieves the delta of the concentration change from the previous call, or
    /// the change from 0.0 if this hasn't been called before.
    ///
    /// ### Arguments
    /// * `cur_time` - current simulation time to evaluate the change at
    /// * `start_time` - simulation time when the change began
    fn next_amount(&mut self, sim_time: SimTime) -> SubstanceConcentration;

    /// Simulation time when the change begins
    fn start_time(&self) -> SimTime;

    /// Duration of the change
    fn duration(&self) -> SimTimeSpan;
}

#[derive(Debug, Clone)]
struct SubstanceChangeFn {
    pub start_time: SimTime,
    pub amount: SubstanceConcentration,
    pub duration: SimTimeSpan,
    pub bound_fn: BoundFn,
}

fn check_duration(duration: SimTimeSpan) {
    if duration.to_s() <= 0.0 {
        panic!("SubstanceChange duration must be greater than 0!")
    }
}

impl SubstanceChangeFn {
    /// Constructs a new SubstanceChangeFn with the given parameters
    ///
    /// ### Arguments
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns a new SubstanceChangeFn starting at 0.0
    pub fn new(
        start_time: SimTime,
        amount: SubstanceConcentration,
        duration: SimTimeSpan,
        bound_fn: BoundFn,
    ) -> Self {
        check_duration(duration);
        Self {
            start_time,
            amount,
            duration,
            bound_fn,
        }
    }

    /// Retrieves the delta of the concentration change from the previous call, or
    /// the change from 0.0 if this hasn't been called before.
    ///
    /// ### Arguments
    /// * `cur_time` - current simulation time to evaluate the change at
    /// * `start_time` - simulation time when the change began
    pub fn next_amount(&self, cur_time: SimTime) -> SubstanceConcentration {
        SubstanceConcentration::from_mM(self.bound_fn.call(
            (cur_time - self.start_time).to_s(),
            self.duration.to_s(),
            self.amount.to_mM(),
        ))
    }

}

/// Representation of a substance change
#[derive(Debug, Clone)]
pub struct SubstanceChange {
    cancel_time: Arc<RwLock<SimTime>>,
    prev_val: SubstanceConcentration,
    change_fn: Arc<SubstanceChangeFn>,
}

impl SubstanceChange {
    /// Constructs a new SubstanceChange with the given parameters
    ///
    /// ### Arguments
    /// * `start_time` - simulation time to start the change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns a new SubstanceChange starting at 0.0
    pub fn new(
        start_time: SimTime,
        amount: SubstanceConcentration,
        duration: SimTimeSpan,
        bound_fn: BoundFn,
    ) -> Self {
        check_duration(duration);
        Self {
            cancel_time: Arc::new(RwLock::new(SimTime::from_s(-1.0))),
            prev_val: SubstanceConcentration::from_mM(0.0),
            change_fn: Arc::new(SubstanceChangeFn::new(start_time, amount, duration, bound_fn))
        }
    }

    /// Cancels any changes dependent on this change
    /// ### Arguments
    /// * `cur_time` - current simulation time to evaluate the change at
    pub fn cancel(&self, cur_time: SimTime) {
        *self.cancel_time.write().unwrap() = cur_time;
    }
}

impl SubstanceChangeItem for SubstanceChange {
    fn next_amount(&mut self, sim_time: SimTime) -> SubstanceConcentration {
        let next = self.change_fn.next_amount(sim_time);
        let result = next - self.prev_val;
        self.prev_val = next;
        result
    }

    fn start_time(&self) -> SimTime {
        self.change_fn.start_time
    }

    fn duration(&self) -> SimTimeSpan {
        self.change_fn.duration
    }
}

/// A change dependent on a change elsewhere
#[derive(Debug, Clone)]
pub struct DependentSubstanceChange {
    time_diff: SimTimeSpan,
    cancel_time: Arc<RwLock<SimTime>>,
    prev_val: SubstanceConcentration,
    change_fn: Arc<SubstanceChangeFn>,
}

impl DependentSubstanceChange {
    /// Constructs a new SubstanceChange with the given parameters
    ///
    /// ### Arguments
    /// * `start_time` - simulation time to start the change
    /// * `amount`     - total concentration change to take place
    /// * `duration`   - amount of time over which the change takes place
    /// * `bound_fn`   - the shape of the function
    ///
    /// Returns a new SubstanceChange starting at 0.0
    pub fn new(
        start_time: SimTime,
        change: &SubstanceChange,
    ) -> Self {
        if start_time <= change.start_time() {
            panic!("DependentSubstanceChange start_time must be greater than the source change's start_time")
        }
        Self {
            time_diff: change.start_time().span_to(&start_time),
            cancel_time: change.cancel_time.clone(),
            prev_val: SubstanceConcentration::from_mM(0.0),
            change_fn: change.change_fn.clone(),
        }
    }

    pub fn is_cancelled(&self, sim_time: SimTime) -> bool {
        let cancel_time = *self.cancel_time.read().unwrap();
        cancel_time > SimTime::from_s(0.0) && cancel_time + self.time_diff < sim_time
    }
}

impl SubstanceChangeItem for DependentSubstanceChange {
    fn next_amount(&mut self, sim_time: SimTime) -> SubstanceConcentration {
        // Check cancellation of the original
        if self.is_cancelled(sim_time) {
            return SubstanceConcentration::from_M(0.0);
        }

        let next = self.change_fn.next_amount(sim_time - self.time_diff);
        let result = next - self.prev_val;
        self.prev_val = next;
        result
    }

    fn start_time(&self) -> SimTime {
        self.change_fn.start_time + self.time_diff
    }

    fn duration(&self) -> SimTimeSpan {
        self.change_fn.duration
    }
}


mod tests {

    use super::{BoundFn, DependentSubstanceChange, SubstanceChange};
    use crate::{mmol_per_L, secs, substance::{change::SubstanceChangeItem, SubstanceConcentration}, SimTimeSpan};

    #[test]
    fn new_change() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), SimTimeSpan::from_s(1.0), BoundFn::Sigmoid);
    }

    #[test]
    #[should_panic(expected = "SubstanceChange duration must be greater than 0")]
    fn new_change_neg_duration() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), SimTimeSpan::from_s(-1.0), BoundFn::Sigmoid);
    }

    #[test]
    #[should_panic(expected = "SubstanceChange duration must be greater than 0")]
    fn new_change_zero_duration() {
        SubstanceChange::new(secs!(0.0), mmol_per_L!(1.0), SimTimeSpan::from_s(0.0), BoundFn::Sigmoid);
    }

    #[test]
    fn change_call() {
        let amt = mmol_per_L!(1.0);
        let duration = SimTimeSpan::from_s(1.0);
        let mut change = SubstanceChange::new(secs!(0.0), amt, duration, BoundFn::Sigmoid);

        // Should be at about half amplitude
        let sim_time = secs!(0.5);
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
        let result = change.next_amount(secs!(1.0));
        let diff = result - (amt / 2.0);
        assert!(
            diff.to_mM().abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.to_s(),
            result.to_mM(),
            diff.to_mM()
        );
    }

    #[test]
    fn dependent_change() {
        let amt = mmol_per_L!(1.0);
        let duration = SimTimeSpan::from_s(1.0);
        let mut change = SubstanceChange::new(secs!(0.0), amt, duration, BoundFn::Sigmoid);

        let mut dep_change = DependentSubstanceChange::new(secs!(0.25), &change);

        // Should be at about half amplitude
        let mut sim_time = secs!(0.75);
        let main_val = change.next_amount(sim_time);
        let result1 = dep_change.next_amount(sim_time);
        assert!(result1 > SubstanceConcentration::from_mM(0.0));

        let diff = result1 - (amt / 2.0);
        assert!(
            diff.to_mM().abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.to_s(),
            result1.to_mM(),
            diff.to_mM()
        );

        // Cancel the main change
        change.cancel(sim_time);

        // Should increase to where the main one was cancelled
        sim_time = secs!(1.0);
        let result2 = dep_change.next_amount(sim_time);
        assert!(result2 > SubstanceConcentration::from_mM(0.0));
        let total_result = result2 + result1;
        let diff = total_result - main_val;
        assert!(
            diff.to_mM().abs() < 0.01,
            "time: {}, result: {}, diff: {}",
            sim_time.to_s(),
            total_result.to_mM(),
            diff.to_mM()
        );


        // Advancing further should result in None
        sim_time = secs!(1.1);
        assert!(dep_change.next_amount(sim_time) == SubstanceConcentration::from_mM(0.0));

    }
}
