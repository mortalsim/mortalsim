
extern crate mortalsim_macros;

use model::Smith2004CvsOde;
use mortalsim_core::{
    event::{AorticBloodPressure, Event, HeartRate, PulmonaryBloodPressure},
    sim::{
        component::SimComponent,
        layer::core::{CoreComponent, CoreConnector}
    }, units::mechanical::Pressure, SimTimeSpan,
};
use mortalsim_human::HumanOrganism;
use mortalsim_math_routines::ode::{runge_kutta::fixed::RungeKutta4, OdeRunner};
use params::{Smith2004CvsAssignmentParam, Smith2004CvsConstantParam};

pub mod params;
pub mod model;

#[derive(Debug)]
pub struct Smith2004CvsParamChanges {
    changes: Vec<(Smith2004CvsConstantParam, f64)>,
}

impl Smith2004CvsParamChanges {
    pub fn new() -> Self {
        Self {
            changes: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            changes: Vec::with_capacity(capacity)
        }
    }

    pub fn set_param(&mut self, param: Smith2004CvsConstantParam, val: f64) {
        self.changes.push((param, val));
    }
}

impl Event for Smith2004CvsParamChanges {
    fn transient(&self) -> bool {
        true
    }
}

/// This is a Mortalsim component to simulate cardiovascular dynamics
/// 
/// Populates `AorticBloodPressure` and `PulmonaryBloodPressure` events
/// 
/// This component exhibits a model from the article:
/// 
/// <strong>Minimal haemodynamic system model including ventricular interaction and valve dynamics.</strong>
/// 
///   Smith BW, Chase JG, Nokes RI, Shaw GM, Wake G.      <em>Med Eng Phys</em>
/// 
///   2004 Mar;26(2):131-9      <a href="http://www.ncbi.nlm.nih.gov/pubmed/15036180">15036180</a>
/// 
/// <strong>Abstract:</strong>
/// </br>
///   Characterising circulatory dysfunction and choosing a suitable treatment is
/// often difficult and time consuming, and can result in a deterioration in patient
/// condition, or unsuitable therapy choices. A stable minimal model of the human
/// cardiovascular system (CVS) is developed with the ultimate specific aim of
/// assisting medical staff for rapid, on site modelling to assist in diagnosis and
/// treatment. Models found in the literature simulate specific areas of the CVS
/// with limited direct usefulness to medical staff. Others model the full CVS as a
/// closed loop system, but they were found to be very complex, difficult to solve,
/// or unstable. This paper develops a model that uses a minimal number of governing
/// equations with the primary goal of accurately capturing trends in the CVS
/// dynamics in a simple, easily solved, robust model. The model is shown to have
/// long term stability and consistency with non-specific initial conditions as a
/// result. An &quot;open on pressure close on flow&quot; valve law is created to capture the
/// effects of inertia and the resulting dynamics of blood flow through the cardiac
/// valves. An accurate, stable solution is performed using a method that varies the
/// number of states in the model depending on the specific phase of the cardiac
/// cycle, better matching the real physiological conditions. Examples of results
/// include a 9% drop in cardiac output when increasing the thoracic pressure from
/// -4 to 0 mmHg, and an increase in blood pressure from 120/80 to 165/130 mmHg when
/// the systemic resistance is doubled. These results show that the model adequately
/// provides appropriate magnitudes and trends that are in agreement with existing
/// data for a variety of physiologically verified test cases simulating human CVS
/// function.
pub struct Smith2004CvsComponent {
    runner: OdeRunner<Smith2004CvsOde>,
    connector: CoreConnector<HumanOrganism>,
    ao_init: AorticBloodPressure,
    pa_init: PulmonaryBloodPressure,
}

impl Smith2004CvsComponent {

    /// Instantiates a new component to run the Smith2004 CVS ODE
    /// with reasonable default values for aortic (120/80) and
    /// pulmonary (25/4) blood pressure.
    pub fn new() -> Self {
        Self {
            runner: OdeRunner::new(Smith2004CvsOde::new()),
            connector: CoreConnector::new(),
            ao_init: AorticBloodPressure {
                systolic: Pressure::from_mmHg(120.0),
                diastolic: Pressure::from_mmHg(80.0),
            },
            pa_init: PulmonaryBloodPressure {
                systolic: Pressure::from_mmHg(25.0),
                diastolic: Pressure::from_mmHg(4.0),
            },
        }
    }
    
    /// Instantiates a new component to run the Smith2004 CVS ODE
    /// with given defaults for aortic and pulmonary blood pressure
    pub fn new_init(ao_init: AorticBloodPressure, pa_init: PulmonaryBloodPressure) -> Self {
        Self {
            runner: OdeRunner::new(Smith2004CvsOde::new()),
            connector: CoreConnector::new(),
            ao_init,
            pa_init,
        }
    }

    /// Sets a constant value for the simulation before it executes
    pub fn set_constant(&mut self, param: Smith2004CvsConstantParam, value: f64) {
        self.runner.set_constant(param, value)
    }
}

impl CoreComponent<HumanOrganism> for Smith2004CvsComponent {
    fn core_connector(&mut self) -> &mut CoreConnector<HumanOrganism> {
        &mut self.connector
    }

    fn core_init(&mut self, initializer: &mut mortalsim_core::sim::layer::core::CoreInitializer<HumanOrganism>) {
        initializer.notify::<HeartRate>();
        initializer.notify::<Smith2004CvsParamChanges>();

        initializer.set_output(self.ao_init);
        initializer.set_output(self.pa_init);
    }
}

impl SimComponent<HumanOrganism> for Smith2004CvsComponent {
    fn id(&self) -> &'static str {
        "Smith2004CvsComponent"
    }

    fn attach(self, registry: &mut mortalsim_core::sim::component::ComponentRegistry<HumanOrganism>) {
        registry.add_core_component(self)
    }

    fn run(&mut self) {
        if let Some(hr) = self.connector.get::<HeartRate>() {
            self.runner.set_constant(Smith2004CvsConstantParam::period, 1.0/hr.as_ref().Hz);
        }

        if let Some(evt) = self.connector.get::<Smith2004CvsParamChanges>() {
            for (param, value) in evt.changes.iter() {
                self.runner.set_constant(*param, *value)
            }
        }

        let t_end = 10.0;
        let step_size = 0.01;

        let results = self.runner.solve_fixed(0.0, t_end, step_size, &RungeKutta4::default());

        let mut bp_ao = AorticBloodPressure {
            systolic: Pressure::from_mmHg(-10000.0),
            diastolic: Pressure::from_mmHg(10000.0),
        };

        let mut bp_pa = PulmonaryBloodPressure {
            systolic: Pressure::from_mmHg(-10000.0),
            diastolic: Pressure::from_mmHg(10000.0),
        };

        // Go to the halfway point, after giving some time
        // for the model to stabilize before pulling the
        // results
        let measure_start_idx = ((t_end/2.0)*step_size) as usize;

        for idx in measure_start_idx..results.len() {
            let bp_ao_x = results.assignment_value(idx, Smith2004CvsAssignmentParam::P_ao);
            let bp_pa_x = results.assignment_value(idx, Smith2004CvsAssignmentParam::P_pa);

            if bp_ao_x > bp_ao.systolic.to_mmHg() {
                bp_ao.systolic = Pressure::from_mmHg(bp_ao_x);
            }
            if bp_ao_x < bp_ao.diastolic.to_mmHg() {
                bp_ao.diastolic = Pressure::from_mmHg(bp_ao_x);
            }
            if bp_pa_x > bp_pa.systolic.to_mmHg() {
                bp_pa.systolic = Pressure::from_mmHg(bp_pa_x);
            }
            if bp_pa_x < bp_pa.diastolic.to_mmHg() {
                bp_pa.diastolic = Pressure::from_mmHg(bp_pa_x);
            }
        }

        let effect_time = SimTimeSpan::from_s(
            results.constant_value(Smith2004CvsConstantParam::period)*(t_end/2.0)
        );

        self.connector.schedule_event(effect_time, bp_ao);
        self.connector.schedule_event(effect_time, bp_pa);

    }
}

#[cfg(test)]
mod tests {
    use mortalsim_core::sim::component::SimComponent;

    use crate::Smith2004CvsComponent;

    #[test]
    fn component_run() {
        let mut comp = Smith2004CvsComponent::new();
        comp.run();
    }
}
