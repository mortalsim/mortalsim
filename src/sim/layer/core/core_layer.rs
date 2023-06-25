use std::fmt;
use std::collections::{HashMap, HashSet, BTreeSet, VecDeque};
use std::any::TypeId;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use uuid::Uuid;
use anyhow::Result;
use either::Either;
use crate::event::Event;
use crate::sim::Time;
use crate::sim::component::SimComponentProcessor;
use crate::sim::layer::LayerConnector;
use crate::util::id_gen::{IdType, InvalidIdError};

use super::component::{CoreComponent, CoreComponentInitializer};

pub struct CoreLayer {
  module_notifications: HashMap<TypeId, Vec<(i32, &'static str)>>,
  transformer_id_map: HashMap<&'static str, Vec<IdType>>,
  /// Map of pending updates for each module
  notify_map: HashMap<&'static str, HashSet<TypeId>>,
}

impl fmt::Debug for CoreLayer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "CoreLayer {{ notifications: {:?}, transformer_id_map: {:?}, notify_map: {:?} }}",
    self.module_notifications,
    self.transformer_id_map,
    self.notify_map)
  }
}

impl CoreLayer {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new(layer_connector: &mut LayerConnector, components: &mut Vec<impl CoreComponent>) -> CoreLayer {
      let mut core = CoreLayer {
        module_notifications: HashMap::new(),
        transformer_id_map: HashMap::new(),
        notify_map: HashMap::new(),
      };
      for component in components {
        let mut initializer = CoreComponentInitializer::new();
        component.core_init(&mut initializer);
        core.setup_component(layer_connector, component, initializer);
      }
      core
    }
    
    /// handles internal registrations and initial outputs for modules
    pub(crate) fn setup_component(&mut self, layer_connector: &mut LayerConnector, component: &mut impl CoreComponent, initializer: CoreComponentInitializer) {
        let mut transformer_ids = Vec::new();
        for transformer in initializer.pending_transforms {
            transformer_ids.push(layer_connector.time_manager.as_mut().unwrap().insert_transformer(transformer));
        }
        self.transformer_id_map.insert(component.id(), transformer_ids);
        
        for (priority, evt) in initializer.pending_notifies {
            let type_id = evt.type_id();
            match self.module_notifications.get_mut(&type_id) {
                None => {
                    self.module_notifications.insert(type_id, vec![(priority, component.id())]);
                }
                Some(list) => {
                    list.push((priority, component.id()));
                }
            }
        }
    }

    pub(crate) fn pending_updates<'a>(&'a mut self) -> impl Iterator<Item = &'static str> + 'a {
        self.notify_map.keys().map(|n| { *n })
    }

    pub(crate) fn clear_notifications(&mut self) {
        self.notify_map.clear()
    }

}

// impl<T: CoreComponent> SimComponentProcessor<T> for CoreLayer {
//     fn prepare_component(&self, component: &mut T) {
//       // Update connector before module execution
//       let connector = component.core_connector();

//       connector.trigger_events = {
//         let notify_ids = self.notify_map.remove(component.id()).unwrap_or(HashSet::new());
//         notify_ids.iter().map(|id| { self.state.lock().unwrap().get_state_ref(id).unwrap() }).collect()
//       };
//       connector.sim_time = self.time_manager.get_time();

//       // If this connector doesn't yet have a reference to the sim state, set it now
//       if !Arc::ptr_eq(&connector.sim_state, &self.state) {
//         connector.sim_state = self.state.clone();
//       }
//     }

//     fn process_component(&mut self, component: &mut impl CoreComponent) {
//       let connector = component.core_connector();

//       // Unschedule any requested events
//       if connector.unschedule_all {
//         for (_, id_map) in connector.scheduled_events.drain() {
//           for (schedule_id, _) in id_map {
//             self.time_manager.unschedule_event(&schedule_id).unwrap();
//           }
//         }
//       }
//       else {
//         for schedule_id in connector.pending_unschedules.drain(..) {
//           self.time_manager.unschedule_event(&schedule_id).unwrap();
//           let type_id = connector.schedule_id_type_map.remove(&schedule_id).unwrap();
//           connector.scheduled_events.remove(&type_id).unwrap();
//         }
//       }

//       // Schedule any new events
//       for (wait_time, evt) in connector.pending_schedules.drain(..) {
//         let type_id = evt.type_id();
//         let schedule_id = self.time_manager.schedule_event(wait_time, evt);
//         connector.schedule_id_type_map.insert(schedule_id, type_id);
//         match connector.scheduled_events.get_mut(&type_id) {
//           None => {
//             let mut map = HashMap::new();
//             map.insert(schedule_id, wait_time);
//             connector.scheduled_events.insert(type_id, map);
//           },
//           Some(map) => {
//             map.insert(schedule_id, wait_time);
//           }
//         }
//       }
//     }
// }

