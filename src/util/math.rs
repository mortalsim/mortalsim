use std::ops::Bound;


/// Mathematical functions characterizing a change from 0 to amplitude
/// over the given duration
pub enum BoundFn {
    Linear {duration: f64, amplitude: f64},
    Sigmoid {duration: f64, amplitude: f64},
}

impl BoundFn {
    fn call(&self, t: &f64) -> f64 {
        match self {
            BoundFn::Linear {duration, amplitude} => bound_linear(t, duration, amplitude),
            BoundFn::Sigmoid {duration, amplitude} => bound_sigmoid(t, duration, amplitude),
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
fn bound_sigmoid(t: &f64, d: &f64, a: &f64) -> f64 {
    return a / (1.0 + f64::exp(-((2.0 * f64::exp(2.0)) / d) * t - f64::exp(2.0)))
}

/// A linear function with additional bounds to define function shape
/// 
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
fn bound_linear(t: &f64, d: &f64, a: &f64) -> f64 {
    if t < d {
        return a*t/d;
    }
    return *a;
}


