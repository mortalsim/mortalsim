extern crate mortalsim_macros;

use mathru::analysis::differential_equation::ordinary::solver::explicit::runge_kutta::fixed::RungeKutta4;
use mortalsim_macros::{AssignmentParamEnum, ConstantParamEnum, RateBoundParamEnum};
use mortalsim_math_routines::{
    ode::{Ode, OdeRunner},
    params::ParamVec
};
use plotters::{backend::BitMapBackend, chart::{ChartBuilder, LabelAreaPosition, SeriesLabelPosition}, drawing::IntoDrawingArea, element::Rectangle, series::LineSeries, style::{full_palette::ORANGE, IntoFont, BLUE, GREEN, RED, WHITE}};


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

impl VdpOde {
    fn new() -> Self {
        Self {}
    }
}

impl Ode for VdpOde {
    type ConstParam = VdpConstantParam;
    type AssignParam = VdpAssignmentParam;
    type RateParam = VdpRateBoundParam;

    fn constants(&self) -> ParamVec<Self::ConstParam> {
        let mut c = ParamVec::new();
        c[VdpConstantParam::Mu] = 1.0;
        c
    }

    fn initial_values(
        &self,
        _constants: &ParamVec<Self::ConstParam>,
    ) -> ParamVec<Self::RateParam> {
        let mut iv = ParamVec::new();
        iv[VdpRateBoundParam::Y1] = 2.0;
        iv[VdpRateBoundParam::Y2] = 0.0;
        iv
    }

    fn calc_assignments(
        &self,
        _x: f64,
        constants: &ParamVec<Self::ConstParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::AssignParam> {
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
        &self,
        _x: f64,
        _constants: &ParamVec<Self::ConstParam>,
        assignments: &ParamVec<Self::AssignParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::RateParam> {
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

#[test]
fn solve() {
    let vdp = VdpOde::new();
    let mut runner = OdeRunner::new(vdp);

    let x_start = 0.0;
    let x_end = 30.0;

    let res = runner.solve_fixed(x_start, x_end, 0.01, &RungeKutta4::default());

    assert!(res.len() > 0);

    let mut graphs: Vec<Vec<(f64, f64)>> = vec![
        Vec::with_capacity(res.len()),
        Vec::with_capacity(res.len()),
        Vec::with_capacity(res.len()),
        Vec::with_capacity(res.len()),
    ];

    let mut ymax = 1.0;

    for i in 0..res.len() {
        let x_i = res.x(i);
        let p1 = res.assignment_results[i][VdpAssignmentParam::P1];
        let p2 = res.assignment_results[i][VdpAssignmentParam::P2];
        let y1 = res.rate_bound_results[i][VdpRateBoundParam::Y1];
        let y2 = res.rate_bound_results[i][VdpRateBoundParam::Y2];
        graphs[0].push((x_i, p1));
        graphs[1].push((x_i, p2));
        graphs[2].push((x_i, y1));
        graphs[3].push((x_i, y2));

        if p1 > ymax { ymax = p1; }
        if p2 > ymax { ymax = p2; }
        if y1 > ymax { ymax = y1; }
        if y2 > ymax { ymax = y2; }
    }

    let root_area =
        BitMapBackend::new("../target/debug/van_der_pal.png", (1200, 800)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_area)
        .margin(20)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Van der pal ODE Simulation", ("Arial", 40))
        .build_cartesian_2d(x_start..x_end, -ymax..ymax)
        .unwrap();

    ctx.configure_mesh()
        .x_desc("Time t")
        .axis_desc_style(("sans-serif", 25).into_font())
        .draw()
        .unwrap();

    let line_cfgs = vec![
        (BLUE, "P1"),
        (RED, "P2"),
        (GREEN, "Y1"),
        (ORANGE, "Y2"),
    ];

    for (idx, g) in graphs.into_iter().enumerate() {
        let (color, label) = line_cfgs[idx];
        ctx.draw_series(LineSeries::new(g, color))
            .unwrap()
            .label(label)
            .legend(move |(x,y)| Rectangle::new([(x - 15, y + 1), (x, y)], color));
    }

    ctx.configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(20)
        .legend_area_size(5)
        .border_style(BLUE)
        .background_style(WHITE)
        .label_font(("Calibri", 20))
        .draw()
        .unwrap();

    assert!((-3.1..-2.9).contains(&res.assignment_value_at_x(0.0, VdpAssignmentParam::P1)));
    assert!((0.9..1.1).contains(&res.assignment_value_at_x(3.7, VdpAssignmentParam::P1)));
    
    assert!((-8.1..-7.9).contains(&res.assignment_value_at_x(0.0, VdpAssignmentParam::P2)));
    assert!((-0.1..0.1).contains(&res.assignment_value_at_x(3.7, VdpAssignmentParam::P2)));

    assert!((1.9..2.1).contains(&res.rate_bound_value_at_x(0.0, VdpRateBoundParam::Y1)));
    assert!((-2.1..-1.9).contains(&res.rate_bound_value_at_x(1.85, VdpRateBoundParam::Y1)));
    assert!((1.9..2.1).contains(&res.rate_bound_value_at_x(3.7, VdpRateBoundParam::Y1)));

    assert!((-0.1..0.1).contains(&res.rate_bound_value_at_x(0.0, VdpRateBoundParam::Y2)));
    assert!((2.8..3.0).contains(&res.rate_bound_value_at_x(2.75, VdpRateBoundParam::Y2)));
}
