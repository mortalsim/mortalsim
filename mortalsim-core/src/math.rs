use std::f64::consts::E;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum BoundFn {
    Linear,
    Sigmoid,
}

impl BoundFn {
    pub fn call(&self, t: f64, d: f64, a: f64) -> f64 {
        match self {
            BoundFn::Linear => bound_linear(t, d, a),
            BoundFn::Sigmoid => bound_sigmoid(t, d, a),
        }
    }
}

/// A mathematical sigmoid / logistic function with additional bounds
/// to define function shape
///
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
pub fn bound_sigmoid(t: f64, d: f64, a: f64) -> f64 {
    return a / (1.0 + f64::exp(-((4.0 * E / d) * t - 2.0 * E)));
}

/// A linear function with additional bounds to define function shape
///
/// ### Arguments
/// * `t` - time
/// * `d` - duration
/// * `a` - amplitude
pub fn bound_linear(t: f64, d: f64, a: f64) -> f64 {
    if t < d {
        return a * t / d;
    }
    return a;
}


mod tests {
    use super::{bound_linear, bound_sigmoid};

    macro_rules! func_tests {
        ($($name:ident: $func:ident, $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (t, d, a, expected) = $value;
                let result = $func(t,d,a);
                assert!((result - expected).abs() < 0.01, "t: {}, result: {}", t, result);
            }
        )*
        }
    }

    func_tests! {
        linear_0:    bound_linear, (0.0, 1.0, 1.0, 0.0),
        linear_1q:   bound_linear, (0.25, 1.0, 1.0, 0.25),
        linear_1h:   bound_linear, (0.5, 1.0, 1.0, 0.5),
        linear_3q:   bound_linear, (0.75, 1.0, 1.0, 0.75),
        linear_1:    bound_linear, (1.0, 1.0, 1.0, 1.0),
        linear_1_1h: bound_linear, (1.5, 1.0, 1.0, 1.0),

        sigmoid_0:    bound_sigmoid, (0.0, 1.0, 1.0, 0.0),
        sigmoid_1q:   bound_sigmoid, (0.25, 1.0, 1.0, 0.0619),
        sigmoid_1h:   bound_sigmoid, (0.5, 1.0, 1.0, 0.5),
        sigmoid_3q:   bound_sigmoid, (0.75, 1.0, 1.0, 0.9381),
        sigmoid_1:    bound_sigmoid, (1.0, 1.0, 1.0, 1.0),
        sigmoid_1_1h: bound_sigmoid, (1.5, 1.0, 1.0, 1.0),
    }

    // #[test]
    // fn test_sigmoid() {

    //     let tests = [

    //     ]

    //     let result = bound_sigmoid(0.0, 1.0, 1.0);
    //     assert!((result - 0.0).abs() < 0.0001, "result: {}", result);

    //     let result = bound_sigmoid(0.25, 1.0, 1.0);
    //     assert!((result - 0.0619).abs() < 0.0001, "result: {}", result);

    //     let result = bound_sigmoid(0.5, 1.0, 1.0);
    //     assert!((result - 0.5).abs() < 0.0001, "result: {}", result);

    //     let result = bound_sigmoid(0.75, 1.0, 1.0);
    //     assert!((result - 0.9381).abs() < 0.0001, "result: {}", result);

    //     let result = bound_sigmoid(1.0, 1.0, 1.0);
    //     assert!((result - 1.0).abs() < 0.0001, "result: {}", result);

    //     let result = bound_sigmoid(1.5, 1.0, 1.0);
    //     assert!((result - 1.0).abs() < 0.0001, "result: {}", result);
    // }
}
