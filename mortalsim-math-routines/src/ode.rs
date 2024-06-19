use std::cell::RefCell;

pub mod runge_kutta {
    pub mod fixed {
        pub use mathru::analysis::differential_equation::ordinary::solver::explicit::runge_kutta::fixed::*;
    }
}

use mathru::analysis::differential_equation::ordinary::{
    ExplicitInitialValueProblemBuilder,
    ExplicitODE
};

use crate::params::{Param, ParamVec};
use crate::Vector;

type NumType = f64;

/// Solution results for a set of explicit Ordinary Differential Equations
/// 
/// The struct includes the following properties:
/// `constants`: constant values which were unchanging throughout the execution
/// `x_values`: the independent variable of integration (often time)
/// `assignment_results`: variables assigned algebraically at each step of the solution
/// `rate_bound_results`: dependent variables of the integration
pub struct OdeResults<T: Ode> {
    /// Constants for the ODE
    pub constants: ParamVec<T::ConstParam>,
    /// Values of the independent variable of the ODE
    pub x_values: Vec<NumType>,
    /// Assignment parameters at each step of the ODE
    pub assignment_results: Vec<ParamVec<T::AssignParam>>,
    /// Rate bound parameters at each step of the ODE
    pub rate_bound_results: Vec<ParamVec<T::RateParam>>,
}

impl<T: Ode> OdeResults<T> {
    /// Length of the result set (number of x values)
    pub fn len(&self) -> usize {
        self.x_values.len()
    }

    /// Value of x at the given index
    pub fn x(&self, index: usize) -> NumType{
        self.x_values[index]
    }

    /// Value of the given constant variable
    pub fn constant_value(&self, param: T::ConstParam) -> NumType {
        self.constants[param]
    }

    /// Value of the assignment variable at the given index
    pub fn assignment_value(&self, index: usize, param: T::AssignParam) -> NumType {
        self.assignment_results[index][param]
    }

    /// Value of the assignment variable at the *nearest* x value.
    /// 
    /// Internally this uses total_cmp to determine nearness.
    /// See https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp
    pub fn assignment_value_at_x(&self, x: NumType, param: T::AssignParam) -> NumType {
        let (index, _) = self.x_values
            .iter()
            .enumerate()
            .min_by(|(_i1, v1), (_i2, v2)| (*v1 - x).abs().total_cmp(&(*v2 - x).abs()))
            .unwrap();
        self.assignment_value(index, param)
    }

    /// Value of the rate bound variable at the given index
    pub fn rate_bound_value(&self, index: usize, param: T::RateParam) -> NumType {
        self.rate_bound_results[index][param]
    }

    /// Value of the rate bound variable at the *nearest* x value.
    /// 
    /// Internally this uses total_cmp to determine nearness.
    /// See https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp
    pub fn rate_bound_value_at_x(&self, x: NumType, param: T::RateParam) -> NumType {
        let (index, _) = self.x_values
            .iter()
            .enumerate()
            .min_by(|(_i1, v1), (_i2, v2)| (*v1 - x).abs().total_cmp(&(*v2 - x).abs()))
            .unwrap();
        self.rate_bound_value(index, param)
    }
}

/// Representation of a set of explicit Ordinary Differential Equations
/// 
/// To define the Ode, implement this trait for a struct.
pub trait Ode
{
    /// Constant param type for this ODE
    type ConstParam: Param;
    /// Assignment param type for this ODE
    type AssignParam: Param;
    /// Rate bound param type (dependent variables) for this ODE
    type RateParam: Param;

    // The values for constant variables in the ODE
    fn constants(&self) -> ParamVec<Self::ConstParam>;

    /// The initial values for rate-bound (dependend) variables for the ODE
    fn initial_values(
        &self,
        constants: &ParamVec<Self::ConstParam>,
    ) -> ParamVec<Self::RateParam>;

    /// Algebraic variable assignments for the ODE
    fn calc_assignments(
        &self,
        x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::AssignParam>;
    
    /// Rate variable assignments for the ODE
    fn calc_rates(
        &self,
        x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        assignments: &ParamVec<Self::AssignParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::RateParam>;
}

/// Construct for executing an explicit ODE
pub struct OdeRunner<T: Ode>
{
    ode: T,
    constants: ParamVec<T::ConstParam>,
    initial_rate_bound: ParamVec<T::RateParam>,
    assignment_history: RefCell<Vec<ParamVec<T::AssignParam>>>,
    t_end: RefCell<NumType>,
    step_size: RefCell<NumType>,
    prev_x: RefCell<NumType>,
}

impl<T: Ode> OdeRunner<T> {
    pub fn new(ode: T) -> Self {

        let constants = ode.constants();
        let initial_rate_bound = ode.initial_values(&constants);
        let initial_assignments = ode.calc_assignments(0.0, &constants, &initial_rate_bound);

        Self {
            ode: ode,
            constants: constants,
            initial_rate_bound,
            assignment_history: RefCell::new(vec![initial_assignments]),
            t_end: RefCell::new(0.0),
            step_size: RefCell::new(0.01),
            prev_x: RefCell::new(-1.0),
        }
    }

    /// 
    pub fn set_constant(&mut self, param: T::ConstParam, value: NumType) {
        self.constants[param] = value;
    }

    pub fn set_initial_value(&mut self, param: T::RateParam, value: NumType) {
        self.initial_rate_bound[param] = value;
    }

    pub fn set_initial_values(&mut self, results: ParamVec<T::RateParam>) {
        self.initial_rate_bound = results;
    }

    pub fn solve_fixed(
        &self,
        t_start: NumType,
        t_end: NumType,
        step_size: NumType,
        method: &impl runge_kutta::fixed::ExplicitRKMethod<NumType>
    ) -> OdeResults<T> {
        *self.t_end.borrow_mut() = t_end;
        *self.step_size.borrow_mut() = step_size;

        let problem = ExplicitInitialValueProblemBuilder::new(
            self,
            t_start,
            self.initial_rate_bound.clone().into(),
        )
        .t_end(t_end)
        .build();

        let solver = runge_kutta::fixed::FixedStepper::new(step_size);

        let (x, y) = solver.solve(&problem, method).unwrap();

        let last_assign = vec![self.assignment_history.borrow().last().unwrap().clone()];

        OdeResults {
            constants: self.constants.clone().into(),
            x_values: x,
            assignment_results: self.assignment_history
                .replace(last_assign),
            rate_bound_results: y.into_iter()
                .map(|v| v.into())
                .collect(),
        }
    }
}

impl<T: Ode> ExplicitODE<NumType> for OdeRunner<T>
{
    fn ode(&self, x: &NumType, y: &Vector<NumType>) -> Vector<NumType> {
        let y_params: ParamVec<T::RateParam> = y.clone().into();
        let assignments = self.ode.calc_assignments(*x, &self.constants, &y_params);
        let rates = self.ode.calc_rates(*x, &self.constants, &assignments, &y_params);

        // this function will often be called between step sizes depending on the
        // method used. This ensures assignment_history aligns with the step sizes
        // which will appear in the output
        let mut prev_x = self.prev_x.borrow_mut();
        let t_end = *self.t_end.borrow();
        let step_size = *self.step_size.borrow();
        if *x == t_end || *x - *prev_x >= step_size - step_size*0.01 {
            self.assignment_history.borrow_mut().push(assignments);
        }

        *prev_x = *x;

        rates.into()
    }
}
