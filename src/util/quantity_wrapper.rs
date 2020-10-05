use std::cmp::Ordering;
use uom::si::f64::Time;
use ordered_float::OrderedFloat;

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct OrderedTime(pub Time);

impl OrderedTime {
    /// Get the value out.
    pub fn get_value(self) -> Time {
        self.0
    }
}

impl AsRef<Time> for OrderedTime {
    fn as_ref(&self) -> &Time {
        let OrderedTime(ref val) = *self;
        val
    }
}

impl AsMut<Time> for OrderedTime {
    fn as_mut(&mut self) -> &mut Time {
        let OrderedTime(ref mut val) = *self;
        val
    }
}

impl PartialEq for OrderedTime {
    fn eq(&self, other: &OrderedTime) -> bool {
        OrderedFloat(self.0.value).eq(&OrderedFloat(other.0.value))
    }
}

impl PartialOrd for OrderedTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for OrderedTime {}

impl Ord for OrderedTime {
  fn cmp(&self, other: &Self) -> Ordering {
    OrderedFloat(self.0.value).cmp(&OrderedFloat(other.0.value))
  }
}