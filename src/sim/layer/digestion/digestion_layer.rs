
use std::collections::{HashMap, BTreeMap, HashSet};
use std::marker::PhantomData;
use crate::sim::layer::InternalLayerTrigger;
use crate::sim::{SimConnector, SimTime};
use crate::sim::organism::Organism;
use crate::util::{IdType, secs};
use crate::sim::component::{SimComponent, SimComponentProcessor};

use super::{ConsumeEvent, DigestionDirection, EliminateEvent};
use super::component::{DigestionComponent, DigestionInitializer};
use super::component::connector::Consumed;
use super::consumable::Consumable;

type ConsumableId = IdType;

pub struct DigestionLayer<O: Organism> {
    pd: PhantomData<O>,
    /// Default duration each component receives a consumable for
    default_digestion_duration: SimTime,
    /// Tracks the order in which substance stores pass
    /// through each component, according to the order
    /// they were added, as well as whether they should
    /// trigger due to a new consumable coming in.
    component_map: HashMap<&'static str, usize>,
    /// Keeps track of component indices which should
    /// trigger due to a new consumable coming in.
    trigger_map: HashSet<usize>,
    /// Map to track stores in between components
    consumed_map: BTreeMap<usize, Vec<Consumed>>,
    /// Consumables staged for elimination
    elimination_list: Vec<(Consumable, DigestionDirection)>,
    /// Internal trigger id to unschedule if needed
    internal_trigger_id: Option<IdType>,
}

// impl fmt::Debug for DigestionLayer {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "DigestionLayer {{ consumed_map: {:?} }}",
//             self.consumed_map
//         )
//     }
// }

impl<O: Organism> DigestionLayer<O> {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            default_digestion_duration: secs!(60.0),
            component_map: HashMap::new(),
            trigger_map: HashSet::new(),
            consumed_map: BTreeMap::new(),
            elimination_list: Vec::new(),
            internal_trigger_id: None,
        }
    }

    /// Consume a new SubstanceStore
    fn consume(&mut self, consumable: Consumable) {
        let consumed = Consumed::new(consumable);
        self.consumed_map.entry(0).or_default().push(consumed);
    }

    fn component_position<T: SimComponent<O>>(&self, component: &T) -> usize {
        *self.component_map.get(component.id()).expect("Digestion component position is missing!")
    }

}

impl<O: Organism, T: DigestionComponent<O>> SimComponentProcessor<O, T> for DigestionLayer<O> {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = DigestionInitializer::new();
        component.digestion_init(&mut initializer);

        self.component_map.insert(component.id(), self.component_map.len());
    }

    fn pre_exec(&mut self, connector: &mut SimConnector) {
        if let Some(id) = self.internal_trigger_id.take() {
            connector.time_manager.unschedule_event(&id);
        }

        for evt in connector.active_events.iter() {
            if let Some(consume_evt) = evt.downcast_ref::<ConsumeEvent>() {
                self.consume(consume_evt.0.clone());
            }
        }
        // Keep track of vector indices of items which need to move
        let mut moving_indices: Vec<Vec<usize>> = vec![vec![]; self.consumed_map.len()];
        for (pos, consumed_list) in self.consumed_map.iter_mut() {
            for (idx, consumed) in consumed_list.iter_mut().enumerate() {
                // advance time for the consumable
                consumed.advance(connector.sim_time());
                // if time has exceeded the exit time, stage it for movement
                if consumed.exit_time <= connector.sim_time() {
                    moving_indices.get_mut(*pos)
                                  .expect("moving_indices initialized improperly")
                                  .push(idx);
                }
            }
        }
        // position of the last digestion component
        let last = self.consumed_map.len() - 1;
        for (pos, indices) in moving_indices.into_iter().enumerate() {
            for idx in indices {
                let mut removed = self.consumed_map.get_mut(&pos)
                    .expect("moving_indices referenced invalid position")
                    .remove(idx);

                // update entry time
                removed.entry_time = removed.exit_time;

                // set defaults, which the component may override
                removed.exit_time = removed.entry_time + self.default_digestion_duration;
                removed.exit_direction = DigestionDirection::FORWARD;

                // Check cases for elimination, either forward or backward
                if  (pos == 0 && removed.exit_direction == DigestionDirection::BACK) ||
                    (pos == last && removed.exit_direction == DigestionDirection::FORWARD) {
                        let evt = Box::new(EliminateEvent::new(removed.consumable, removed.exit_direction));
                        connector.time_manager.schedule_event(secs!(0.0), evt);
                    }
                else {
                    match removed.exit_direction {
                        DigestionDirection::FORWARD => {
                            let target_idx = pos + 1;
                            self.consumed_map.get_mut(&target_idx).expect("invalid index").push(removed);
                            self.trigger_map.insert(target_idx);
                        }
                        DigestionDirection::BACK => {
                            let target_idx = pos - 1;
                            self.consumed_map.get_mut(&target_idx).expect("invalid index").push(removed);
                            self.trigger_map.insert(target_idx);
                        }
                        DigestionDirection::EXHAUSTED => {
                            // Drop the removed consumable completely
                        }
                    }
                }
            }
        }
    }

    fn check_component(&mut self, component: &T) -> bool {
        let component_pos = self.component_position(component);
        self.trigger_map.contains(&component_pos)
    }

    fn prepare_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_pos = self.component_position(component);

        // move consumed items from the layer map into the component connector
        let consumed_list = self.consumed_map.entry(component_pos).or_default();
        component.digestion_connector().consumed_list.extend(consumed_list.drain(..));
    }

    fn process_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_pos = self.component_position(component);

        // move consumed items from the component connector back into the layer map
        let consumed_list = &mut component.digestion_connector().consumed_list;
        self.consumed_map.entry(component_pos).or_default().extend(consumed_list.drain(..));

        // Reset the trigger
        self.trigger_map.remove(&component_pos);
    }

    fn post_exec(&mut self, connector: &mut SimConnector) {

        if let Some(min_consumed) = self.consumed_map.values().flatten()
            .min_by(|a, b| a.exit_time.partial_cmp(&b.exit_time).unwrap()) {
            
            let mut delay = secs!(0.0);
            if min_consumed.exit_time > connector.sim_time() {
                delay = min_consumed.exit_time - connector.sim_time();
            }
            let id = connector.time_manager.schedule_event(delay, Box::new(InternalLayerTrigger));
            self.internal_trigger_id = Some(id);
        }
    }
}
