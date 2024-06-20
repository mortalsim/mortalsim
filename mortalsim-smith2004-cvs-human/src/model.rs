#![allow(non_snake_case)]

use mortalsim_math_routines::{ode::{NumType, Ode}, params::ParamVec};

use crate::params::{Smith2004CvsAssignmentParam, Smith2004CvsConstantParam, Smith2004CvsRateBoundParam};

pub struct Smith2004CvsOde {}

impl Smith2004CvsOde {
    pub fn new() -> Self {
        Self {}
    }
}

impl Ode for Smith2004CvsOde {
    type ConstParam = Smith2004CvsConstantParam;
    type AssignParam = Smith2004CvsAssignmentParam;
    type RateParam = Smith2004CvsRateBoundParam;

    fn constants(&self) -> ParamVec<Self::ConstParam> {
        let mut default_constants = ParamVec::<Self::ConstParam>::new();

        default_constants[Self::ConstParam::R_mt] = 0.0158; // mmHg_second_per_mL
        default_constants[Self::ConstParam::R_av] = 0.018; // mmHg_second_per_mL
        default_constants[Self::ConstParam::R_tc] = 0.0237; // mmHg_second_per_mL
        default_constants[Self::ConstParam::R_pv] = 0.0015; // mmHg_second_per_mL
        default_constants[Self::ConstParam::R_pul] = 0.0952; // mmHg_second_per_mL
        default_constants[Self::ConstParam::R_sys] = 1.3889; // mmHg_second_per_mL
        default_constants[Self::ConstParam::L_tc] = 8.0093e-05; // mmHg_second2_per_mL
        default_constants[Self::ConstParam::L_pv] = 0.00014868; // mmHg_second2_per_mL
        default_constants[Self::ConstParam::L_mt] = 7.6968e-05; // mmHg_second2_per_mL
        default_constants[Self::ConstParam::L_av] = 0.00012189; // mmHg_second2_per_mL
        default_constants[Self::ConstParam::P_th] = -4.0; // mmHg
        default_constants[Self::ConstParam::A] = 1.0; // dimensionless
        default_constants[Self::ConstParam::B] = 80.0; // per_second2
        default_constants[Self::ConstParam::C] = 0.27; // s
        default_constants[Self::ConstParam::period] = 1.0; // s
        default_constants[Self::ConstParam::P_0_pcd] = 0.5003; // mmHg
        default_constants[Self::ConstParam::V_0_pcd] = 200.0; // mL
        default_constants[Self::ConstParam::lambda_pcd] = 0.03; // per_mL
        default_constants[Self::ConstParam::E_es_lvf] = 2.8798; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_lvf] = 0.0; // mL
        default_constants[Self::ConstParam::P_0_lvf] = 0.1203; // mmHg
        default_constants[Self::ConstParam::lambda_lvf] = 0.033; // per_mL
        default_constants[Self::ConstParam::V_0_lvf] = 0.0; // mL
        default_constants[Self::ConstParam::E_es_rvf] = 0.585; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_rvf] = 0.0; // mL
        default_constants[Self::ConstParam::P_0_rvf] = 0.2157; // mmHg
        default_constants[Self::ConstParam::lambda_rvf] = 0.023; // per_mL
        default_constants[Self::ConstParam::V_0_rvf] = 0.0; // mL
        default_constants[Self::ConstParam::V_spt] = 1.0; // mL
        default_constants[Self::ConstParam::E_es_pa] = 0.369; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_pa] = 0.0; // mL
        default_constants[Self::ConstParam::E_es_pu] = 0.0073; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_pu] = 0.0; // mL
        default_constants[Self::ConstParam::E_es_ao] = 0.6713; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_ao] = 0.0; // mL
        default_constants[Self::ConstParam::E_es_vc] = 0.0059; // mmHg_per_mL
        default_constants[Self::ConstParam::V_d_vc] = 0.0; // mL

        default_constants
    }

    fn initial_values(
        &self,
        _constants: &ParamVec<Self::ConstParam>,
    ) -> ParamVec<Self::RateParam> {
        let mut initial_vars = ParamVec::<Self::RateParam>::new();

        initial_vars[Self::RateParam::V_lv] = 94.6812; // mL
        initial_vars[Self::RateParam::V_rv] = 90.7302; // mL
        initial_vars[Self::RateParam::V_pa] = 43.0123; // mL
        initial_vars[Self::RateParam::V_pu] = 808.458; // mL
        initial_vars[Self::RateParam::V_ao] = 133.338; // mL
        initial_vars[Self::RateParam::V_vc] = 329.78; // mL
        initial_vars[Self::RateParam::Q_mt] = 245.581; // mL_per_second
        initial_vars[Self::RateParam::Q_av] = 0.0; // mL_per_second
        initial_vars[Self::RateParam::Q_tc] = 190.066; // mL_per_second
        initial_vars[Self::RateParam::Q_pv] = 0.0; // mL_per_second

        initial_vars
    }

    fn calc_assignments(
        &self,
        x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::AssignParam> {
        // setup constants
        let R_pul: f64 = constants[Self::ConstParam::R_pul];
        let R_sys: f64 = constants[Self::ConstParam::R_sys];
        let P_th: f64 = constants[Self::ConstParam::P_th];
        let A: f64 = constants[Self::ConstParam::A];
        let B: f64 = constants[Self::ConstParam::B];
        let C: f64 = constants[Self::ConstParam::C];
        let period: f64 = constants[Self::ConstParam::period];
        let P_0_pcd: f64 = constants[Self::ConstParam::P_0_pcd];
        let V_0_pcd: f64 = constants[Self::ConstParam::V_0_pcd];
        let lambda_pcd: f64 = constants[Self::ConstParam::lambda_pcd];
        let E_es_lvf: f64 = constants[Self::ConstParam::E_es_lvf];
        let V_d_lvf: f64 = constants[Self::ConstParam::V_d_lvf];
        let P_0_lvf: f64 = constants[Self::ConstParam::P_0_lvf];
        let lambda_lvf: f64 = constants[Self::ConstParam::lambda_lvf];
        let V_0_lvf: f64 = constants[Self::ConstParam::V_0_lvf];
        let E_es_rvf: f64 = constants[Self::ConstParam::E_es_rvf];
        let V_d_rvf: f64 = constants[Self::ConstParam::V_d_rvf];
        let P_0_rvf: f64 = constants[Self::ConstParam::P_0_rvf];
        let lambda_rvf: f64 = constants[Self::ConstParam::lambda_rvf];
        let V_0_rvf: f64 = constants[Self::ConstParam::V_0_rvf];
        let V_spt: f64 = constants[Self::ConstParam::V_spt];
        let E_es_pa: f64 = constants[Self::ConstParam::E_es_pa];
        let V_d_pa: f64 = constants[Self::ConstParam::V_d_pa];
        let E_es_pu: f64 = constants[Self::ConstParam::E_es_pu];
        let V_d_pu: f64 = constants[Self::ConstParam::V_d_pu];
        let E_es_ao: f64 = constants[Self::ConstParam::E_es_ao];
        let V_d_ao: f64 = constants[Self::ConstParam::V_d_ao];
        let E_es_vc: f64 = constants[Self::ConstParam::E_es_vc];
        let V_d_vc: f64 = constants[Self::ConstParam::V_d_vc];

        // Setup rate-bound variables from the previous step
        let V_lv = ode_vars[Self::RateParam::V_lv];
        let V_rv = ode_vars[Self::RateParam::V_rv];
        let V_pa = ode_vars[Self::RateParam::V_pa];
        let V_pu = ode_vars[Self::RateParam::V_pu];
        let V_ao = ode_vars[Self::RateParam::V_ao];
        let V_vc = ode_vars[Self::RateParam::V_vc];
        
        // Assign algebraic variables
        let tau = x % period;
        let e_t = A * f64::exp(-(B / f64::powf(period, 2.0)) * f64::powf(tau - C * period, 2.0));
        let V_pcd = V_lv + V_rv;
        let P_pcd = P_0_pcd * (f64::exp(lambda_pcd * (V_pcd - V_0_pcd)) - 1.0);
        let P_peri = P_pcd + P_th;
        let V_lvf = V_lv - V_spt;
        let P_ed_lvf = P_0_lvf * (f64::exp(lambda_lvf * (V_lvf - V_0_lvf)) - 1.0);
        let P_es_lvf = E_es_lvf * (V_lvf - V_d_lvf);
        let P_lvf = e_t * P_es_lvf + (1.0 - e_t) * P_ed_lvf;
        let P_lv = P_lvf + P_peri;
        let V_rvf = V_rv + V_spt;
        let P_ed_rvf = P_0_rvf * (f64::exp(lambda_rvf * (V_rvf - V_0_rvf)) - 1.0);
        let P_es_rvf = E_es_rvf * (V_rvf - V_d_rvf);
        let P_rvf = e_t * P_es_rvf + (1.0 - e_t) * P_ed_rvf;
        let P_rv = P_rvf + P_peri;
        let P_pa = E_es_pa * (V_pa - V_d_pa) + P_th;
        let P_pu = E_es_pu * (V_pu - V_d_pu) + P_th;
        let P_ao = E_es_ao * (V_ao - V_d_ao);
        let P_vc = E_es_vc * (V_vc - V_d_vc);
        let Q_sys = (P_ao - P_vc) / R_sys;
        let Q_pul = (P_pa - P_pu) / R_pul;
        
        let mut assign_vars = ParamVec::<Self::AssignParam>::new();

       assign_vars[Self::AssignParam::tau] = tau;
       assign_vars[Self::AssignParam::e_t] = e_t;
       assign_vars[Self::AssignParam::V_pcd] = V_pcd;
       assign_vars[Self::AssignParam::P_pcd] = P_pcd;
       assign_vars[Self::AssignParam::P_peri] = P_peri;
       assign_vars[Self::AssignParam::V_lvf] = V_lvf;
       assign_vars[Self::AssignParam::P_ed_lvf] = P_ed_lvf;
       assign_vars[Self::AssignParam::P_es_lvf] = P_es_lvf;
       assign_vars[Self::AssignParam::P_lvf] = P_lvf;
       assign_vars[Self::AssignParam::P_lv] = P_lv;
       assign_vars[Self::AssignParam::V_rvf] = V_rvf;
       assign_vars[Self::AssignParam::P_ed_rvf] = P_ed_rvf;
       assign_vars[Self::AssignParam::P_es_rvf] = P_es_rvf;
       assign_vars[Self::AssignParam::P_rvf] = P_rvf;
       assign_vars[Self::AssignParam::P_rv] = P_rv;
       assign_vars[Self::AssignParam::P_pa] = P_pa;
       assign_vars[Self::AssignParam::P_pu] = P_pu;
       assign_vars[Self::AssignParam::P_ao] = P_ao;
       assign_vars[Self::AssignParam::P_vc] = P_vc;
       assign_vars[Self::AssignParam::Q_sys] = Q_sys;
       assign_vars[Self::AssignParam::Q_pul] = Q_pul;

       assign_vars

    }

    fn calc_rates(
        &self,
        _x: NumType,
        constants: &ParamVec<Self::ConstParam>,
        assignments: &ParamVec<Self::AssignParam>,
        ode_vars: &ParamVec<Self::RateParam>,
    ) -> ParamVec<Self::RateParam> {
        // setup constants
        let R_mt: f64 = constants[Self::ConstParam::R_mt];
        let R_av: f64 = constants[Self::ConstParam::R_av];
        let R_tc: f64 = constants[Self::ConstParam::R_tc];
        let R_pv: f64 = constants[Self::ConstParam::R_pv];
        let L_tc: f64 = constants[Self::ConstParam::L_tc];
        let L_pv: f64 = constants[Self::ConstParam::L_pv];
        let L_mt: f64 = constants[Self::ConstParam::L_mt];
        let L_av: f64 = constants[Self::ConstParam::L_av];

        // Setup assigned variables
        let P_lv = assignments[Self::AssignParam::P_lv];
        let P_rv = assignments[Self::AssignParam::P_rv];
        let P_pa = assignments[Self::AssignParam::P_pa];
        let P_pu = assignments[Self::AssignParam::P_pu];
        let P_ao = assignments[Self::AssignParam::P_ao];
        let P_vc = assignments[Self::AssignParam::P_vc];
        let Q_sys = assignments[Self::AssignParam::Q_sys];
        let Q_pul = assignments[Self::AssignParam::Q_pul];

        // Setup rate-bound variables from the previous step
        let Q_mt = ode_vars[Self::RateParam::Q_mt];
        let Q_av = ode_vars[Self::RateParam::Q_av];
        let Q_tc = ode_vars[Self::RateParam::Q_tc];
        let Q_pv = ode_vars[Self::RateParam::Q_pv];

        // Calculate rates
        let rate_V_lv = {
            if Q_mt < 0.0 && Q_av < 0.0 { 0.0 }
            else if Q_mt < 0.0 { -Q_av }
            else if Q_av < 0.0 { Q_mt }
            else { Q_mt - Q_av }
        };
        let rate_V_rv = {
            if Q_tc < 0.0 && Q_pv < 0.0 { 0.0 }
            else if Q_tc < 0.0 { -Q_pv }
            else if Q_pv < 0.0 { Q_tc }
            else { Q_tc - Q_pv }
        };
        let rate_V_pa = {
            if Q_pv < 0.0 { -Q_pul }
            else { Q_pv - Q_pul }
        };
        let rate_V_pu = {
            if Q_mt < 0.0 { Q_pul }
            else { Q_pul - Q_mt }
        };
        let rate_V_ao = {
            if Q_av < 0.0 { -Q_sys }
            else { Q_av - Q_sys }
        };
        let rate_V_vc = {
            if Q_tc < 0.0 { Q_sys }
            else { Q_sys - Q_tc }
        };
        let rate_Q_mt = {
            if P_pu - P_lv < 0.0 && Q_mt < 0.0 { 0.0 }
            else { (P_pu - P_lv - Q_mt * R_mt) / L_mt }
        };
        let rate_Q_av = {
            if P_lv - P_ao < 0.0 && Q_av < 0.0 { 0.0 }
            else { (P_lv - P_ao - Q_av * R_av) / L_av }
        };
        let rate_Q_tc = {
            if P_vc - P_rv < 0.0 && Q_tc < 0.0 { 0.0 }
            else { (P_vc - P_rv - Q_tc * R_tc) / L_tc }
        };
        let rate_Q_pv = {
            if P_rv - P_pa < 0.0 && Q_pv < 0.0 { 0.0 }
            else { (P_rv - P_pa - Q_pv * R_pv) / L_pv }
        };

        let mut rate_vars = ParamVec::<Self::RateParam>::new();

        rate_vars[Self::RateParam::V_lv] = rate_V_lv;
        rate_vars[Self::RateParam::V_rv] = rate_V_rv;
        rate_vars[Self::RateParam::V_pa] = rate_V_pa;
        rate_vars[Self::RateParam::V_pu] = rate_V_pu;
        rate_vars[Self::RateParam::V_ao] = rate_V_ao;
        rate_vars[Self::RateParam::V_vc] = rate_V_vc;
        rate_vars[Self::RateParam::Q_mt] = rate_Q_mt;
        rate_vars[Self::RateParam::Q_av] = rate_Q_av;
        rate_vars[Self::RateParam::Q_tc] = rate_Q_tc;
        rate_vars[Self::RateParam::Q_pv] = rate_Q_pv;

        rate_vars
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use mortalsim_math_routines::ode::{runge_kutta::fixed::RungeKutta4, OdeRunner};
    use plotters::{
        backend::BitMapBackend, chart::{ChartBuilder, LabelAreaPosition}, drawing::IntoDrawingArea, series::LineSeries, style::{full_palette::ORANGE, IntoFont, BLACK, BLUE, GREEN, RED, WHITE}
    };

    use Smith2004CvsAssignmentParam as AssignParam;
    use Smith2004CvsRateBoundParam as RateParam;

    #[test]
    fn it_works() {
        let x_start: f64 = 0.0;
        let x_end: f64 = 10.0;

        let runner = OdeRunner::new(Smith2004CvsOde::new());

        let results = runner.solve_fixed(x_start, x_end, 0.01, &RungeKutta4::default());

        // Create chart
        let mut graph_x1: Vec<(f64, f64)> = Vec::with_capacity(results.len());
        let mut graph_x2: Vec<(f64, f64)> = Vec::with_capacity(results.len());
        let mut graph_x3: Vec<(f64, f64)> = Vec::with_capacity(results.len());
        let mut graph_x4: Vec<(f64, f64)> = Vec::with_capacity(results.len());

        let mut ymax = 1.0;

        for i in 0..results.len() {
            let x_i = results.x(i);
            let x1 = results.assignment_value(i, AssignParam::P_lv);
            let x2 = results.assignment_value(i, AssignParam::P_rv);
            let x3 = results.assignment_value(i, AssignParam::P_ao);
            let x4 = results.assignment_value(i, AssignParam::P_pa);
            graph_x1.push((x_i, x1));
            graph_x2.push((x_i, x2));
            graph_x3.push((x_i, x3));
            graph_x4.push((x_i, x4));

            if x1 > ymax { ymax = x1; }
            if x2 > ymax { ymax = x2; }
            if x3 > ymax { ymax = x3; }
            if x4 > ymax { ymax = x4; }
        }

        let root_area =
            BitMapBackend::new("./target/pressures.png", (1200, 800)).into_drawing_area();
        root_area.fill(&WHITE).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .margin(20)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Smith2004 ODE Simulation", ("Arial", 40))
            .build_cartesian_2d(x_start..x_end, -1.0f64..ymax)
            .unwrap();

        ctx.configure_mesh()
            .x_desc("Time t")
            .axis_desc_style(("sans-serif", 25).into_font())
            .draw()
            .unwrap();

        ctx.draw_series(LineSeries::new(graph_x1, &BLUE)).unwrap();

        ctx.draw_series(LineSeries::new(graph_x2, &RED)).unwrap();

        ctx.draw_series(LineSeries::new(graph_x3, &GREEN)).unwrap();
        
        ctx.draw_series(LineSeries::new(graph_x4, &ORANGE)).unwrap();


        // Create chart
        let mut graph_pv: Vec<(f64, f64)> = Vec::with_capacity(results.len());

        let mut xmax = 1.0;
        let mut ymax = 1.0;

        for i in 0..results.len() {
            let x_i = results.rate_bound_value(i, RateParam::V_lv);
            let y = results.assignment_value(i, AssignParam::P_lv);
            graph_pv.push((x_i, y));

            if x_i > xmax { xmax = x_i; }
            if y > ymax { ymax = y; }
        }

        // let mut output_file = std::env::current_dir().unwrap();
        // output_file.push("figures");
        // output_file.push("ode_smith2004_cvs_human.png");
    
        let root_area =
            BitMapBackend::new("./target/pv.png", (1200, 800)).into_drawing_area();
        root_area.fill(&WHITE).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .margin(20)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Smith2004 ODE Simulation", ("Arial", 40))
            .build_cartesian_2d(00.0..(xmax + 1.0), -5.0f64..(ymax + 1.0))
            .unwrap();

        ctx.configure_mesh()
            .x_desc("Time t")
            .axis_desc_style(("sans-serif", 25).into_font())
            .draw()
            .unwrap();

        ctx.draw_series(LineSeries::new(graph_pv, &BLACK)).unwrap();


        let varnames = vec![
            "time",
            "e_t",
            "tau",
            "V_pcd",
            "P_pcd",
            "P_peri",
            "V_lvf",
            "P_lvf",
            "P_lv",
            "P_es_lvf",
            "P_ed_lvf",
            "V_rvf",
            "P_rvf",
            "P_rv",
            "P_es_rvf",
            "P_ed_rvf",
            "P_pa",
            "P_pu",
            "P_ao",
            "P_vc",
            "Q_sys",
            "Q_pul",
            "V_lv",
            "V_rv",
            "V_pa",
            "V_pu",
            "V_ao",
            "V_vc",
            "Q_mt",
            "Q_av",
            "Q_tc",
            "Q_pv",
        ];

        let csv_filename = "./target/cvs_human.csv";
        let mut first_line = varnames.join(",");
        first_line.push_str("\n");
        std::fs::write(csv_filename, first_line).expect("Unable to write file");

        let mut csv_file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(csv_filename)
            .unwrap();

        let num_elems = 1 + results.assignment_results.len() + results.rate_bound_results.len();

        for i in 0..results.len() {
            let mut row = Vec::with_capacity(num_elems);
            row.push(format!("{:.2}", results.x(i)));
            row.extend(results.assignment_results[i].iter().map(|x| format!("{}", x)));
            row.extend(results.rate_bound_results[i].iter().map(|x| format!("{}", x)));
            if let Err(e) = writeln!(csv_file, "{}", row.join(",")) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
}
