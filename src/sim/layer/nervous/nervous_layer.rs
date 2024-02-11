use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::swap;

use crate::sim::component::SimComponentProcessor;
use crate::sim::layer::{InternalLayerTrigger, SimLayer};
use crate::sim::organism::Organism;
use crate::sim::SimConnector;
use crate::util::{secs, IdGenerator, IdType, OrderedTime};

use super::component::{NervousComponent, NervousInitializer};
use super::nerve::NerveSignal;
use super::NerveSignalTransformer;

pub struct NervousLayer<O: Organism> {
    /// ID generator for transform registration
    id_gen: IdGenerator,
    /// Map to keep track of which modules to notify for certain signals
    signal_notifies: HashMap<O::NerveType, HashMap<TypeId, HashSet<&'static str>>>,
    /// Map to keep track of which modules to notify
    notify_map: HashMap<&'static str, HashSet<IdType>>,
    /// List of signals staged for delivery to components
    delivery_signals: Vec<NerveSignal<O>>,
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
            transforms: HashMap::new(),
            pending_signals: BTreeMap::new(),
            internal_trigger_id: None,
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
                // Get the TypeId for these signals
                let type_id = signals.get(0).unwrap().type_id();

                // Determine which components need to be triggered
                for signal in signals.iter().filter(|s| !s.is_blocked()) {
                    for nerve in signal.neural_path() {
                        if let Some(id_map) = self.signal_notifies.get(&nerve) {
                            if let Some(comp_ids) = id_map.get(&type_id) {
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
    }

    fn check_component(&mut self, component: &T) -> bool {
        self.notify_map.contains_key(component.id())
    }

    fn prepare_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        let incoming = self
            .notify_map
            .remove(component.id())
            .expect("missing component signals");
        let n_connector = component.nervous_connector();

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
            n_connector
                .incoming
                .entry(signal.type_id())
                .or_default()
                .push(signal);
        }

        // Add some other connector things before the component run
        n_connector.sim_time = connector.sim_time();
        swap(&mut n_connector.pending_signals, &mut self.pending_signals);
        swap(&mut n_connector.transforms, &mut self.transforms);
    }

    fn process_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let n_connector = component.nervous_connector();

        // Make sure to swap these back in
        swap(&mut n_connector.pending_signals, &mut self.pending_signals);
        swap(&mut n_connector.transforms, &mut self.transforms);

        // Move the signals back into our signal list in case other components need them
        for (_, mut signals) in n_connector.incoming.drain() {
            self.delivery_signals.append(&mut signals);
        }

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
        for (nerve, mut type_map) in n_connector.adding_transforms.drain() {
            for (type_id, transformer) in type_map.drain() {
                let transform_id = self.id_gen.get_id();
                n_connector
                    .registered_transforms
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
