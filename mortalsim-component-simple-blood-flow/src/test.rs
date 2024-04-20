use std::collections::HashSet;

use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::sim::layer::circulation::{CirculationComponent, CirculationConnector};
use mortalsim_core::sim::organism::test::{TestBloodVessel, TestOrganism};
use mortalsim_core::substance::{Substance, SubstanceChange, SubstanceConcentration};
use mortalsim_core::SimTime;
use rand::distributions::{Alphanumeric, DistString};


struct TestBloodComponent {
    id: &'static str,
    vessel: TestBloodVessel,
    pending_writes: Vec<(SimTime, Substance, SubstanceChange)>,
    pending_reads: Vec<(SimTime, Substance)>,
    reads: Vec<(SimTime, Substance, SubstanceConcentration)>,
    circ_connector: CirculationConnector<TestOrganism>,
}

impl TestBloodComponent {
    fn new(
        vessel: TestBloodVessel,
        mut writes: Vec<(SimTime, Substance, SubstanceChange)>,
        mut reads: Vec<(SimTime, Substance)>,
    ) -> Self {
        writes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        reads.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        Self {
            id: Alphanumeric.sample_string(&mut rand::thread_rng(), 16).leak(),
            vessel,
            pending_writes: writes,
            pending_reads: reads,
            reads: Vec::new(),
            circ_connector: CirculationConnector::new(),
        }
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
        
    }
}
