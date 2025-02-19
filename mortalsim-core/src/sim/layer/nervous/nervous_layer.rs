use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::swap;
use std::sync::{Arc, Mutex};

use downcast_rs::Downcast;

use crate::sim::component::{SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::{InternalLayerTrigger, SimLayer, SimLayerSync};
use crate::sim::organism::Organism;
use crate::sim::time_manager::ScheduleId;
use crate::sim::SimConnector;
use crate::{secs, IdGenerator, IdType, SimTime, SimTimeSpan};

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
    /// Signal transformers on given nerve segments
    transforms:
        HashMap<O::NerveType, HashMap<TypeId, HashMap<IdType, Box<dyn NerveSignalTransformer>>>>,
    /// Pending notifies
    pending_signals: BTreeMap<SimTime, Vec<NerveSignal<O>>>,
    /// Internal trigger id to unschedule if needed
    internal_trigger_id: Option<ScheduleId>,
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

    /// Add new transforms to the registered_transforms map
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
                log::debug!("Adding transform on nerve {:?} for type {:?}. ID: {}", nerve, type_id, transform_id);
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

    /// Remove any signals in the given iterator of (emit_time, signal_id).
    fn remove_signals(&mut self, items: impl Iterator<Item = (SimTime, IdType)>) {
        for (signal_time, signal_id) in items {
            if let Some(mut signal_ids) = self.pending_signals.remove(&signal_time) {
                if let Some(idx) = signal_ids.iter().position(|x| x.id() == signal_id) {
                    log::debug!("removing nerve signal {} at time {}", signal_id, signal_time);
                    signal_ids.remove(idx);
                    // Reinsert as long as there are other signals remaining
                    if signal_ids.len() > 0 {
                        log::debug!("There are still other signals scheduled for time {}", signal_time);
                        self.pending_signals.insert(signal_time, signal_ids);
                    }
                }
            }
        }
    }

    /// Removing any transforms in the given iterator of (nerve, Map<signal_type, id>).
    fn remove_transforms(&mut self, items: impl Iterator<Item = (O::NerveType, HashMap<TypeId, IdType>)>) {
        for (nerve, mut type_map) in items {
            for (type_id, transform_id) in type_map.drain() {
                log::debug!("Removing transform {} from nerve {:?} for signal type {:?}", transform_id, nerve, type_id);
                self.transforms
                    .entry(nerve)
                    .or_default()
                    .entry(type_id)
                    .or_default()
                    .remove(&transform_id);
            }
        }
    }

    fn prepare_connector(&mut self, connector: &mut SimConnector, component: &mut (impl NervousComponent<O> + ?Sized)) -> HashSet<u32> {
        component.nervous_connector().sim_time = connector.sim_time();

        self
            .notify_map
            .remove(component.id())
            .unwrap_or_default()
    }

    fn process_connector(&mut self, _connector: &mut SimConnector, component: &mut (impl NervousComponent<O> + ?Sized)) {
        let n_connector = component.nervous_connector();

        // Remove any signals staged for removal
        self.remove_signals(n_connector.pending_unschedules.drain(..));

        // Remove any transforms staged for removal
        self.remove_transforms(n_connector.removing_transforms.drain());

        // Add any newly registered transforms
        self.add_transforms(
            &mut n_connector.registered_transforms,
            n_connector.adding_transforms.drain()
        );

        // Add any new signals
        for signal in n_connector.outgoing.drain(..) {
            let signal_time = signal.send_time();
            self.pending_signals
                .entry(signal_time)
                .or_default()
                .push(signal);
        }
    }
}

impl<O: Organism> SimLayer for NervousLayer<O> {
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        let otime = connector.sim_time();

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
                                for (transform_id, transform_box) in transform_list.iter_mut() {
                                    // Note the dyn_message_mut call will panic if there are multiple
                                    // references to the inner message. But at this point there
                                    // should always only be a single reference in operation
                                    log::debug!(
                                        "Calling transform {} for nerve signal {:?}",
                                        transform_id,
                                        signal.dyn_message()
                                    );
                                    if transform_box.transform(signal.dyn_message_mut()).is_none() {
                                        log::debug!("Cancelling nerve signal {:?}", signal.dyn_message());
                                        continue 'sigloop;
                                    }
                                }
                            }
                        }

                        // Determine which components need to be triggered
                        if let Some(id_map) = self.signal_notifies.get(&nerve) {
                            if let Some(comp_ids) = id_map.get(&signal.message_type_id()) {
                                for cid in comp_ids {
                                    log::debug!("Preparing to trigger {} for nerve signal {}", cid, signal.id());
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
            let mut delay = SimTimeSpan::from_s(0.0);
            if *min_time > connector.sim_time() {
                delay = connector.sim_time().span_to(min_time);
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
    }

    fn post_exec_sync(&mut self, connector: &mut SimConnector) {
        self.post_exec(connector)
    }
}

impl<O: Organism, T: NervousComponent<O> + ?Sized> SimComponentProcessor<O, T> for NervousLayer<O> {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = NervousInitializer::new();
        component.nervous_init(&mut initializer);

        for (nerve, type_ids) in initializer.signal_notifies.into_iter() {
            for type_id in type_ids {
                log::debug!(
                    "Adding nerve notification for {:?} with signal type {:?} for component {}",
                    nerve,
                    type_id,
                    component.id(),
                );
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
            log::trace!("Prepping nerve signal {} for component {}", signal.id(), component.id());
            component.nervous_connector()
                .incoming
                .entry(signal.message_type_id())
                .or_default()
                .push(signal);
        }

        // Update sim time
        component.nervous_connector().sim_time = connector.sim_time();

        // Trim down the scheduled signals to remove any that have already passed
        component.nervous_connector().scheduled_signals.retain(|_, time| *time > connector.sim_time());
    }

    fn process_component(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_connector(connector, component);

        // Move the signals back into our signal list in case other components need them
        for (_, mut signals) in component.nervous_connector().incoming.drain() {
            self.delivery_signals.append(&mut signals);
        }
    }

    fn remove_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let n_connector = component.nervous_connector();
        self.remove_signals(n_connector.scheduled_signals.drain().map(|(t,i)| (i,t)));
        self.remove_transforms(n_connector.registered_transforms.drain());
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
        let (incoming_signals, _): (Vec<&NerveSignal<O>>, Vec<_>) = self
            .delivery_signals
            .iter()
            .partition(|s| incoming.contains(&s.id()));

        // Clone the incoming signals to the component's connector (as opposed to moving them)
        for signal in incoming_signals {
            log::trace!("Prepping nerve signal {} for component {}", signal.id(), component.id());
            component.nervous_connector()
                .incoming
                .entry(signal.message_type_id())
                .or_default()
                .push(signal.clone());
        }
    }

    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_connector(connector, component);

        // Drop the incoming signal references
        component.nervous_connector().incoming.clear();
    }

    fn remove_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.remove_component(connector, component)
    }
}


pub mod test {
    use std::os::windows::process;
    use std::sync::Mutex;
    use std::thread::scope;

    use crate::event::test::TestEventA;
    use crate::sim::component::{SimComponent, SimComponentProcessor};
    use crate::sim::layer::nervous::component::test::{MovementEvent, PainEvent, TestMovementComponent, TestPainReflexComponent, TestPainkillerComponent};
    use crate::sim::layer::nervous::{NervousComponent, NervousLayer};
    use crate::sim::layer::{SimLayer, SimLayerSync};
    use crate::sim::organism::test::TestOrganism;
    use crate::sim::{Organism, SimConnector, SimTime};
    use crate::SimTimeSpan;


    fn process_components<O: Organism>(layer: &mut NervousLayer<O>, connector: &mut SimConnector, components: &mut Vec<Box<dyn NervousComponent<O>>>) {
        layer.pre_exec(connector);

        for component in components.iter_mut() {
            layer.prepare_component(connector, component.as_mut());
            component.run();
            layer.process_component(connector, component.as_mut());
        }

        layer.post_exec(connector);
    }

    fn process_components_sync<O: Organism>(layer: &Mutex<NervousLayer<O>>, connector: &Mutex<SimConnector>, components: &mut Vec<Box<dyn NervousComponent<O>>>) {
        layer.lock().unwrap().pre_exec_sync(&mut *connector.lock().unwrap());

        scope(|s| {
            for component in components.iter_mut() {
                s.spawn(|| {
                    layer.lock().unwrap().prepare_component(&mut *connector.lock().unwrap(), component.as_mut());
                    component.run();
                    layer.lock().unwrap().process_component(&mut *connector.lock().unwrap(), component.as_mut());
                });
            }
        });

        layer.lock().unwrap().post_exec_sync(&mut *connector.lock().unwrap());
    }

    #[test]
    fn layer_process() {
        let mut layer = NervousLayer::<TestOrganism>::new();
        let mut connector = SimConnector::new();

        let mut components: Vec<Box<dyn NervousComponent<TestOrganism>>> = vec![
            Box::new(TestPainReflexComponent::new()),
            Box::new(TestMovementComponent::new()),
        ];

        for component in components.iter_mut() {
            layer.setup_component(&mut connector, component.as_mut());
        }

        process_components(&mut layer, &mut connector, &mut components);

        // We should have four pain events ready to dispatch in the future
        assert!(layer.pending_signals.len() == 4);
        assert!(layer.pending_signals.values().all(|x| x.get(0).is_some_and(|s| s.message_is::<PainEvent>())));

        println!("{:?}", layer.pending_signals.keys());

        connector.time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components(&mut layer, &mut connector, &mut components);

        // We should have one reflex movement pending
        assert!(layer.pending_signals.len() == 4);
        assert!(layer.pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        connector.time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components(&mut layer, &mut connector, &mut components);

        // Reflex event should have completed (no longer in pending)
        assert!(layer.pending_signals.len() == 3);
        assert!(!layer.pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        connector.time_manager.advance_by(SimTimeSpan::from_s(4.0));
        process_components(&mut layer, &mut connector, &mut components);

        // We should only have one signal since the second PainEvent shouldn't evoke a movement
        assert!(layer.pending_signals.len() == 2);
        
        connector.time_manager.advance_by(SimTimeSpan::from_s(5.0));
        process_components(&mut layer, &mut connector, &mut components);

        // We should have one additional movement response signal, and one last pain signal
        assert!(layer.pending_signals.len() == 2);
        assert!(layer.pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        // Lets add the painkiller component
        println!("Gettin' some painkillers!");
        let mut painkiller_component = TestPainkillerComponent::new();
        layer.setup_component(&mut connector, &mut painkiller_component);
        components.push(Box::new(painkiller_component));

        connector.time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components(&mut layer, &mut connector, &mut components);

        // The pain event should have been transformed so it shouldn't evoke
        // any new movement response. i.e. pending signals should be zero
        assert!(layer.pending_signals.len() == 0);

    }

    #[test]
    fn layer_process_sync() {
        let layer = Mutex::new(NervousLayer::<TestOrganism>::new());
        let connector = Mutex::new(SimConnector::new());

        let mut components: Vec<Box<dyn NervousComponent<TestOrganism>>> = vec![
            Box::new(TestPainReflexComponent::new()),
            Box::new(TestMovementComponent::new()),
        ];

        for component in components.iter_mut() {
            layer.lock().unwrap().setup_component(&mut *connector.lock().unwrap(), component.as_mut());
        }

        process_components_sync(&layer, &connector, &mut components);

        // We should have four pain events ready to dispatch in the future
        assert!(layer.lock().unwrap().pending_signals.len() == 4);
        assert!(layer.lock().unwrap().pending_signals.values().all(|x| x.get(0).is_some_and(|s| s.message_is::<PainEvent>())));

        println!("{:?}", layer.lock().unwrap().pending_signals.keys());

        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components_sync(&layer, &connector, &mut components);

        // We should have one reflex movement pending
        assert!(layer.lock().unwrap().pending_signals.len() == 4);
        assert!(layer.lock().unwrap().pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components_sync(&layer, &connector, &mut components);

        // Reflex event should have completed (no longer in pending)
        assert!(layer.lock().unwrap().pending_signals.len() == 3);
        assert!(!layer.lock().unwrap().pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(4.0));
        process_components_sync(&layer, &connector, &mut components);

        // We should only have one signal since the second PainEvent shouldn't evoke a movement
        assert!(layer.lock().unwrap().pending_signals.len() == 2);
        
        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(5.0));
        process_components_sync(&layer, &connector, &mut components);

        // We should have one additional movement response signal, and one last pain signal
        assert!(layer.lock().unwrap().pending_signals.len() == 2);
        assert!(layer.lock().unwrap().pending_signals.values().any(|x| x.get(0).is_some_and(|s| s.message_is::<MovementEvent>())));

        // Lets add the painkiller component
        println!("Gettin' some painkillers!");
        let mut painkiller_component = TestPainkillerComponent::new();
        layer.lock().unwrap().setup_component(&mut *connector.lock().unwrap(), &mut painkiller_component);
        components.push(Box::new(painkiller_component));

        connector.lock().unwrap().time_manager.advance_by(SimTimeSpan::from_s(1.0));
        process_components_sync(&layer, &connector, &mut components);

        // The pain event should have been transformed so it shouldn't evoke
        // any new movement response. i.e. pending signals should be zero
        assert!(layer.lock().unwrap().pending_signals.len() == 0);

    }
}