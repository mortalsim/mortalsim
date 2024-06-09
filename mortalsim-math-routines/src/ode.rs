use std::cell::RefCell;

use mathru::{algebra::linear::vector::Vector, analysis::differential_equation::ordinary::{solver::explicit::runge_kutta::fixed::{ExplicitRKMethod, FixedStepper}, ExplicitInitialValueProblemBuilder, ExplicitODE}};

use crate::params::{ConstantParam, AssignmentParam, RateBoundParam, ParamVec};

type NumType = f64;

pub struct OdeResults {
    constants: Vector<NumType>,
    x_values: Vec<NumType>,
    assignment_results: Vec<Vector<NumType>>,
    rate_bound_results: Vec<Vector<NumType>>,
}

impl OdeResults {
    pub fn len(&self) -> usize {
        self.x_values.len()
    }

    pub fn time(&self, index: usize) -> NumType{
        self.x_values[index]
    }

    pub fn constant_value(&self, param: impl ConstantParam) -> NumType {
        self.constants[param.into()]
    }

    pub fn assignment_value(&self, index: usize, param: impl AssignmentParam) -> NumType {
        self.assignment_results[index][param.into()]
    }

    pub fn rate_bound_value(&self, index: usize, param: impl RateBoundParam) -> NumType {
        self.rate_bound_results[index][param.into()]
    }
}

pub trait Ode
{
    type Constant: ConstantParam;
    type Assignment: AssignmentParam;
    type RateBound: RateBoundParam;
    fn constants() -> ParamVec<Self::Constant>;
    fn initial_values(
        constants: &ParamVec<Self::Constant>,
    ) -> ParamVec<Self::RateBound>;
    fn calc_assignments(
        x: NumType,
        constants: &ParamVec<Self::Constant>,
        ode_vars: &ParamVec<Self::RateBound>,
    ) -> ParamVec<Self::Assignment>;
    fn calc_rates(
        x: NumType,
        constants: &ParamVec<Self::Constant>,
        assignments: &ParamVec<Self::Assignment>,
        ode_vars: &ParamVec<Self::RateBound>,
    ) -> ParamVec<Self::RateBound>;
}

pub struct OdeRunner<T: Ode>
{
    constants: ParamVec<T::Constant>,
    initial_rate_bound: ParamVec<T::RateBound>,
    assignment_history: RefCell<Vec<ParamVec<T::Assignment>>>,
    t_end: NumType,
    step_size: NumType,
    prev_x: RefCell<NumType>,
}

impl<T: Ode> OdeRunner<T> {
    pub fn new() -> Self {

        let constants = T::constants();
        let initial_rate_bound = T::initial_values(&constants);
        let initial_assignments = T::calc_assignments(0.0, &constants, &initial_rate_bound);

        Self {
            constants: constants,
            initial_rate_bound,
            assignment_history: RefCell::new(vec![initial_assignments]),
            t_end: 0.0,
            step_size: 0.01,
            prev_x: RefCell::new(-1.0),
        }
    }

    pub fn set_constant(&mut self, param: T::Constant, value: NumType) {
        self.constants[param] = value;
    }

    pub fn solve_fixed(
        &mut self,
        t_start: NumType,
        t_end: NumType,
        step_size: NumType,
        method: &impl ExplicitRKMethod<NumType>
    ) -> OdeResults {
        self.t_end = t_end;
        self.step_size = step_size;

        let problem = ExplicitInitialValueProblemBuilder::new(
            self,
            t_start,
            self.initial_rate_bound.clone().into(),
        )
        .t_end(t_end)
        .build();

        let solver = FixedStepper::new(self.step_size);

        let (x, y) = solver.solve(&problem, method).unwrap();

        OdeResults {
            constants: self.constants.clone().into(),
            x_values: x,
            assignment_results: self.assignment_history
                .replace(vec![self.assignment_history.borrow().last().unwrap().clone()])
                .into_iter()
                .map(|v| v.into())
                .collect(),
            rate_bound_results: y,
        }
    }
}

impl<T: Ode> ExplicitODE<NumType> for OdeRunner<T>
{
    fn ode(&self, x: &NumType, y: &Vector<NumType>) -> Vector<NumType> {
        let y_params: ParamVec<T::RateBound> = y.clone().into();
        let assignments = T::calc_assignments(*x, &self.constants, &y_params);
        let rates = T::calc_rates(*x, &self.constants, &assignments, &y_params);
        if *x == self.t_end || *x - *self.prev_x.borrow() >= self.step_size - self.step_size*0.01 {
            self.assignment_history.borrow_mut().push(assignments);
        }
        rates.into()
    }
}
