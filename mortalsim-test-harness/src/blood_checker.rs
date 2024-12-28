use std::collections::{HashMap, HashSet, VecDeque};
use mortalsim_core::sim::Organism;
use rand::distributions::{Alphanumeric, DistString};
use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::sim::layer::circulation::{CirculationComponent, CirculationConnector};
use mortalsim_core::substance::{Substance, SubstanceChange, SubstanceConcentration};
use mortalsim_core::SimTime;

pub struct SubstanceConcentrationRange {
    min: SubstanceConcentration,
    max: SubstanceConcentration,
}

impl SubstanceConcentrationRange {
    #[allow(non_snake_case)]
    pub fn new(min_uM: f64, max_uM: f64) -> Self {
        Self {
            min: SubstanceConcentration::from_uM(min_uM),
            max: SubstanceConcentration::from_uM(max_uM),
        }
    }
    pub fn check(&self, val: SubstanceConcentration) {
        log::info!("Concentration {}. Expected range {} -> {}",
            val,
            self.min,
            self.max
        );

        assert!(
            val >= self.min && val <= self.max,
            "Concentration {} is not within expected range {} -> {}",
            val,
            self.min,
            self.max
        );
    }
}


pub struct BloodCheckerComponent<O: Organism> {
    /// Generated ID
    id: &'static str,
    /// Which vessel to associate with
    vessel: O::VesselType,
    /// List of time to execute, substance to change, how much
    pending_writes: VecDeque<(SimTime, Substance, SubstanceChange)>,
    /// List of time to read, substance to check, expected value
    pending_reads: VecDeque<(SimTime, Substance, SubstanceConcentrationRange)>,
    /// Circulation connector
    circ_connector: CirculationConnector<O>,
    /// Prev
    prev: HashMap<Substance, SubstanceConcentration>,
}

impl<O: Organism> BloodCheckerComponent<O> {
    pub fn new(
        vessel: O::VesselType,
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
            prev: HashMap::new(),
        }
    }
}

impl<O: Organism> CirculationComponent<O> for BloodCheckerComponent<O> {
    fn circulation_connector(&mut self) -> &mut CirculationConnector<O> {
        &mut self.circ_connector
    }

    fn circulation_init(&mut self, circulation_initializer: &mut mortalsim_core::sim::layer::circulation::CirculationInitializer<O>) {
        log::info!("Circ init {}", self.id);
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

impl<O: Organism> SimComponent<O> for BloodCheckerComponent<O> {
    fn id(&self) -> &'static str {
        self.id
    }

    fn attach(self, registry: &mut mortalsim_core::sim::component::ComponentRegistry<O>) {
        registry.add_circulation_component(self)
    }

    fn run(&mut self) {
        let sim_time = self.circ_connector.sim_time();

        // See if we need to write any substance changes
        while self.pending_writes.len() > 0 && self.pending_writes.get(0).unwrap().0 <= sim_time {
            let (_, substance, change) = self.pending_writes.pop_front().unwrap();
            log::info!("Scheduling change on {:?}", self.vessel);
            self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .schedule_custom_change(substance, change);
        }

        if let Some((_, substance, _ex)) = self.pending_reads.get(0) {
            let val = self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .concentration_of(&substance);
            if !self.prev.contains_key(substance) || (&val - self.prev.get(substance).unwrap()).molpm3 > 0.000001 {
                log::info!("{}: {} {:?}: {}", self.circ_connector.sim_time(), self.id(), self.vessel, val);
            }

            self.prev.insert(*substance, val);
        }

        while self.pending_reads.len() > 0 && self.pending_reads.get(0).unwrap().0 <= sim_time {
            let (_, substance, expected) = self.pending_reads.pop_front().unwrap();
            log::info!("{}: Reading value on {:?}", sim_time, self.vessel);
            let conc = self.circ_connector
                .blood_store(&self.vessel)
                .unwrap()
                .concentration_of(&substance);

            expected.check(conc);
        }
    }
}
