use std::f64::consts::E;
pub mod quantity_wrapper;
pub mod id_gen;
pub mod version;

pub use quantity_wrapper::*;
pub use id_gen::{IdGenerator, IdType};
pub use version::Version;

/// The mathematical sigmoid / logistic function with additional bounds
/// to define function shape
/// 
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
fn bound_sigmoid(t: &f64, d: &f64, a: &f64) -> f64 {
    return a / (1.0 + f64::powf(E, ((4.0 * E) / d) * t - 2.0 * E))
}
