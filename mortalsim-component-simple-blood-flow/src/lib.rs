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
    map: HashMap<TypeId, HashMap<(&'static str, &'static str), T>>,
}

impl<T> VesselDistanceCache<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn get(&self, organism_type: &TypeId, a: &'static str, b: &'static str) -> Option<&T> {
        self.map.get(organism_type)?.get(&(a, b))
    }
    pub fn insert(&mut self, organism_type: TypeId, a: &'static str, b: &'static str, val: T) {
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

    fn calculate_blood_delay(&self, vessel_a: O::VesselType, vessel_b: O::VesselType) -> SimTimeSpan {
        let reference_cycle = O::VesselType::max_cycle();
        let dist = Self::distance_between(vessel_a, vessel_b);

        let heart_rate = self.core_connector.get::<HeartRate>().unwrap_or(&self.base_heart_rate);

        let diffusion_delay = (f64::from(dist) / f64::from(reference_cycle)) * (heart_rate.as_ref() / self.base_heart_rate.as_ref()) * self.base_diffusion_time;
        
        SimTimeSpan(diffusion_delay)
    }

    fn distance_between(vessel_a: O::VesselType, vessel_b: O::VesselType) -> u32 {
        if let Some(d) = DIST_CACHE.get_or_init(|| {
            Arc::new(RwLock::new(VesselDistanceCache::new()))
        }).read().unwrap().get(&TypeId::of::<O>(), vessel_a.into(), vessel_b.into()) {
            return *d
        }

        // Internal recursive function to find the distance between any arbitrary vessel
        fn dist_calc<O: Organism>(a: O::VesselType, b: O::VesselType, visited: &mut Vec<O::VesselType>) -> u32 {
            // If we've hit a cycle, return immediately
            if visited.contains(&a) {
                return O::VesselType::max_cycle()*2
            }
            else if a == b {
                return 0
            }

            // Add the current node to the list
            visited.push(a);

            let res;

            if a.downstream().peekable().peek().is_none() {
                // Pulmonary circulation length (at the ends of the systemic circulation tree)
                let pulm_len = std::cmp::max(O::VesselType::max_cycle() / SimpleBloodFlow::<O>::PULMONARY_RATIO, 1);

                // add that length to the minimum found from the start vessels
                res = pulm_len + O::VesselType::start_vessels()
                    .map(|v| 1 + dist_calc::<O>(v, b, visited)).min().unwrap()
            }
            else {
                // find the minimum distance from each downstream node
                res = a.downstream().map(|v| 1 + dist_calc::<O>(v, b, visited)).min().unwrap()
            }
            visited.pop();
            res
        }

        let mut visited = Vec::new();
        let result = dist_calc::<O>(vessel_a, vessel_b, &mut visited);
        DIST_CACHE.get().unwrap().write().unwrap().insert(
            TypeId::of::<O>(),
            vessel_a.into(),
            vessel_b.into(),
            result
        );
        result
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
                change_list.push(vessel);
            }
        });

        for source in change_list.iter() {
            for target in all_list.iter() {
                let mut source_store = self.circ_connector.blood_store(source).unwrap();
                let mut target_store = self.circ_connector.blood_store(target).unwrap();
                let delay = self.calculate_blood_delay(*source, *target);

                for (substance, change) in source_store.get_new_direct_changes() {
                    target_store.schedule_dependent_change(
                        substance,
                        self.circ_connector.sim_time() + delay,
                        change,
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use mortalsim_core::units::mechanical::Frequency;
    // use mortalsim_core::event::HeartRate;
    use mortalsim_human::{HumanBloodVessel, HumanOrganism};

    use super::*;

    #[test]
    fn distance_between() {
        assert_eq!(
            SimpleBloodFlow::<HumanOrganism>::distance_between(HumanBloodVessel::Aorta, HumanBloodVessel::AbdominalAorta),
            2
        );
        assert_eq!(
            SimpleBloodFlow::<HumanOrganism>::distance_between(HumanBloodVessel::Aorta, HumanBloodVessel::InferiorVenaCava),
            5
        );
        assert_eq!(
            SimpleBloodFlow::<HumanOrganism>::distance_between(HumanBloodVessel::RightFibularArtery, HumanBloodVessel::LeftFibularArtery),
            16
        );
    }
}
