use crate::sim::component::{SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::{InternalLayerTrigger, SimLayer, SimLayerSync};
use crate::sim::organism::Organism;
use crate::sim::SimConnector;
use crate::id_gen::IdType;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;
use std::mem::swap;

use super::component::{CoreComponent, CoreInitializer};
use super::CoreConnector;

#[derive(Debug)]
pub struct CoreLayer<O: Organism> {
    pd: PhantomData<O>,
    /// Map of event types to component names to trigger when
    /// the event is emitted
    module_notifications: HashMap<TypeId, Vec<&'static str>>,
    /// Map of pending updates for each module
    notify_map: HashMap<&'static str, HashSet<TypeId>>,
}

impl<O: Organism> CoreLayer<O> {
    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            module_notifications: HashMap::new(),
            notify_map: HashMap::new(),
        }
    }

    fn prep_connector(&mut self, connector: &mut SimConnector, component: &mut impl CoreComponent<O>) {
        component.core_connector().trigger_events = self
            .notify_map
            .remove(component.id())
            .unwrap_or(HashSet::new())
            .iter()
            .map(|id| *id)
            .collect();

        let comp_connector = component.core_connector();
        comp_connector.sim_time = connector.sim_time();
    }

    fn process_connector(&mut self, connector: &mut SimConnector, component: &mut impl CoreComponent<O>) {
        let comp_id = component.id();
        let comp_connector = component.core_connector();

        // Unschedule any requested events
        if comp_connector.unschedule_all {
            for (_, schedule_id) in comp_connector.scheduled_id_map.drain() {
                log::trace!("Unscheduling change {} for component {}", schedule_id, comp_id);
                connector
                    .time_manager
                    .unschedule_event(&schedule_id)
                    .unwrap();
            }
        } else {
            for schedule_id in comp_connector.pending_unschedules.drain(..) {
                log::trace!("Unscheduling change {} for component {}", schedule_id, comp_id);
                connector
                    .time_manager
                    .unschedule_event(&schedule_id)
                    .unwrap();
            }
        }

        // Unschedule any requested transforms
        for transformer_id in comp_connector.pending_untransforms.drain(..) {
            log::trace!("Unscheduling transform {} for component {}", transformer_id, comp_id);
            connector.time_manager.unset_transform(&transformer_id)
                .expect("tried to unset an invalid transformer_id!");
        }

        // Schedule any new events
        for (wait_time, (local_id, evt)) in comp_connector.pending_schedules.drain(..) {
            let schedule_id = connector.time_manager.schedule_event(wait_time, evt);
            log::trace!("Scheduling event {} for component {}", schedule_id, comp_id);
            comp_connector
                .scheduled_id_map
                .insert(local_id, schedule_id);
        }

        // Add any pending transformations from the component
        for (local_id, transformer) in comp_connector.pending_transforms.drain(..) {
            let transform_id = connector.time_manager.insert_transformer(transformer);
            log::trace!("Scheduling transform {} for component {}", transform_id, comp_id);
            comp_connector.transform_id_map.insert(local_id, transform_id);
        }
    }
}

impl<O: Organism> SimLayer for CoreLayer<O> {
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        // Drain events from the last run, if any
        connector.active_events.drain(..);

        connector
            .time_manager
            .next_events()
            .map(|x| x.1)
            .flatten()
            .for_each(|evt| {
                // populate the notify list for this event
                if let Some(notify_list) = self.module_notifications.get(&evt.type_id()) {
                    for comp_id in notify_list {
                        self.notify_map
                            .entry(comp_id)
                            .or_default()
                            .insert(evt.type_id());
                    }
                }

                // Internal layer trigger events don't end up on the state
                // or in the active_events list
                if !evt.is::<InternalLayerTrigger>() {
                    connector.active_events.push(evt.into());
                }
            })
    }

    fn post_exec(&mut self, connector: &mut SimConnector) {
        // update state
        for evt in connector.active_events.iter() {
            if !evt.transient() {
                log::debug!("Updating state with event {:?}", evt);
                connector.state.put_state(evt.clone());
            }
            else {
                log::debug!("Dropping transient event {:?}", evt);
            }
        }
    }
}


impl<O: Organism> SimLayerSync for CoreLayer<O> {
    fn pre_exec_sync(&mut self, connector: &mut SimConnector) {
        self.pre_exec(connector);
    }

    fn post_exec_sync(&mut self, connector: &mut SimConnector) {
        self.post_exec(connector)
    }

}

impl<O: Organism, T: CoreComponent<O>> SimComponentProcessor<O, T> for CoreLayer<O> {
    fn setup_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_id = component.id();
        let mut initializer = CoreInitializer::new();
        component.core_init(&mut initializer);

        let comp_connector = component.core_connector();

        comp_connector.id_gen = initializer.id_gen;

        // Add any pending transformations from the component
        for (local_id, transformer) in initializer.pending_transforms {
            let transform_id = connector.time_manager.insert_transformer(transformer);
            log::debug!("Adding initial transform {} from component {}", transform_id, comp_id);
            comp_connector.transform_id_map.insert(local_id, transform_id);
        }

        for type_id in initializer.pending_notifies {
            log::debug!("Adding notification for event type {:?} for component {}", type_id, comp_id);
            self.module_notifications
                .entry(type_id)
                .or_default()
                .push(component.id());
        }

        for event in initializer.initial_outputs {
            log::debug!("Setting initial state with event {:?} from component {}", event, comp_id);
            connector.state.put_state(event.into());
        }
    }

    fn check_component(&mut self, component: &T) -> bool {
        // Trigger the module only if the notify_list is non empty
        self.notify_map.contains_key(component.id())
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.prep_connector(connector, component);

        // Swap out state and active events with the connector
        swap(&mut connector.state, &mut component.core_connector().sim_state);
        swap(&mut connector.active_events, &mut component.core_connector().active_events);
    }

    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        // Swap back state and active events with the connector
        swap(&mut connector.state, &mut component.core_connector().sim_state);
        swap(&mut connector.active_events, &mut component.core_connector().active_events);

        self.process_connector(connector, component)
    }

    fn remove_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let comp_id = component.id();

        // unschedule all the component's pending events and transforms
        for schedule_id in component.core_connector().scheduled_id_map.values() {
            log::debug!("Unscheduling event {} from component {}", schedule_id, comp_id);
            connector.time_manager.unschedule_event(schedule_id).ok();
        }
        for transformer_id in component.core_connector().transform_id_map.values() {
            log::debug!("Unscheduling transform {} from component {}", transformer_id, comp_id);
            connector.time_manager.unset_transform(transformer_id).ok();
        }
    }

}

impl<O: Organism, T: CoreComponent<O>> SimComponentProcessorSync<O, T> for CoreLayer<O> {
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.setup_component(connector, component);

    }

    fn check_component_sync(&mut self, component: &T) -> bool {
        self.check_component(component)
    }

    fn prepare_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.prep_connector(connector, component);
        
        // Merge the events which were modified since the last update into the
        // component's copy of state
        component.core_connector().sim_state.merge_tainted(&mut connector.state);

        // Clone any active events into the connector
        component.core_connector().active_events.extend(
            connector.active_events
                .iter()
                .map(|evt| evt.clone())
        )
    }

    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_connector(connector, component);

        // clear active events from the core connector
        component.core_connector().active_events.drain(..);
    }

    fn remove_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.remove_component(connector, component)
    }
}


pub mod test {
    use std::panic::catch_unwind;
    use std::sync::Mutex;
    use std::thread::{scope, spawn};

    use simple_si_units::base::Amount;

    use super::CoreLayer;
    use crate::units::base::Distance;
    use crate::event::test::{TestEventA, TestEventB};
    use crate::sim::component::{SimComponent, SimComponentProcessor, SimComponentProcessorSync};
    use crate::sim::layer::{SimLayer, SimLayerSync};
    use crate::sim::layer::core::component::test::{TestComponentA, TestComponentB};
    use crate::sim::layer::core::component::connector::test::basic_event_a;
    use crate::sim::organism::test::TestOrganism;
    use crate::sim::{SimConnector, SimTime};
    use crate::{secs, SimTimeSpan};

    #[test]
    fn layer_process() {
        let mut layer = CoreLayer::<TestOrganism>::new();
        let mut component_a = TestComponentA::new();
        let mut component_b = TestComponentB::new();
        let mut connector = SimConnector::new();

        layer.setup_component(&mut connector, &mut component_a);
        layer.setup_component(&mut connector, &mut component_b);

        connector.time_manager.schedule_event(SimTimeSpan::from_s(1.0), Box::new(TestEventA::new(Distance::from_m(1.0))));
        connector.time_manager.schedule_event(SimTimeSpan::from_s(1.0), Box::new(TestEventB::new(Amount::from_mmol(1.0))));

        connector.time_manager.advance_by(SimTimeSpan::from_s(2.0));

        layer.pre_exec(&mut connector);

        layer.prepare_component(&mut connector, &mut component_a);
        component_a.run();
        layer.process_component(&mut connector, &mut component_a);

        layer.prepare_component(&mut connector, &mut component_b);
        component_b.run();
        layer.process_component(&mut connector, &mut component_b);

        layer.post_exec(&mut connector);

        assert!(connector.state.get_state::<TestEventA>().is_some());
        assert!(connector.state.get_state::<TestEventB>().is_none());

        for evt in  connector.active_events.drain(..) {
            assert!(evt.is::<TestEventA>() || evt.is::<TestEventB>());

            if evt.is::<TestEventA>() {
                assert_eq!(evt.downcast_ref::<TestEventA>().unwrap().len, Distance::from_m(3.0));
            }
            if evt.is::<TestEventB>() {
                assert_eq!(evt.downcast_ref::<TestEventB>().unwrap().amt, Amount::from_mmol(1.0));
            }
        }
    }

    #[test]
    fn layer_process_sync() {
        let layer = Mutex::new(CoreLayer::<TestOrganism>::new());
        let mut component_a = TestComponentA::new();
        let mut component_b = TestComponentB::new();
        let connector = Mutex::new(SimConnector::new());

        layer.lock().unwrap().setup_component_sync(&mut connector.lock().unwrap(), &mut component_a);
        layer.lock().unwrap().setup_component_sync(&mut connector.lock().unwrap(), &mut component_b);

        connector.lock().unwrap().time_manager.schedule_event(SimTimeSpan::from_s(1.0), Box::new(TestEventA::new(Distance::from_m(1.0))));
        connector.lock().unwrap().time_manager.schedule_event(SimTimeSpan::from_s(1.0), Box::new(TestEventB::new(Amount::from_mmol(1.0))));

        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(2.0));

        layer.lock().unwrap().pre_exec_sync(&mut connector.lock().unwrap());

        scope(|s| {
            s.spawn(|| {
                layer.lock().unwrap().prepare_component_sync(&mut connector.lock().unwrap(), &mut component_a);
                component_a.run();
                layer.lock().unwrap().process_component_sync(&mut connector.lock().unwrap(), &mut component_a);
            });

            s.spawn(|| {
                layer.lock().unwrap().prepare_component_sync(&mut connector.lock().unwrap(), &mut component_b);
                component_b.run();
                layer.lock().unwrap().process_component_sync(&mut connector.lock().unwrap(), &mut component_b);
            });
        });

        layer.lock().unwrap().post_exec_sync(&mut connector.lock().unwrap());

        assert!(connector.lock().unwrap().state.get_state::<TestEventA>().is_some());
        assert!(connector.lock().unwrap().state.get_state::<TestEventB>().is_none());

        for evt in  connector.lock().unwrap().active_events.drain(..) {
            assert!(evt.is::<TestEventA>() || evt.is::<TestEventB>());

            if evt.is::<TestEventA>() {
                assert_eq!(evt.downcast_ref::<TestEventA>().unwrap().len, Distance::from_m(3.0));
            }
            if evt.is::<TestEventB>() {
                assert_eq!(evt.downcast_ref::<TestEventB>().unwrap().amt, Amount::from_mmol(1.0));
            }
        }
    }
}
