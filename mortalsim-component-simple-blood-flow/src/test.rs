use std::collections::{HashSet, VecDeque};

use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::sim::layer::circulation::{CirculationComponent, CirculationConnector};
use mortalsim_core::sim::organism::test::{TestBloodVessel, TestOrganism};
use mortalsim_core::substance::{Substance, SubstanceChange, SubstanceConcentration};
use mortalsim_core::SimTime;
use rand::distributions::{Alphanumeric, DistString};


struct TestBloodComponent {
    id: &'static str,
    vessel: TestBloodVessel,
    pending_writes: VecDeque<(SimTime, Substance, SubstanceChange)>,
    pending_reads: VecDeque<(SimTime, Substance)>,
    reads: Vec<(SimTime, Substance, SubstanceConcentration)>,
    circ_connector: CirculationConnector<TestOrganism>,
}

impl TestBloodComponent {
    fn new(
        vessel: TestBloodVessel,
        mut writes: Vec<(SimTime, Substance, SubstanceChange)>,
        mut reads: Vec<(SimTime, Substance)>,
    ) -> Self {
        writes.sort_by(|a, b| a.0.cmp(&b.0));
        reads.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            id: Alphanumeric.sample_string(&mut rand::thread_rng(), 16).leak(),
            vessel,
            pending_writes: writes.into(),
            pending_reads: reads.into(),
            reads: Vec::new(),
            circ_connector: CirculationConnector::new(),
        }
    }

    fn get_reads(&self) -> impl Iterator<Item = &(SimTime, Substance, SubstanceConcentration)> {
        self.reads.iter()
    }
}

impl CirculationComponent<TestOrganism> for TestBloodComponent {
    fn circulation_connector(&mut self) -> &mut CirculationConnector<TestOrganism> {
        &mut self.circ_connector
    }

    fn circulation_init(&mut self, circulation_initializer: &mut mortalsim_core::sim::layer::circulation::CirculationInitializer<TestOrganism>) {
        circulation_initializer.attach_vessel(self.vessel);
        for substance in self.pending_reads.iter().map(|(_, s)| *s).collect::<HashSet<Substance>>() {
            circulation_initializer.notify_composition_change(
                self.vessel,
                substance,
                SubstanceConcentration::from_mM(0.0)
            )
        }
    }
}

impl SimComponent<TestOrganism> for TestBloodComponent {
    fn id(&self) -> &'static str {
        self.id
    }

    fn attach(self, registry: &mut mortalsim_core::sim::component::ComponentRegistry<TestOrganism>) {
        registry.add_circulation_component(self)
    }

    fn run(&mut self) {
        let sim_time = self.circ_connector.sim_time();

        // See if we need to write any substance changes
        while self.pending_writes.len() > 0 && self.pending_writes.get(0).unwrap().0 < sim_time {
            let (_, substance, change) = self.pending_writes.pop_front().unwrap();
            self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .schedule_custom_change(substance, change);
        }

        while self.pending_reads.len() > 0 && self.pending_reads.get(0).unwrap().0 < sim_time {
            let (_, substance) = self.pending_reads.pop_front().unwrap();
            let conc = self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .concentration_of(&substance);

            self.reads.push((sim_time, substance, conc));
        }
    }
}
