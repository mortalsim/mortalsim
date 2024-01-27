use crate::event::Event;
use crate::sim::component::SimComponentProcessor;
use crate::sim::layer::InternalLayerTrigger;
use crate::sim::SimConnector;
use crate::sim::organism::Organism;
use crate::util::id_gen::IdType;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;
use std::mem::swap;
use std::sync::Arc;

use super::component::{CoreComponent, CoreInitializer};

pub struct CoreLayer<O: Organism> {
    pd: PhantomData<O>,
    module_notifications: HashMap<TypeId, Vec<(i32, &'static str)>>,
    transformer_id_map: HashMap<&'static str, Vec<IdType>>,
    /// Map of pending updates for each module
    notify_map: HashMap<&'static str, HashSet<TypeId>>,
}

impl<O: Organism> fmt::Debug for CoreLayer<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CoreLayer {{ notifications: {:?}, transformer_id_map: {:?}, notify_map: {:?} }}",
            self.module_notifications, self.transformer_id_map, self.notify_map
        )
    }
}

impl<O: Organism> CoreLayer<O> {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            module_notifications: HashMap::new(),
            transformer_id_map: HashMap::new(),
            notify_map: HashMap::new(),
        }
    }
    
    pub fn update(&mut self, connector: &mut SimConnector) {
        connector.time_manager.next_events()
            .map(|x| x.1)
            .flatten()
            .for_each(|evt| {
                if let Some(notify_list) = self.module_notifications.get(&evt.type_id()) {
                    for (_, comp_id) in notify_list {
                        self.notify_map.entry(comp_id).or_default().insert(evt.type_id());
                    }
                }
                // Internal layer trigger events don't end up on the state
                // or in the active_events list
                if evt.type_id() != TypeId::of::<InternalLayerTrigger>() {
                    let rc_evt: Arc<dyn Event> = evt.into();
                    connector.active_events.push(rc_evt.clone());
                    connector.state.put_state(rc_evt);
                }
        })
    }

}

impl<O: Organism, T: CoreComponent<O>> SimComponentProcessor<O, T> for CoreLayer<O> {

    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let mut initializer = CoreInitializer::new();
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

    fn check_component(&mut self, component: &T) -> bool {
        // Trigger the module only if the notify_list is non empty
        self.notify_map.contains_key(component.id())
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        component.core_connector().trigger_events = {
            let notify_ids = self
                .notify_map
                .remove(component.id())
                .unwrap_or(HashSet::new());
            notify_ids
                .iter()
                .map(|id| connector.state.get_state_ref(id).unwrap().type_id())
                .collect()
        };

        let comp_connector = component.core_connector();
        comp_connector.sim_time = connector.sim_time();

        // Swap out state with the connector
        swap(&mut connector.state, &mut comp_connector.sim_state);
    }

    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_connector = component.core_connector();
        
        // Swap back state with the connector
        swap(&mut connector.state, &mut comp_connector.sim_state);

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
