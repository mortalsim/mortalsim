extern crate mortalsim_macros;

use mortalsim_macros::{AssignmentParamEnum, ConstantParamEnum, RateBoundParamEnum};
use mortalsim_math_routines::{
    ode::Ode,
    params::ParamVec
};


/// Implementation of the Van der pol equation using the ode math routines: y′′1 − μ(1−y1^2)y′1 + y1 = 0
/// Rewritten as a system of first-order ODEs, this becomes:
/// y′1 = y2
/// y'2 = μ(1 − y1^2)y2 − y1.


#[derive(Clone, Copy, ConstantParamEnum)]
enum VdpConstantParam {
    Mu,
}

#[derive(Clone, Copy, AssignmentParamEnum)]
enum VdpAssignmentParam {
    P1,
    P2,
}

#[derive(Clone, Copy, RateBoundParamEnum)]
enum VdpRateBoundParam {
    Y1,
    Y2,
}

struct VdpOde {}

impl Ode for VdpOde {
    type Constant = VdpConstantParam;
    type Assignment = VdpAssignmentParam;
    type RateBound = VdpRateBoundParam;

    fn constants() -> ParamVec<Self::Constant> {
        let mut c = ParamVec::new();
        c[VdpConstantParam::Mu] = 1.0;
        c
    }

    fn initial_values(
        _constants: &ParamVec<Self::Constant>,
    ) -> ParamVec<Self::RateBound> {
        let mut iv = ParamVec::new();
        iv[VdpRateBoundParam::Y1] = 2.0;
        iv[VdpRateBoundParam::Y2] = 0.0;
        iv
    }

    fn calc_assignments(
        _x: f64,
        constants: &ParamVec<Self::Constant>,
        ode_vars: &ParamVec<Self::RateBound>,
    ) -> ParamVec<Self::Assignment> {
        let mu = constants[VdpConstantParam::Mu];
        let y1 = ode_vars[VdpRateBoundParam::Y1];
        let y2 = ode_vars[VdpRateBoundParam::Y1];

        let p1 = 1.0 - f64::powf(y1, 2.0);
        let p2 = mu * p1 * y2 - y1;

        let mut a = ParamVec::new();
        a[VdpAssignmentParam::P1] = p1;
        a[VdpAssignmentParam::P2] = p2;
        a
    }

    fn calc_rates(
        _x: f64,
        _constants: &ParamVec<Self::Constant>,
        assignments: &ParamVec<Self::Assignment>,
        ode_vars: &ParamVec<Self::RateBound>,
    ) -> ParamVec<Self::RateBound> {
        let p2 = assignments[VdpAssignmentParam::P2];

        let y2 = ode_vars[VdpRateBoundParam::Y2];

        let dy1_dt = y2;
        let dy2_dt = p2;

        let mut dy_dt = ParamVec::new();
        dy_dt[VdpRateBoundParam::Y1] = dy1_dt;
        dy_dt[VdpRateBoundParam::Y2] = dy2_dt;
        dy_dt
    }
}
