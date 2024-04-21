use std::collections::{HashSet, VecDeque};

use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::sim::layer::circulation::{CirculationComponent, CirculationConnector};
use mortalsim_core::sim::organism::test::{TestBloodVessel, TestOrganism};
use mortalsim_core::substance::{Substance, SubstanceChange, SubstanceConcentration};
use mortalsim_core::SimTime;
use rand::distributions::{Alphanumeric, DistString};

pub struct SubstanceConcentrationRange {
    min: SubstanceConcentration,
    max: SubstanceConcentration,
}

impl SubstanceConcentrationRange {
    #[allow(non_snake_case)]
    pub fn new(min_mM: f64, max_mM: f64) -> Self {
        Self {
            min: SubstanceConcentration::from_mM(min_mM),
            max: SubstanceConcentration::from_mM(max_mM),
        }
    }
    pub fn check(&self, val: SubstanceConcentration) {
        assert!(
            val >= self.min && val <= self.max,
            "Concentration {} is not within expected range {} -> {}",
            val,
            self.min,
            self.max
        );

        println!("Concentration {}. Expected range {} -> {}",
            val,
            self.min,
            self.max
        )
    }
}


pub struct TestBloodCheckerComponent {
    /// Generated ID
    id: &'static str,
    /// Which vessel to associate with
    vessel: TestBloodVessel,
    /// List of time to execute, substance to change, how much
    pending_writes: VecDeque<(SimTime, Substance, SubstanceChange)>,
    /// List of time to read, substance to check, expected value
    pending_reads: VecDeque<(SimTime, Substance, SubstanceConcentrationRange)>,
    /// Circulation connector
    circ_connector: CirculationConnector<TestOrganism>,
}

impl TestBloodCheckerComponent {
    pub fn new(
        vessel: TestBloodVessel,
        mut writes: Vec<(SimTime, Substance, SubstanceChange)>,
        mut reads: Vec<(SimTime, Substance, SubstanceConcentrationRange)>,
    ) -> Self {
        writes.sort_by(|a, b| a.0.cmp(&b.0));
        reads.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            id: Alphanumeric.sample_string(&mut rand::thread_rng(), 16).leak(),
            vessel,
            pending_writes: writes.into(),
            pending_reads: reads.into(),
            circ_connector: CirculationConnector::new(),
        }
    }
}

impl CirculationComponent<TestOrganism> for TestBloodCheckerComponent {
    fn circulation_connector(&mut self) -> &mut CirculationConnector<TestOrganism> {
        &mut self.circ_connector
    }

    fn circulation_init(&mut self, circulation_initializer: &mut mortalsim_core::sim::layer::circulation::CirculationInitializer<TestOrganism>) {
        println!("Circ init {}", self.id);
        circulation_initializer.attach_vessel(self.vessel);
        for substance in self.pending_reads.iter().map(|(_, s, _e)| *s).collect::<HashSet<Substance>>() {
            circulation_initializer.notify_composition_change(
                self.vessel,
                substance,
                SubstanceConcentration::from_mM(0.0)
            )
        }
    }
}

impl SimComponent<TestOrganism> for TestBloodCheckerComponent {
    fn id(&self) -> &'static str {
        self.id
    }

    fn attach(self, registry: &mut mortalsim_core::sim::component::ComponentRegistry<TestOrganism>) {
        registry.add_circulation_component(self)
    }

    fn run(&mut self) {
        let sim_time = self.circ_connector.sim_time();

        // See if we need to write any substance changes
        while self.pending_writes.len() > 0 && self.pending_writes.get(0).unwrap().0 <= sim_time {
            let (_, substance, change) = self.pending_writes.pop_front().unwrap();
            println!("Scheduling change on {:?}", self.vessel);
            self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .schedule_custom_change(substance, change);
        }

        while self.pending_reads.len() > 0 && self.pending_reads.get(0).unwrap().0 <= sim_time {
            let (_, substance, expected) = self.pending_reads.pop_front().unwrap();
            println!("Reading value on {:?}", self.vessel);
            let conc = self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .concentration_of(&substance);

            expected.check(conc);
        }
    }
}
