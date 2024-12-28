use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};


use mortalsim_core::sim::component::SimComponent;
use mortalsim_core::sim::layer::circulation::{BloodVessel, CirculationComponent, CirculationConnector};
use mortalsim_core::sim::layer::core::{CoreComponent, CoreConnector};
use mortalsim_core::sim::Organism;
use mortalsim_core::event::{AorticBloodPressure, HeartRate};
use mortalsim_core::units::base::Time;
use mortalsim_core::SimTimeSpan;

/// Mortalsim module for simple propagation of blood composition
/// through a closed circulation system.
/// 
/// Major assumptions:
/// - Blood composition is evenly mixed, meaning the concentration
///   at each vessel becomes equal to the average of all compositions
///   from each preceding vessel.
/// - Diffusion time across the vasculature is linearly proportional
///   to the number of vessels in the circulation tree
/// - Time required for blood to pass through each vessel in the tree
///   is equivalent
/// - Pulmonary circulation time is approximately 1/12 the maximum
///   systemic circulation time

struct VesselDistanceCache<T> {
    map: HashMap<TypeId, HashMap<(&'static str, &'static str), Vec<(T, f64)>>>,
}

impl<T> VesselDistanceCache<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn get(&self, organism_type: &TypeId, a: &'static str, b: &'static str) -> Option<&Vec<(T, f64)>> {
        self.map.get(organism_type)?.get(&(a, b))
    }
    pub fn insert(&mut self, organism_type: TypeId, a: &'static str, b: &'static str, val: Vec<(T, f64)>) {
        self.map.entry(organism_type).or_default().insert((a, b), val);
    }
}

static DIST_CACHE: OnceLock<Arc<RwLock<VesselDistanceCache<u32>>>> = OnceLock::new();

pub struct SimpleBloodFlow<O: Organism> {
    base_heart_rate: HeartRate,
    base_diffusion_time: Time<f64>,
    core_connector: CoreConnector<O>,
    circ_connector: CirculationConnector<O>,
}


impl<O: Organism> SimpleBloodFlow<O> {
    pub const PULMONARY_RATIO: u32 = 12; // 1/12 of the max systemic length

    pub fn new(base_heart_rate: HeartRate, base_diffusion_time: Time<f64>) -> Self {
        Self {
            base_heart_rate,
            base_diffusion_time,
            core_connector: CoreConnector::new(),
            circ_connector: CirculationConnector::new(),
        }
    }

    pub fn factory(base_heart_rate: HeartRate, base_diffusion_time: Time<f64>) -> impl Fn() -> Self {
        return move || {
            Self::new(base_heart_rate, base_diffusion_time)
        }
    }

    fn calculate_blood_delays(&self, vessel_a: O::VesselType, vessel_b: O::VesselType) -> Vec<(SimTimeSpan, f64)> {
        let reference_cycle = O::VesselType::max_cycle();
        let heart_rate = self.core_connector.get::<HeartRate>().unwrap_or(&self.base_heart_rate);

        Self::distance_factor_between(vessel_a, vessel_b)
            .into_iter()
            .map(|(dist, fact)| {
                let diffusion_delay = (f64::from(dist) / f64::from(reference_cycle)) * (heart_rate.as_ref() / self.base_heart_rate.as_ref()) * self.base_diffusion_time;
                (SimTimeSpan(diffusion_delay), fact)
            }).collect()
    }

    fn get_downstream_add(v: O::VesselType) -> (impl Iterator<Item=O::VesselType>, u32) {
        if v.downstream().len() == 0 {
            // Pulmonary circulation length (at the ends of the systemic circulation tree)
            let pulm_len = std::cmp::max(O::VesselType::max_cycle() / SimpleBloodFlow::<O>::PULMONARY_RATIO, 1);
            (O::VesselType::start_vessels(), pulm_len + 1)
        }
        else {
            (v.downstream(), 1)
        }
    }

    // Internal recursive function to find the distance between any arbitrary vessel
    fn dist_calc(
        a: O::VesselType,
        b: O::VesselType,
        visited: &mut Vec<O::VesselType>,
        factor: f64
    ) -> Vec<(u32, f64)> {
        // If we've hit a cycle, return immediately
        if visited.contains(&a) {
            log::trace!("Path: {:?} -> {:?}", visited, a);
            return vec![];
        }

        let a_factor = {
            if a.upstream().len() > 0 {
                factor / a.upstream().len() as f64
            }
            else {
                factor
            }
        };
        
        if a == b {
            return vec![(0, a_factor)];
        }

        // Add the current node to the list
        visited.push(a);
        
        let (downstream, add_amount) = Self::get_downstream_add(a);

        let res = downstream
            .map(|v| {
                let mut items = Self::dist_calc(v, b, visited, a_factor);
                for (x, _) in items.iter_mut() {
                    *x += add_amount;
                }
                items
            })
            .fold(vec![], |mut acc, mut r| {
                acc.append(&mut r);
                acc
            });

        visited.pop();
        res
    }

    fn distance_factor_between(vessel_a: O::VesselType, vessel_b: O::VesselType) -> Vec<(u32, f64)> {
        if let Some(d) = DIST_CACHE.get_or_init(|| {
            Arc::new(RwLock::new(VesselDistanceCache::new()))
        }).read().unwrap().get(&TypeId::of::<O>(), vessel_a.into(), vessel_b.into()) {
            return d.clone();
        }

        let mut visited = Vec::new();
        let result = Self::dist_calc(vessel_a, vessel_b, &mut visited, 1.0);

        let cache = DIST_CACHE.get().unwrap();

        cache.write().unwrap().insert(
            TypeId::of::<O>(),
            vessel_a.into(),
            vessel_b.into(),
            result
        );

        cache.read()
            .unwrap()
            .get(&TypeId::of::<O>(), vessel_a.into(), vessel_b.into())
            .unwrap()
            .clone()
    }
}

impl<O: Organism> CoreComponent<O> for SimpleBloodFlow<O> {
    fn core_init(&mut self, initializer: &mut mortalsim_core::sim::layer::core::CoreInitializer<O>) {
        initializer.notify::<HeartRate>();
        initializer.notify::<AorticBloodPressure>();
    }
    fn core_connector(&mut self) -> &mut CoreConnector<O> {
        return &mut self.core_connector
    }
}

impl<O: Organism> CirculationComponent<O> for SimpleBloodFlow<O> {
    fn circulation_init(&mut self, circulation_initializer: &mut mortalsim_core::sim::layer::circulation::CirculationInitializer<O>) {
        circulation_initializer.notify_any_change();
    }

    fn circulation_connector(&mut self) -> &mut CirculationConnector<O> {
        return &mut self.circ_connector
    }
}

impl<O: Organism> SimComponent<O> for SimpleBloodFlow<O> {
    fn id(&self) -> &'static str {
        "SimpleBloodFlow"
    }
    fn attach(self, registry: &mut mortalsim_core::sim::component::ComponentRegistry<O>) {
        registry.add_core_circulation_component(self)
    }
    fn run(&mut self) {
        let mut change_list = Vec::new();
        let mut all_list = Vec::new();
        self.circ_connector.with_blood_stores(|vessel, store| {
            all_list.push(vessel);
            if store.has_new_changes() {
                log::debug!("New changes on {:?}", vessel);
                change_list.push(vessel);
            }
        });

        for source in change_list.iter() {
            for target in all_list.iter().filter(|v| *v != source) {
                let mut source_store = self.circ_connector.blood_store(source).unwrap();
                let mut target_store = self.circ_connector.blood_store(target).unwrap();

                log::debug!("propagating changes from {:?} to {:?}", source, target);

                for (delay, factor) in self.calculate_blood_delays(*source, *target) {
                    for (substance, change) in source_store.get_new_direct_changes() {
                        target_store.schedule_dependent_change(
                            substance,
                            self.circ_connector.sim_time() + delay,
                            factor,
                            change,
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mortalsim_core::math::BoundFn;
    use mortalsim_core::sim::organism::test::{TestBloodVessel, TestOrganism};
    use mortalsim_core::substance::{Substance, SubstanceChange, SubstanceConcentration};
    use mortalsim_core::units::mechanical::Frequency;
    use mortalsim_core::event::HeartRate;
    use mortalsim_core::sim::organism::test::TestSim;
    use mortalsim_core::SimTime;
    use mortalsim_core::sim::Sim;
    use mortalsim_test_harness::blood_checker::{BloodCheckerComponent, SubstanceConcentrationRange};

    use super::*;

    #[test_log::test]
    fn distance_factor_ao_ab() {
        let res = SimpleBloodFlow::<TestOrganism>::distance_factor_between(TestBloodVessel::Aorta, TestBloodVessel::AbdominalAorta);
        for (dist, fact) in res {
            assert_eq!(dist, 1);
            assert_eq!(fact, 1.0);
        }
    }
    #[test_log::test]
    fn distance_factor_ao_vc() {
        let res = SimpleBloodFlow::<TestOrganism>::distance_factor_between(TestBloodVessel::Aorta, TestBloodVessel::InferiorVenaCava);
        for (dist, fact) in res {
            assert_eq!(dist, 4);
            assert_eq!(fact, 0.5);
        }
    }
    #[test_log::test]
    fn distance_factor_rf_lf() {
        let res = SimpleBloodFlow::<TestOrganism>::distance_factor_between(TestBloodVessel::RightFemoralArtery, TestBloodVessel::LeftFemoralArtery);
        for (dist, fact) in res {
            assert_eq!(dist, 6);
            assert_eq!(fact, 0.5);
        }
    }

    #[test_log::test]
    fn blood_delay() {
        let sbf = SimpleBloodFlow::<TestOrganism>::new(
            HeartRate(Frequency::from_Hz(60.0)),
            Time::from_s(60.0),
        );

        for (d1, _f) in sbf.calculate_blood_delays(TestBloodVessel::Aorta, TestBloodVessel::AbdominalAorta) {
            assert!(
                d1 < SimTimeSpan::from_s(60.0) && d1 > SimTimeSpan::from_s(1.0),
                "Aorta->AbdominalAorta delay {d1} is not in a reasonable range."
            );
        }
        
        for (d2, _f) in sbf.calculate_blood_delays(TestBloodVessel::Aorta, TestBloodVessel::VenaCava) {
            assert!(
                d2 < SimTimeSpan::from_s(60.0) && d2 > SimTimeSpan::from_s(20.0),
                "Aorta->VenaCava delay {d2} is not in a reasonable range."
            )
        }
    }

    fn blood_component_aorta(time_factor: f64) -> BloodCheckerComponent<TestOrganism> {
        BloodCheckerComponent::new(
            TestBloodVessel::Aorta,
            vec![
                (
                    SimTime::from_s(0.0),
                    Substance::O2,
                    SubstanceChange::new(
                        SimTime::from_s(1.0),
                        SubstanceConcentration::from_uM(300.0),
                        SimTimeSpan::from_s(30.0),
                        BoundFn::Linear,
                    ),
                )
            ],
            vec![
                // 36s delay from each other vessel
                (SimTime::from_s(5.0*time_factor), Substance::CO2, SubstanceConcentrationRange::new(-0.1, 0.1)),
                (SimTime::from_s(120.0*time_factor), Substance::CO2, SubstanceConcentrationRange::new(99.0, 101.0)),
                (SimTime::from_s(140.0*time_factor), Substance::CO2, SubstanceConcentrationRange::new(3.0, 5.0)),
            ],
        )
    }

    fn blood_component_left_arm(time_factor: f64) -> BloodCheckerComponent<TestOrganism> {
        BloodCheckerComponent::new(
            TestBloodVessel::LeftAxillaryVein,
            vec![
                (
                    SimTime::from_s(1.0),
                    Substance::CO2,
                    SubstanceChange::new(
                        SimTime::from_s(5.0),
                        SubstanceConcentration::from_uM(200.0),
                        SimTimeSpan::from_s(20.0),
                        BoundFn::Linear,
                    ),
                )
            ],
            vec![
                // 24s delay
                (SimTime::from_s(25.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(-0.1, 0.1)),
                (SimTime::from_s(35.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(99.9, 100.1)),
                (SimTime::from_s(45.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(199.9, 200.1)),
                (SimTime::from_s(55.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(299.9, 300.1)),
            ],
        )
    }

    fn blood_component_right_leg(time_factor: f64) -> BloodCheckerComponent<TestOrganism> {
        BloodCheckerComponent::new(
            TestBloodVessel::RightFemoralVein,
            vec![
                (
                    SimTime::from_s(1.0),
                    Substance::CO2,
                    SubstanceChange::new(
                        SimTime::from_s(5.0),
                        SubstanceConcentration::from_uM(200.0),
                        SimTimeSpan::from_s(20.0),
                        BoundFn::Linear,
                    ),
                )
            ],
            vec![
                // 36s typical delay
                (SimTime::from_s(37.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(-0.1, 0.1)),
                (SimTime::from_s(47.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(99.9, 100.1)),
                (SimTime::from_s(57.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(199.9, 200.1)),
                (SimTime::from_s(67.0*time_factor), Substance::O2, SubstanceConcentrationRange::new(299.9, 300.1)),
            ],
        )
    }

    #[test_log::test]
    fn test_blood_flow() {
        let bhr = HeartRate(Frequency::from_Hz(60.0));
        let bdt = Time::from_s(60.0);
        let mut sim = TestSim::new();
        sim.add_component(SimpleBloodFlow::new(bhr, bdt)).unwrap();
        sim.add_component(blood_component_aorta(1.0)).unwrap();
        sim.add_component(blood_component_left_arm(1.0)).unwrap();
        sim.add_component(blood_component_right_leg(1.0)).unwrap();

        for _ in 1..120 {
            sim.advance_by(SimTimeSpan::from_s(1.0));
        }
    }
}
