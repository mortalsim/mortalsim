use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::swap;
use std::sync::{Arc, Mutex};

use downcast_rs::Downcast;

use crate::sim::component::{SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::{InternalLayerTrigger, SimLayer, SimLayerSync};
use crate::sim::organism::Organism;
use crate::sim::SimConnector;
use crate::util::{secs, IdGenerator, IdType, OrderedTime};

use super::component::{NervousComponent, NervousInitializer};
use super::nerve_signal::NerveSignal;
use super::transform::NerveSignalTransformer;

pub struct NervousLayer<O: Organism> {
    /// ID generator for transform registration
    id_gen: IdGenerator,
    /// Map to keep track of which modules to notify for certain signals
    signal_notifies: HashMap<O::NerveType, HashMap<TypeId, HashSet<&'static str>>>,
    /// Map to keep track of which modules to notify
    notify_map: HashMap<&'static str, HashSet<IdType>>,
    /// List of signals staged for delivery to components
    delivery_signals: Vec<NerveSignal<O>>,
    /// List of signals staged for delivery to components (thread safe)
    delivery_signals_sync: Vec<Arc<Mutex<NerveSignal<O>>>>,
    /// Signal transformers on given nerve segments
    transforms:
        HashMap<O::NerveType, HashMap<TypeId, HashMap<IdType, Box<dyn NerveSignalTransformer>>>>,
    /// Pending notifies
    pending_signals: BTreeMap<OrderedTime, Vec<NerveSignal<O>>>,
    /// Internal trigger id to unschedule if needed
    internal_trigger_id: Option<IdType>,
}

impl<O: Organism> NervousLayer<O> {
    pub fn new() -> Self {
        Self {
            id_gen: IdGenerator::new(),
            signal_notifies: HashMap::new(),
            notify_map: HashMap::new(),
            delivery_signals: Vec::new(),
            delivery_signals_sync: Vec::new(),
            transforms: HashMap::new(),
            pending_signals: BTreeMap::new(),
            internal_trigger_id: None,
        }
    }

    fn add_transforms(
        &mut self,
        registered_transforms: &mut HashMap<O::NerveType, HashMap<TypeId, IdType>>,
        new_transforms: impl Iterator<Item = (
            O::NerveType,
            HashMap<TypeId, Box<dyn NerveSignalTransformer>>
        )>,
    ) {
        for (nerve, mut type_map) in new_transforms {
            for (type_id, transformer) in type_map.drain() {
                let transform_id = self.id_gen.get_id();
                registered_transforms
                    .entry(nerve)
                    .or_default()
                    .insert(type_id, transform_id);

                self.transforms
                    .entry(nerve)
                    .or_default()
                    .entry(type_id)
                    .or_default()
                    .insert(transform_id, transformer);
            }
        }
    }

    fn prepare_connector(&mut self, connector: &mut SimConnector, component: &mut impl NervousComponent<O>) -> HashSet<u32> {
        component.nervous_connector().sim_time = connector.sim_time();

        self
            .notify_map
            .remove(component.id())
            .expect("missing component signals")
    }

    fn process_connector(&mut self, _connector: &mut SimConnector, component: &mut impl NervousComponent<O>) {
        let n_connector = component.nervous_connector();

        // Remove any transforms staged for removal
        for (nerve, mut type_map) in n_connector.removing_transforms.drain() {
            for (type_id, transform_id) in type_map.drain() {
                self.transforms
                    .entry(nerve)
                    .or_default()
                    .entry(type_id)
                    .or_default()
                    .remove(&transform_id);
            }
        }

        // Add any newly registered transforms
        self.add_transforms(
            &mut n_connector.registered_transforms,
            n_connector.adding_transforms.drain()
        );

        // Add any new signals
        for signal in n_connector.outgoing.drain(..) {
            let signal_time = OrderedTime(signal.send_time());
            self.pending_signals
                .entry(signal_time)
                .or_default()
                .push(signal);
        }
    }
}

impl<O: Organism> SimLayer for NervousLayer<O> {
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        let otime = OrderedTime(connector.sim_time());

        if let Some(id) = self.internal_trigger_id.take() {
            let _ = connector.time_manager.unschedule_event(&id);
        }

        // Do this for all sim times up to the present
        while self
            .pending_signals
            .first_key_value()
            .is_some_and(|(t, _)| t <= &otime)
        {
            let (_, mut signals) = self.pending_signals.pop_first().unwrap();
            if !signals.is_empty() {
                'sigloop: for signal in signals.iter_mut() {
                    for nerve in signal.neural_path().collect::<Vec<_>>().iter() {

                        // Apply any transformations
                        if let Some(fn_map) = self.transforms.get_mut(&nerve) {
                            if let Some(transform_list) = fn_map.get_mut(&signal.message_type_id()) {
                                for (_, transform_box) in transform_list.iter_mut() {
                                    if None == transform_box.transform(signal.dyn_message_mut()) {
                                        continue 'sigloop;
                                    }
                                }
                            }
                        }

                        // Determine which components need to be triggered
                        if let Some(id_map) = self.signal_notifies.get(&nerve) {
                            if let Some(comp_ids) = id_map.get(&signal.message_type_id()) {
                                for cid in comp_ids {
                                    self.notify_map.entry(cid).or_default().insert(signal.id());
                                }
                            }
                        }
                    }
                }

                // Stage the signals for delivery
                self.delivery_signals.append(&mut signals);
            }
        }
    }

    fn post_exec(&mut self, connector: &mut SimConnector) {
        if let Some(min_time) = self.pending_signals.keys().min() {
            let mut delay = secs!(0.0);
            if min_time.0 > connector.sim_time() {
                delay = min_time.0 - connector.sim_time();
            }
            let id = connector
                .time_manager
                .schedule_event(delay, Box::new(InternalLayerTrigger));
            self.internal_trigger_id = Some(id);
        }
    }

}

impl<O: Organism> SimLayerSync for NervousLayer<O> {
    fn pre_exec_sync(&mut self, connector: &mut SimConnector) {
        self.pre_exec(connector);

        self.delivery_signals_sync.extend(
            self.delivery_signals
            .drain(..)
            .map(|s| {
                Arc::new(Mutex::new(s))
            })
        );
    }

    fn post_exec_sync(&mut self, connector: &mut SimConnector) {
        self.post_exec(connector)
    }
}

impl<O: Organism, T: NervousComponent<O>> SimComponentProcessor<O, T> for NervousLayer<O> {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = NervousInitializer::new();
        component.nervous_init(&mut initializer);

        for (nerve, type_ids) in initializer.signal_notifies.into_iter() {
            for type_id in type_ids {
                self.signal_notifies
                    .entry(nerve)
                    .or_default()
                    .entry(type_id)
                    .or_default()
                    .insert(component.id());
            }
        }

        // Add any initial transformations
        self.add_transforms(
            &mut component.nervous_connector().registered_transforms,
            initializer.adding_transforms.drain()
        );
    }

    fn check_component(&mut self, component: &T) -> bool {
        self.notify_map.contains_key(component.id())
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let incoming = self.prepare_connector(connector, component);

        // partition the delivery_signals vector to extract the ones which
        // apply to this component only
        let (incoming_signals, others) = self
            .delivery_signals
            .drain(..)
            .partition(|s| incoming.contains(&s.id()));

        // Make sure to keep the rest around so they're not lost
        self.delivery_signals = others;

        // Add the incoming signals to the component's connector
        for signal in incoming_signals {
            component.nervous_connector()
                .incoming
                .entry(signal.message_type_id())
                .or_default()
                .push(signal);
        }
    }

    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_connector(connector, component);

        // Move the signals back into our signal list in case other components need them
        for (_, mut signals) in component.nervous_connector().incoming.drain() {
            self.delivery_signals.append(&mut signals);
        }
    }

}

impl<O: Organism, T: NervousComponent<O>> SimComponentProcessorSync<O, T> for NervousLayer<O> {
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.setup_component(connector, component)
    }

    fn check_component_sync(&mut self, component: &T) -> bool {
        self.check_component(component)
    }

    fn prepare_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        let incoming = self.prepare_connector(connector, component);

        // partition the delivery_signals vector to extract the ones which
        // apply to this component only
        let (incoming_signals, _): (Vec<&Arc<Mutex<NerveSignal<O>>>>, Vec<_>) = self
            .delivery_signals_sync
            .iter()
            .partition(|s| incoming.contains(&s.lock().unwrap().id()));

        // Add the incoming signals to the component's connector
        for signal in incoming_signals {
            component.nervous_connector()
                .incoming_sync
                .entry(signal.lock().unwrap().message_type_id())
                .or_default()
                .push(signal.clone());
        }
    }

    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_connector(connector, component);

        // Drop the incoming signal references
        component.nervous_connector().incoming_sync.clear();
    }
}
