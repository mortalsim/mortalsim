use std::cell::RefCell;

use mathru::{algebra::linear::vector::Vector, analysis::differential_equation::ordinary::{solver::explicit::runge_kutta::fixed::{ExplicitRKMethod, FixedStepper}, ExplicitInitialValueProblemBuilder, ExplicitODE}};

use crate::params::{ConstantParam, AssignmentParam, RateBoundParam, ParamVec};

type NumType = f64;

pub struct OdeResults<T: Ode> {
    pub constants: ParamVec<T::ConstParam>,
    pub x_values: Vec<NumType>,
    pub assignment_results: Vec<ParamVec<T::AssignParam>>,
    pub rate_bound_results: Vec<ParamVec<T::RateParam>>,
}

impl<T: Ode> OdeResults<T> {
    pub fn len(&self) -> usize {
        self.x_values.len()
    }

    pub fn x(&self, index: usize) -> NumType{
        self.x_values[index]
    }

    pub fn constant_value(&self, param: T::ConstParam) -> NumType {
        self.constants[param]
    }

    pub fn assignment_value(&self, index: usize, param: T::AssignParam) -> NumType {
        self.assignment_results[index][param]
    }

    pub fn assignment_value_at_x(&self, x: NumType, param: T::AssignParam) -> NumType {
        let (index, _) = self.x_values
            .iter()
            .enumerate()
            .min_by(|(_i1, v1), (_i2, v2)| (*v1 - x).abs().partial_cmp(&(*v2 - x).abs()).unwrap())
            .unwrap();
        self.assignment_value(index, param)
    }

    pub fn rate_bound_value(&self, index: usize, param: T::RateParam) -> NumType {
        self.rate_bound_results[index][param]
    }

    pub fn rate_bound_value_at_x(&self, x: NumType, param: T::RateParam) -> NumType {
        let (index, _) = self.x_values
            .iter()
            .enumerate()
            .min_by(|(_i1, v1), (_i2, v2)| (*v1 - x).abs().partial_cmp(&(*v2 - x).abs()).unwrap())
            .unwrap();
        self.rate_bound_value(index, param)
    }
}

pub trait Ode
{
    type ConstParam: ConstantParam;
    type AssignParam: AssignmentParam;
    type RateParam: RateBoundParam;
    fn constants(&self) -> ParamVec<Self::ConstParam>;
    fn initial_values(
        &self,
        constants: &ParamVec<Self::ConstParam>,
    ) -> ParamVec<Self::RateParam>;
    fn calc_assignments(
        &self,
        x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::AssignParam>;
    fn calc_rates(
        &self,
        x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        assignments: &ParamVec<Self::AssignParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::RateParam>;
}

pub struct OdeRunner<T: Ode>
{
    ode: T,
    constants: ParamVec<T::ConstParam>,
    initial_rate_bound: ParamVec<T::RateParam>,
    assignment_history: RefCell<Vec<ParamVec<T::AssignParam>>>,
    t_end: NumType,
    step_size: NumType,
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
            t_end: 0.0,
            step_size: 0.01,
            prev_x: RefCell::new(-1.0),
        }
    }

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
        &mut self,
        t_start: NumType,
        t_end: NumType,
        step_size: NumType,
        method: &impl ExplicitRKMethod<NumType>
    ) -> OdeResults<T> {
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
        if *x == self.t_end || *x - *self.prev_x.borrow() >= self.step_size - self.step_size*0.01 {
            self.assignment_history.borrow_mut().push(assignments);
        }
        rates.into()
    }
}
