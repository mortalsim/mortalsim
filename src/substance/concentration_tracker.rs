use crate::{substance::SubstanceConcentration, util::mmol_per_L};

pub struct ConcentrationTracker {
    pub threshold: SubstanceConcentration,
    previous_val: SubstanceConcentration,
}

impl ConcentrationTracker {
    pub fn new(threshold: SubstanceConcentration) -> ConcentrationTracker {
        ConcentrationTracker {
            threshold,
            previous_val: mmol_per_L!(0.0),
        }
    }
    pub fn update(&mut self, val: SubstanceConcentration) {
        self.previous_val = val;
    }
    pub fn check(&self, val: SubstanceConcentration) -> bool {
        (val >= self.previous_val && val - self.previous_val > self.threshold)
            || (val < self.previous_val && self.previous_val - val > self.threshold)
    }
}

#[cfg(test)]
pub mod test {
    use simple_si_units::chemical::Concentration;

    use crate::substance::SubstanceConcentration;

    use super::ConcentrationTracker;

    #[test]
    fn test_tracker() {
        let mut tracker = ConcentrationTracker::new(SubstanceConcentration::from_M(1.0));
        assert!(!tracker.check(Concentration::from_M(0.0)));
        assert!(!tracker.check(Concentration::from_M(0.5)));
        assert!(tracker.check(Concentration::from_M(1.5)));
        tracker.update(Concentration::from_M(1.5));
        assert!(!tracker.check(Concentration::from_M(1.7)));
    }
}
