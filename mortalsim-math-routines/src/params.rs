use mathru::algebra::linear::vector::Vector;
use std::{marker::PhantomData, ops::{Index, IndexMut}};

pub trait Param : Into<usize> + Clone + Copy {
    const COUNT: usize;
}

/// A Vector for ODE parameters.
/// 
/// This construct primarily provides convenient
/// initialization and index methods for Vectors
/// of values associated with ODE parameters.
/// 
/// ```
/// use mortalsim_macros::ParamEnum;
/// use mortalsim_math_routines::params::ParamVec;
/// 
/// #[derive(Clone, Copy, ParamEnum)]
/// enum MyParamEnum {
///     ParamA,
///     ParamB,
/// }
/// 
/// let mut myVec = ParamVec::<MyParamEnum>::new();
/// myVec[MyParamEnum::ParamA] = 1.0_f64;
/// myVec[MyParamEnum::ParamB] = 3.0_f64;
/// ```
#[derive(Clone)]
pub struct ParamVec<T: Param> {
    v: Vector<f64>,
    pd: PhantomData<T>,
}

impl<T: Param> ParamVec<T> {
    /// Constructs a zero-filled Vector for each Param variant
    pub fn new() -> Self {
        Self {
            v: Vector::new_column(vec![0.0; T::COUNT]),
            pd: PhantomData,
        }
    }
}

impl<T: Param> From<Vec<f64>> for ParamVec<T> {
    fn from(value: Vec<f64>) -> Self {
        Self {
            v: Vector::new_column(value),
            pd: PhantomData,
        }
    }
}

impl<T: Param> Into<Vec<f64>> for ParamVec<T> {
    fn into(self) -> Vec<f64> {
        self.v.convert_to_vec()
    }
}

impl<T: Param> From<Vector<f64>> for ParamVec<T> {
    fn from(value: Vector<f64>) -> Self {
        Self {
            v: value,
            pd: PhantomData,
        }
    }
}

impl<T: Param> Into<Vector<f64>> for ParamVec<T> {
    fn into(self) -> Vector<f64> {
        self.v
    }
}

impl<T: Param> Index<T> for ParamVec<T> {
    type Output = f64;
    fn index(&self, param: T) -> &Self::Output {
        &self.v[param.into()]
    }
}

impl<T: Param> IndexMut<T> for ParamVec<T> {
    fn index_mut(&mut self, param: T) -> &mut Self::Output {
        &mut self.v[param.into()]
    }
}
