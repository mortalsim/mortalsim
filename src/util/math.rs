use std::ops::{Div, Mul};
use ordered_float::NotNan;
use uom::si::Units;
use crate::core::sim::Time;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum BoundFn {
    Linear,
    Sigmoid,
}


impl BoundFn {
    pub fn call(&self, t: &f64, d: &f64, a: &f64) -> f64 {
        match self {
            BoundFn::Linear => bound_linear(t, d, a),
            BoundFn::Sigmoid => bound_sigmoid(t, d, a),
        }
    }
}

/// The mathematical sigmoid / logistic function with additional bounds
/// to define function shape
/// 
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
pub fn bound_sigmoid(t: &f64, d: &f64, a: &f64) -> f64 {
    return a / (1.0 + f64::exp(-((2.0 * f64::exp(2.0)) / d) * t - f64::exp(2.0)))
}

/// A linear function with additional bounds to define function shape
/// 
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
pub fn bound_linear(t: &f64, d: &f64, a: &f64) -> f64 {
    if t < d {
        return a*t/d;
    }
    return *a;
}


