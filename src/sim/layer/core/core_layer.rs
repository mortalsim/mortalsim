use crate::event::Event;
use crate::sim::component::SimComponentProcessor;
use crate::sim::SimConnector;
use crate::sim::Time;
use crate::util::id_gen::{IdType, InvalidIdError};
use anyhow::Result;
use either::Either;
use std::any::TypeId;
use std::cell::{Ref, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::component::{CoreComponent, CoreComponentInitializer};

pub struct CoreLayer {
    module_notifications: HashMap<TypeId, Vec<(i32, &'static str)>>,
    transformer_id_map: HashMap<&'static str, Vec<IdType>>,
    /// Map of pending updates for each module
    notify_map: HashMap<&'static str, HashSet<TypeId>>,
}

impl fmt::Debug for CoreLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CoreLayer {{ notifications: {:?}, transformer_id_map: {:?}, notify_map: {:?} }}",
            self.module_notifications, self.transformer_id_map, self.notify_map
        )
    }
}

impl CoreLayer {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> CoreLayer {
        CoreLayer {
            module_notifications: HashMap::new(),
            transformer_id_map: HashMap::new(),
            notify_map: HashMap::new(),
        }
    }

    /// handles internal registrations and initial outputs for modules
    pub fn pending_updates<'a>(&'a mut self) -> impl Iterator<Item = &'static str> + 'a {
        self.notify_map.keys().map(|n| *n)
    }

    pub fn clear_notifications(&mut self) {
        self.notify_map.clear()
    }
}

impl<T: CoreComponent> SimComponentProcessor<T> for CoreLayer {
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let mut initializer = CoreComponentInitializer::new();
        component.core_init(&mut initializer);

        let mut transformer_ids = Vec::new();
        for transformer in initializer.pending_transforms {
            transformer_ids.push(connector.time_manager.insert_transformer(transformer));
        }
        self.transformer_id_map
            .insert(component.id(), transformer_ids);

        for (priority, evt) in initializer.pending_notifies {
            let type_id = evt.type_id();
            match self.module_notifications.get_mut(&type_id) {
                None => {
                    self.module_notifications
                        .insert(type_id, vec![(priority, component.id())]);
                }
                Some(list) => {
                    list.push((priority, component.id()));
                }
            }
        }
    }

    fn prepare_component(&mut self, connector: &SimConnector, component: &mut T) -> bool {
        // Update connector before module execution

        component.core_connector().trigger_events = {
            let notify_ids = self
                .notify_map
                .remove(component.id())
                .unwrap_or(HashSet::new());
            notify_ids
                .iter()
                .map(|id| connector.state.lock().unwrap().get_state_ref(id).unwrap())
                .collect()
        };

        let comp_connector = component.core_connector();
        comp_connector.sim_time = connector.time_manager.get_time();

        // If this comp_connector doesn't yet have a reference to the sim state, set it now
        if !Arc::ptr_eq(&comp_connector.sim_state, &connector.state) {
            comp_connector.sim_state = connector.state.clone();
        }

        // Trigger the module only if the trigger events list is non empty
        !comp_connector.trigger_events.is_empty()
    }

    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_connector = component.core_connector();

        // Unschedule any requested events
        if comp_connector.unschedule_all {
            for (_, id_map) in comp_connector.scheduled_events.drain() {
                for (schedule_id, _) in id_map {
                    connector
                        .time_manager
                        .unschedule_event(&schedule_id)
                        .unwrap();
                }
            }
        } else {
            for schedule_id in comp_connector.pending_unschedules.drain(..) {
                connector
                    .time_manager
                    .unschedule_event(&schedule_id)
                    .unwrap();
                let type_id = comp_connector
                    .schedule_id_type_map
                    .remove(&schedule_id)
                    .unwrap();
                comp_connector.scheduled_events.remove(&type_id).unwrap();
            }
        }

        // Schedule any new events
        for (wait_time, evt) in comp_connector.pending_schedules.drain(..) {
            let type_id = evt.type_id();
            let schedule_id = connector.time_manager.schedule_event(wait_time, evt);
            comp_connector
                .schedule_id_type_map
                .insert(schedule_id, type_id);
            match comp_connector.scheduled_events.get_mut(&type_id) {
                None => {
                    let mut map = HashMap::new();
                    map.insert(schedule_id, wait_time);
                    comp_connector.scheduled_events.insert(type_id, map);
                }
                Some(map) => {
                    map.insert(schedule_id, wait_time);
                }
            }
        }
    }
}
