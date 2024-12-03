use std::io::Write;

use mortalsim_math_routines::ode::{Ode, OdeResults};
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::prelude::{BitMapBackend, IntoDrawingArea};
use plotters::series::LineSeries;
use plotters::style::full_palette::{ORANGE, PURPLE};
use plotters::style::{IntoFont, BLUE, GREEN, MAGENTA, RED, WHITE};

pub struct ChartOptions<T: Ode> {
    title: String,
    assign_params: Vec<T::AssignParam>,
    rate_params: Vec<T::RateParam>,
    x_desc: String,
    chart_x: u32,
    chart_y: u32,
    x_start: Option<f64>,
    x_end: Option<f64>,
    y_min: Option<f64>,
    y_max: Option<f64>,
}

impl<T: Ode> ChartOptions<T> {
    pub fn new(title: String, x_desc: String, assign_params: Vec<T::AssignParam>, rate_params: Vec<T::RateParam>) -> Self {
        Self {
            title,
            assign_params,
            rate_params,
            x_desc,
            chart_x: 1920,
            chart_y: 1080,
            x_start: None,
            x_end: None,
            y_min: None,
            y_max: None,
        }
    }

    pub fn x_start(&mut self, x_start: f64) {
        self.x_start = Some(x_start);
    }

    pub fn x_end(&mut self, x_end: f64) {
        self.x_end = Some(x_end);
    }

    pub fn y_min(&mut self, y_min: f64) {
        self.y_min = Some(y_min);
    }

    pub fn y_max(&mut self, y_max: f64) {
        self.y_max = Some(y_max);
    }
}

pub fn write_chart<T: Ode> (filepath: &str, chart_data: &OdeResults<T>, options: &ChartOptions<T>) {
    // Create chart
    let mut assign_graphs: Vec<Vec<(f64, f64)>> = Vec::new();
    for _ in 0..options.assign_params.len() {
        assign_graphs.push(Vec::with_capacity(chart_data.len()));
    }
    
    let mut rate_graphs: Vec<Vec<(f64, f64)>> = Vec::new();
    for _ in 0..options.rate_params.len() {
        rate_graphs.push(Vec::with_capacity(chart_data.len()));
    }

    let mut ymax = 1.0;
    let mut ymin = -1.0;

    for i in 0..chart_data.len() {
        let x_i = chart_data.x(i);

        for (p_i, p) in options.assign_params.iter().enumerate() {
            let val = chart_data.assignment_value(i, *p);
            assign_graphs[p_i].push((x_i, val));
            if val > ymax { ymax = val }
            if val < ymin { ymin = val }
        }

        for (p_i,p) in options.rate_params.iter().enumerate() {
            let val = chart_data.rate_bound_value(i, *p);
            rate_graphs[p_i].push((x_i, val));
            if val > ymax { ymax = val }
            if val < ymin { ymin = val }
        }
    }

    let root_area =
        BitMapBackend::new(filepath, (options.chart_x, options.chart_y)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let x_start = options.x_start.unwrap_or(chart_data.x(0));
    let x_end = options.x_end.unwrap_or(chart_data.x(chart_data.len() - 1));

    if options.y_min.is_some() {
        ymin = options.y_min.unwrap();
    }

    if options.y_max.is_some() {
        ymax = options.y_max.unwrap();
    }

    let mut ctx = ChartBuilder::on(&root_area)
        .margin(20)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption(&options.title, ("Arial", 40))
        .build_cartesian_2d(x_start..x_end, ymin..ymax)
        .unwrap();

    ctx.configure_mesh()
        .x_desc(&options.x_desc)
        .axis_desc_style(("sans-serif", 25).into_font())
        .draw()
        .unwrap();

    let line_colors = [
        BLUE,
        RED,
        GREEN,
        PURPLE,
        ORANGE,
        MAGENTA,
    ];

    let mut line_i = 0;

    for g in assign_graphs {
        ctx.draw_series(LineSeries::new(g, line_colors[line_i])).expect("Error writing series");
        line_i = (line_i + 1) % line_colors.len();
    }
    
    for g in rate_graphs {
        ctx.draw_series(LineSeries::new(g,line_colors[line_i])).expect("Error writing series");
        line_i = (line_i + 1) % line_colors.len();
    }
}

pub fn write_csv<'a, T: Ode> (filepath: &str, data: &OdeResults<T>, x_var_name: &str, assign_var_names: impl Iterator<Item = &'a str>, rate_var_names: impl Iterator<Item = &'a str>) {

    let mut first_line = String::new() + x_var_name;

    for v in assign_var_names.chain(rate_var_names) {
        first_line += ",";
        first_line += v;
    }

    first_line.push_str("\n");
    std::fs::write(filepath, first_line).expect("Unable to write file");

    let mut csv_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
        .unwrap();

    let num_elems = 1 + data.assignment_results.len() + data.rate_bound_results.len();

    for i in 0..data.len() {
        let mut row = Vec::with_capacity(num_elems);
        row.push(format!("{:.2}", data.x(i)));
        row.extend(data.assignment_results[i].iter().map(|x| format!("{}", x)));
        row.extend(data.rate_bound_results[i].iter().map(|x| format!("{}", x)));
        if let Err(e) = writeln!(csv_file, "{}", row.join(",")) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}