#![allow(non_snake_case)]

use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Add, Sub, AddAssign, SubAssign, Mul, MulAssign, Div, DivAssign};

type Time = crate::units::base::Time<f64>;

macro_rules! ordered_time {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(pub Time);

        impl Into<Time> for $name {
            fn into(self) -> Time {
                self.0
            }
        }

        impl Into<Time> for &$name {
            fn into(self) -> Time {
                self.0
            }
        }

        impl AsRef<Time> for $name {
            fn as_ref(&self) -> &Time {
                let $name(ref val) = *self;
                val
            }
        }

        impl AsMut<Time> for $name {
            fn as_mut(&mut self) -> &mut Time {
                let $name(ref mut val) = *self;
                val
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                OrderedFloat(self.0.s).eq(&OrderedFloat(other.0.s))
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Eq for $name {}

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                OrderedFloat(self.0.s).cmp(&OrderedFloat(other.0.s))
            }
        }

        impl Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self(Time::from_s(self.0.s + rhs.0.s))
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0.s += rhs.0.s
            }
        }

        impl Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self(Time::from_s(self.0.s - rhs.0.s))
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0.s -= rhs.0.s
            }
        }

        impl Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self {
                Self(Time::from_s(self.0.s * rhs.0.s))
            }
        }

        impl MulAssign for $name {
            fn mul_assign(&mut self, rhs: Self) {
                self.0.s *= rhs.0.s
            }
        }

        impl Div for $name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self {
                Self(Time::from_s(self.0.s / rhs.0.s))
            }
        }

        impl DivAssign for $name {
            fn div_assign(&mut self, rhs: Self) {
                self.0.s /= rhs.0.s
            }
        }

        impl Add<f64> for $name {
            type Output = Self;

            fn add(self, rhs: f64) -> Self {
                Self(Time::from_s(self.0.s + rhs))
            }
        }

        impl AddAssign<f64> for $name {
            fn add_assign(&mut self, rhs: f64) {
                self.0.s += rhs
            }
        }

        impl Sub<f64> for $name {
            type Output = Self;

            fn sub(self, rhs: f64) -> Self {
                Self(Time::from_s(self.0.s - rhs))
            }
        }

        impl SubAssign<f64> for $name {
            fn sub_assign(&mut self, rhs: f64) {
                self.0.s -= rhs
            }
        }

        impl Mul<f64> for $name {
            type Output = Self;

            fn mul(self, rhs: f64) -> Self {
                Self(Time::from_s(self.0.s * rhs))
            }
        }

        impl MulAssign<f64> for $name {
            fn mul_assign(&mut self, rhs: f64) {
                self.0.s *= rhs
            }
        }

        impl Div<f64> for $name {
            type Output = Self;

            fn div(self, rhs: f64) -> Self {
                Self(Time::from_s(self.0.s / rhs))
            }
        }

        impl DivAssign<f64> for $name {
            fn div_assign(&mut self, rhs: f64) {
                self.0.s /= rhs
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }


        impl $name {

            /// Returns the standard unit name of time: "seconds"
            pub fn unit_name() -> &'static str { Time::unit_name() }

            /// Returns the abbreviated name or symbol of time: "s" for seconds
            pub fn unit_symbol() -> &'static str { Time::unit_symbol() }

            /// Returns a new time value from the given number of seconds
            ///
            /// # Arguments
            /// * `s` - Any number-like type, representing a quantity of seconds
            pub fn from_s(s: f64) -> Self { Self(Time::from_s(s)) }

            /// Returns a copy of this time value in seconds
            pub fn to_s(&self) -> f64 { self.0.to_s() }

            /// Returns a new time value from the given number of seconds
            ///
            /// # Arguments
            /// * `seconds` - Any number-like type, representing a quantity of seconds
            pub fn from_seconds(seconds: f64) -> Self { Self(Time::from_seconds(seconds)) }

            /// Returns a copy of this time value in seconds
            pub fn to_seconds(&self) -> f64 { self.0.to_seconds() }

            /// Returns a copy of this time value in milliseconds
            pub fn to_ms(&self) -> f64 {
            	return self.0.to_ms()
            }

            /// Returns a new time value from the given number of milliseconds
            ///
            /// # Arguments
            /// * `ms` - Any number-like type, representing a quantity of milliseconds
            pub fn from_ms(ms: f64) -> Self {
            	Self(Time::from_ms(ms))
            }

            /// Returns a copy of this time value in microseconds
            pub fn to_us(&self) -> f64 {
            	return self.0.to_us()
            }

            /// Returns a new time value from the given number of microseconds
            ///
            /// # Arguments
            /// * `us` - Any number-like type, representing a quantity of microseconds
            pub fn from_us(us: f64) -> Self {
            	Self(Time::from_us(us))
            }

            /// Returns a copy of this time value in nanoseconds
            pub fn to_ns(&self) -> f64 {
            	return self.0.to_ns()
            }

            /// Returns a new time value from the given number of nanoseconds
            ///
            /// # Arguments
            /// * `ns` - Any number-like type, representing a quantity of nanoseconds
            pub fn from_ns(ns: f64) -> Self {
            	Self(Time::from_ns(ns))
            }

            /// Returns a copy of this time value in picoseconds
            pub fn to_ps(&self) -> f64 {
            	return self.0.to_ps()
            }

            /// Returns a new time value from the given number of picoseconds
            ///
            /// # Arguments
            /// * `ps` - Any number-like type, representing a quantity of picoseconds
            pub fn from_ps(ps: f64) -> Self {
            	Self(Time::from_ps(ps))
            }

            /// Returns a copy of this time value in minutes
            pub fn to_min(&self) -> f64 {
            	return self.0.to_min()
            }

            /// Returns a new time value from the given number of minutes
            ///
            /// # Arguments
            /// * `min` - Any number-like type, representing a quantity of minutes
            pub fn from_min(min: f64) -> Self {
            	Self(Time::from_min(min))
            }

            /// Returns a copy of this time value in hours
            pub fn to_hr(&self) -> f64 {
            	return self.0.to_hr()
            }

            /// Returns a new time value from the given number of hours
            ///
            /// # Arguments
            /// * `hr` - Any number-like type, representing a quantity of hours
            pub fn from_hr(hr: f64) -> Self {
            	Self(Time::from_hr(hr))
            }

            /// Returns a copy of this time value in days
            pub fn to_days(&self) -> f64 {
            	return self.0.to_days()
            }

            /// Returns a new time value from the given number of days
            ///
            /// # Arguments
            /// * `days` - Any number-like type, representing a quantity of days
            pub fn from_days(days: f64) -> Self {
            	Self(Time::from_days(days))
            }

            /// Returns a copy of this time value in weeks
            pub fn to_weeks(&self) -> f64 {
            	return self.0.to_weeks()
            }

            /// Returns a new time value from the given number of weeks
            ///
            /// # Arguments
            /// * `weeks` - Any number-like type, representing a quantity of weeks
            pub fn from_weeks(weeks: f64) -> Self {
            	Self(Time::from_weeks(weeks))
            }

            /// Returns a copy of this time value in years
            /// 
            /// *Note: This method is not available for `f32` and other number types lacking the `From<f64>` trait*
            pub fn to_yr(&self) -> f64 {
            	return self.0.to_yr()
            }

            /// Returns a new time value from the given number of years
            ///
            /// # Arguments
            /// * `yr` - Any number-like type, representing a quantity of years
            pub fn from_yr(yr: f64) -> Self {
            	Self(Time::from_yr(yr))
            }

            /// Returns a copy of this time value in millennia
            pub fn to_kyr(&self) -> f64 {
            	return self.0.to_kyr()
            }

            /// Returns a new time value from the given number of millennia
            ///
            /// # Arguments
            /// * `kyr` - Any number-like type, representing a quantity of millennia
            pub fn from_kyr(kyr: f64) -> Self {
            	Self(Time::from_kyr(kyr))
            }

            /// Returns a copy of this time value in million years
            pub fn to_Myr(&self) -> f64 {
            	return self.0.to_Myr()
            }

            /// Returns a new time value from the given number of million years
            ///
            /// # Arguments
            /// * `Myr` - Any number-like type, representing a quantity of million years
            pub fn from_Myr(Myr: f64) -> Self {
            	Self(Time::from_Myr(Myr))
            }

            /// Returns a copy of this time value in billion years
            pub fn to_Gyr(&self) -> f64 {
            	return self.0.to_Gyr()
            }

            /// Returns a new time value from the given number of billion years
            ///
            /// # Arguments
            /// * `Gyr` - Any number-like type, representing a quantity of billion years
            pub fn from_Gyr(Gyr: f64) -> Self {
            	Self(Time::from_Gyr(Gyr))
            }
        }
    };
}

ordered_time!(SimTime);
ordered_time!(SimTimeSpan);

impl SimTime {
    /// Returns a new `SimTimeSpan` from `self` to another
    pub fn span_to(&self, other: &Self) -> SimTimeSpan {
        SimTimeSpan(Time::from_s(other.0.s - self.0.s))
    }
}

impl Add<SimTimeSpan> for SimTime {
    type Output = Self;
    fn add(self, rhs: SimTimeSpan) -> Self::Output {
        Self(Time::from_s(self.0.s + rhs.0.s))
    }
}

impl AddAssign<SimTimeSpan> for SimTime {
    fn add_assign(&mut self, rhs: SimTimeSpan) {
        self.0.s += rhs.0.s
    }
}

impl Sub<SimTimeSpan> for SimTime {
    type Output = Self;
    fn sub(self, rhs: SimTimeSpan) -> Self::Output {
        Self(Time::from_s(self.0.s - rhs.0.s))
    }
}

impl SubAssign<SimTimeSpan> for SimTime {
    fn sub_assign(&mut self, rhs: SimTimeSpan) {
        self.0.s -= rhs.0.s
    }
}

// #[derive(Debug, Clone, Copy)]
// #[repr(transparent)]
// pub struct OrderedTime(pub Time);

// impl Into<Time> for OrderedTime {
//     fn into(self) -> Time {
//         self.0
//     }
// }

// impl Into<Time> for &OrderedTime {
//     fn into(self) -> Time {
//         self.0
//     }
// }

// impl AsRef<Time> for OrderedTime {
//     fn as_ref(&self) -> &Time {
//         let OrderedTime(ref val) = *self;
//         val
//     }
// }

// impl AsMut<Time> for OrderedTime {
//     fn as_mut(&mut self) -> &mut Time {
//         let OrderedTime(ref mut val) = *self;
//         val
//     }
// }

// impl PartialEq for OrderedTime {
//     fn eq(&self, other: &OrderedTime) -> bool {
//         OrderedFloat(self.0.s).eq(&OrderedFloat(other.0.s))
//     }
// }

// impl PartialOrd for OrderedTime {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Eq for OrderedTime {}

// impl Ord for OrderedTime {
//     fn cmp(&self, other: &Self) -> Ordering {
//         OrderedFloat(self.0.s).cmp(&OrderedFloat(other.0.s))
//     }
// }
