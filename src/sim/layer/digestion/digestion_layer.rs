
use std::borrow::BorrowMut;
use std::fmt;
use std::collections::{HashMap, BTreeMap, HashSet};
use crate::sim::{SimConnector, SimTime};
use crate::sim::organism::Organism;
use crate::substance::SubstanceStore;
use crate::util::{IdType, secs};
use crate::sim::component::SimComponentProcessor;

use super::DigestionDirection;
use super::component::{DigestionComponent, DigestionInitializer};
use super::component::connector::Consumed;
use super::consumable::Consumable;

type ConsumableId = IdType;

const DEFAULT_DIGESTION_DURATION: SimTime = secs!(60.0);

pub struct DigestionLayer {
    /// Current simulation time according to the layer
    sim_time: SimTime,
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

impl DigestionLayer {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> DigestionLayer {
        DigestionLayer {
            sim_time: SimTime::from_s(0.0),
            component_map: HashMap::new(),
            trigger_map: HashSet::new(),
            consumed_map: BTreeMap::new(),
            elimination_list: Vec::new(),
        }
    }

    /// Consume a new SubstanceStore
    pub fn consume(&mut self, consumable: Consumable) {
        let consumed = Consumed::new(consumable);
        self.consumed_map.entry(0).or_default().push(consumed);
    }

    /// Eliminate the remains of any SubstanceStores
    pub fn eliminate<'a>(&'a mut self) -> impl Iterator<Item = (Consumable, DigestionDirection)> + 'a {
        self.elimination_list.drain(..)
    }

    fn update(&mut self, sim_time: SimTime) {
        if sim_time == self.sim_time {
            return;
        }
        self.sim_time = sim_time;

        // Keep track of vector indices of items which need to move
        let mut moving_indices: Vec<Vec<usize>> = vec![vec![]; self.consumed_map.len()];
        for (pos, consumed_list) in self.consumed_map.iter_mut() {
            for (idx, consumed) in consumed_list.iter_mut().enumerate() {
                // advance time for the consumable
                consumed.advance(sim_time);
                // if time has exceeded the exit time, stage it for movement
                if consumed.exit_time <= sim_time {
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
                removed.exit_time = removed.entry_time + DEFAULT_DIGESTION_DURATION;
                removed.exit_direction = DigestionDirection::FORWARD;

                // Check cases for elimination, either forward or backward
                if  (pos == 0 && removed.exit_direction == DigestionDirection::BACK) ||
                    (pos == last && removed.exit_direction == DigestionDirection::FORWARD) {
                        self.elimination_list.push((removed.consumable, removed.exit_direction));
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
}

impl<O: Organism, T: DigestionComponent<O>> SimComponentProcessor<O, T> for DigestionLayer {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = DigestionInitializer::new();
        component.digestion_init(&mut initializer);

        self.component_map.insert(component.id(), self.component_map.len());
    }

    fn prepare_component(&mut self, connector: &SimConnector, component: &mut T) -> bool {
        // advances time and moves any consumables that need to be moved
        self.update(connector.sim_time);

        let component_pos = self.component_map.get(component.id()).expect("Digestion component position is missing!");
        let trigger = self.trigger_map.contains(component_pos);

        if trigger {
            // move consumed items from the layer map into the component connector
            let consumed_list = self.consumed_map.entry(*component_pos).or_default();
            component.digestion_connector().consumed_list.extend(consumed_list.drain(..));
        }

        trigger
    }

    fn process_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_pos = self.component_map.get(component.id()).expect("Digestion component position is missing!");

        // move consumed items from the component connector back into the layer map
        let consumed_list = &mut component.digestion_connector().consumed_list;
        self.consumed_map.entry(*component_pos).or_default().extend(consumed_list.drain(..));

        // Reset the trigger
        self.trigger_map.remove(component_pos);
    }
}
