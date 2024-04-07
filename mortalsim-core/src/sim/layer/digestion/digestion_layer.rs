use crate::sim::component::{SimComponent, SimComponentProcessor, SimComponentProcessorSync};
use crate::sim::layer::{InternalLayerTrigger, SimLayer, SimLayerSync};
use crate::sim::organism::Organism;
use crate::sim::{SimConnector, SimTime};
use crate::{secs, IdType};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::marker::PhantomData;

use super::component::{DigestionComponent, DigestionInitializer};
use super::consumable::Consumable;
use super::consumed::Consumed;
use super::{ConsumeEvent, DigestionDirection, EliminateEvent};

type ConsumableId = IdType;

pub struct DigestionLayer<O: Organism> {
    pd: PhantomData<O>,
    /// Default duration each component receives a consumable for
    default_digestion_duration: SimTime,
    /// Tracks the order in which substance stores pass
    /// through each component, according to the order
    /// they were added
    component_map: HashMap<&'static str, usize>,
    /// Keeps track of component indices which should
    /// trigger due to a new consumable coming in.
    trigger_map: HashSet<usize>,
    /// Map to track stores in between components
    consumed_map: Vec<Vec<Consumed>>,
    /// Consumables staged for elimination
    elimination_list: Vec<(Consumable, DigestionDirection)>,
    /// Internal trigger id to unschedule if needed
    internal_trigger_id: Option<IdType>,
}

impl<O: Organism> DigestionLayer<O> {
    // Delay between elimination discovery and execution
    const ELIMINATION_DELAY: SimTime = SimTime {s: 0.0};

    /// Creates a Sim with the default set of modules which is equal to all registered
    /// modules at the time of execution.
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            default_digestion_duration: secs!(60.0),
            component_map: HashMap::new(),
            trigger_map: HashSet::new(),
            consumed_map: Vec::new(),
            elimination_list: Vec::new(),
            internal_trigger_id: None,
        }
    }

    /// Consume a new SubstanceStore
    fn consume(&mut self, consumable: Consumable) {
        let consumed = Consumed::new(consumable);
        if let Some(list) = self.consumed_map.get_mut(0) {
            list.push(consumed);
        }
    }

    // Internal method for retrieving the position of a component
    // in the digestive tract
    fn component_position<T: SimComponent<O>>(&self, component: &T) -> usize {
        *self
            .component_map
            .get(component.id())
            .expect("Digestion component position is missing!")
    }
}

impl<O: Organism> SimLayer for DigestionLayer<O> {
    fn pre_exec(&mut self, connector: &mut SimConnector) {
        if let Some(id) = self.internal_trigger_id.take() {
            // Ignore if it's an Err. This just tries to unschedule
            // if the scheduled event hasn't already passed.
            connector.time_manager.unschedule_event(&id).ok();
        }

        for evt in connector.active_events.iter() {
            if let Some(consume_evt) = evt.downcast_ref::<ConsumeEvent>() {
                self.consume(consume_evt.0.clone());
            }
        }
        // Keep track of vector indices of items which need to move
        let mut moving_indices: Vec<Vec<usize>> = vec![vec![]; self.consumed_map.len()];
        for (pos, consumed_list) in self.consumed_map.iter_mut().enumerate() {
            for (idx, consumed) in consumed_list.iter_mut().enumerate() {
                // advance time for the consumable
                consumed.advance(connector.sim_time());
                // if time has exceeded the exit time, stage it for movement
                if consumed.exit_time <= connector.sim_time() {
                    moving_indices
                        .get_mut(pos)
                        .expect("moving_indices initialized improperly")
                        .push(idx);
                }
            }
        }
        // position of the last digestion component
        let last = {
            if self.consumed_map.len() > 0 {
                self.consumed_map.len() - 1
            } else {
                0
            }
        };

        for (pos, indices) in moving_indices.into_iter().enumerate() {
            for idx in indices {
                let mut removed = self
                    .consumed_map
                    .get_mut(pos)
                    .expect("moving_indices referenced invalid position")
                    .remove(idx);

                // Check cases for elimination, either forward or backward
                if (pos == 0 && removed.exit_direction == DigestionDirection::BACK)
                    || (pos >= last && removed.exit_direction == DigestionDirection::FORWARD)
                {
                    let evt = Box::new(EliminateEvent::new(
                        removed.consumable,
                        removed.exit_direction,
                    ));
                    connector.time_manager.schedule_event(Self::ELIMINATION_DELAY, evt);
                    continue;
                }
                
                // update sim time
                removed.sim_time = connector.sim_time();

                // update entry time
                removed.entry_time = removed.exit_time;

                // set defaults, which the component may override
                removed.exit_time = removed.entry_time + self.default_digestion_duration;
                
                match removed.exit_direction {
                    DigestionDirection::FORWARD => {
                        let target_idx = pos + 1;
                        self.consumed_map
                            .get_mut(target_idx)
                            .expect("invalid index")
                            .push(removed);
                        self.trigger_map.insert(target_idx);
                    }
                    DigestionDirection::BACK => {
                        // Always default to FORWARD, even if it was previously BACK
                        removed.exit_direction = DigestionDirection::FORWARD;
                        let target_idx = pos - 1;
                        self.consumed_map
                            .get_mut(target_idx)
                            .expect("invalid index")
                            .push(removed);
                        self.trigger_map.insert(target_idx);
                    }
                    DigestionDirection::EXHAUSTED => {
                        // Drop the removed consumable completely
                    }
                }
            }
        }
    }

    fn post_exec(&mut self, connector: &mut SimConnector) {
        if let Some(min_consumed) = self
            .consumed_map
            .iter()
            .flatten()
            .min_by(|a, b| a.exit_time.partial_cmp(&b.exit_time).unwrap())
        {
            let mut delay = secs!(0.0);
            if min_consumed.exit_time > connector.sim_time() {
                delay = min_consumed.exit_time - connector.sim_time();
            }
            let id = connector
                .time_manager
                .schedule_event(delay, Box::new(InternalLayerTrigger));
            self.internal_trigger_id = Some(id);
        }
    }

}

impl<O: Organism> SimLayerSync for DigestionLayer<O> {
    fn pre_exec_sync(&mut self, connector: &mut SimConnector) {
        self.pre_exec(connector)
    }

    fn post_exec_sync(&mut self, connector: &mut SimConnector) {
        self.post_exec(connector)
    }
}

impl<O: Organism, T: DigestionComponent<O>> SimComponentProcessor<O, T> for DigestionLayer<O> {
    fn setup_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let mut initializer = DigestionInitializer::new();
        component.digestion_init(&mut initializer);

        self.component_map
            .insert(component.id(), self.component_map.len());

        if self.consumed_map.len() < self.component_map.len() {
            self.consumed_map.push(Vec::new());
        }
    }

    fn check_component(&mut self, component: &T) -> bool {
        let component_pos = self.component_position(component);
        self.trigger_map.contains(&component_pos)
    }

    fn prepare_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_pos = self.component_position(component);

        // move consumed items from the layer map into the component connector
        let consumed_list = self.consumed_map.get_mut(component_pos).unwrap();

        if component.digestion_connector().unschedule_all {
            consumed_list.iter_mut().for_each(|c| c.clear_all_changes());
        }

        component
            .digestion_connector()
            .consumed_list
            .extend(consumed_list.drain(..));
    }

    fn process_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_pos = self.component_position(component);

        // move consumed items from the component connector back into the layer map
        let consumed_list = &mut component.digestion_connector().consumed_list;
        self.consumed_map
            .get_mut(component_pos)
            .unwrap()
            .extend(consumed_list.drain(..));

        // Reset the trigger
        self.trigger_map.remove(&component_pos);
    }

    fn remove_component(&mut self, _connector: &mut SimConnector, component: &mut T) {
        let component_idx = self.component_map.remove(component.id())
            .expect(format!("component index is missing for '{:?}'!", component.id()).as_str());
        self.consumed_map.remove(component_idx);
    }

}

// We can do the same thing here as the non-threaded version, since all
// consumed items are effectively owned by only one component at a time.
// no sharing needed.
impl<O: Organism, T: DigestionComponent<O>> SimComponentProcessorSync<O, T> for DigestionLayer<O> {
    fn setup_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.setup_component(connector, component)
    }

    fn check_component_sync(&mut self, component: &T) -> bool {
        self.check_component(component)
    }

    fn prepare_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.prepare_component(connector, component)
    }

    fn process_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.process_component(connector, component)
    }

    fn remove_component_sync(&mut self, connector: &mut SimConnector, component: &mut T) {
        self.remove_component(connector, component)
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::BorrowMut, ops::RangeBounds, sync::{Arc, Mutex}, thread::scope};

    use crate::{sim::{component::{SimComponent, SimComponentProcessor, SimComponentProcessorSync}, layer::{digestion::{component::test::TestDigestionComponent, consumable::test::{test_ammonia, test_fiber, test_food}, ConsumeEvent, DigestionComponent, DigestionDirection, EliminateEvent}, InternalLayerTrigger, SimLayer}, organism::test::TestOrganism, Organism, SimConnector, SimTime}, substance::{Substance, SubstanceConcentration}, util::secs};

    use super::DigestionLayer;


    #[test]
    fn layer() {
        DigestionLayer::<TestOrganism>::new();
    }

    fn run_layer<O: Organism>(layer: &mut DigestionLayer<O>, connector: &mut SimConnector, components: &mut Vec<TestDigestionComponent<O>>) {
        for component in components.iter_mut() {
            layer.prepare_component(connector, component);
            component.run();
            layer.process_component(connector, component);
        }
    }

    fn run_layer_sync<O: Organism>(layer: &Mutex<DigestionLayer<O>>, connector: &Mutex<SimConnector>, components: &mut Vec<TestDigestionComponent<O>>) {
        scope(|s| {
            for component in components.iter_mut() {
                s.spawn(|| {
                    layer.lock().unwrap().prepare_component_sync(&mut connector.lock().unwrap(), component);
                    component.run();
                    layer.lock().unwrap().process_component_sync(&mut connector.lock().unwrap(), component);
                });
            }
        });
    }

    #[test]
    fn layer_process() {
        let mut layer = DigestionLayer::<TestOrganism>::new();
        let mut components = vec![
            TestDigestionComponent::new(),
            TestDigestionComponent::new(),
        ];
        let mut connector = SimConnector::new();
        for component in components.iter_mut() {
            layer.setup_component(&mut connector, component);
        }

        let food = test_food(200.0);
        let starting_glc = food.concentration_of(&Substance::GLC);

        connector.active_events.push(Arc::new(ConsumeEvent(food)));
        connector.active_events.push(Arc::new(ConsumeEvent(test_ammonia(50.0))));
        connector.active_events.push(Arc::new(ConsumeEvent(test_fiber(150.0))));

        // Initial run (nothing should happen)
        run_layer(&mut layer, &mut connector, &mut components);

        // First pre_exec should pull in the 3 ConsumeEvents
        layer.pre_exec(&mut connector);
        assert!(layer.consumed_map.get(0).is_some_and(|x| x.len() == 3));

        run_layer(&mut layer, &mut connector, &mut components);

        layer.post_exec(&mut connector);

        // Make sure to drain the active events so they don't keep
        // getting added
        connector.active_events.drain(..);

        connector.time_manager.advance_by(SimTime::from_s(10.0));

        // Should schedule an internal trigger event
        let mut next_events = connector.time_manager.next_events();
        assert!(next_events.next().unwrap().1.get(0).unwrap().is::<InternalLayerTrigger>());

        // Shouldn't see any EliminateEvents yet
        assert!(next_events.next().is_none());

        layer.pre_exec(&mut connector);
        run_layer(&mut layer, &mut connector, &mut components);
        layer.post_exec(&mut connector);

        connector.time_manager.advance_by(SimTime::from_s(2.0));

        // Should be eliminating the ammonia
        let (exec_time, mut evts) = connector.time_manager.next_events().next().unwrap();
        let expected_time = SimTime::from_s(10.0) + DigestionLayer::<TestOrganism>::ELIMINATION_DELAY;
        let threshold = secs!(0.1);
        assert!(
            (exec_time-threshold..exec_time+threshold).contains(&expected_time),
            "Expected time {} != {}",
            expected_time,
            exec_time
        );

        assert!(evts.get(0).is_some());
        assert!(evts.get(0).unwrap().is::<EliminateEvent>());

        let elim = evts.pop().unwrap().downcast::<EliminateEvent>().unwrap();
        assert_eq!(elim.direction, DigestionDirection::BACK);
        assert!(elim.excrement.concentration_of(&Substance::NH3) > SubstanceConcentration::from_mM(0.0));

        layer.pre_exec(&mut connector);


        // Both of the remaining consumables should still be with the first component
        assert_eq!(layer.consumed_map.get(0).unwrap().len(), 2);
        
        // Should be consuming some sugar
        assert!(layer.consumed_map.get(0).unwrap().get(0).unwrap().concentration_of(&Substance::GLC) < starting_glc);
        
        // Exec for completeness (shouldn't change anything)
        run_layer(&mut layer, &mut connector, &mut components);
        layer.post_exec(&mut connector);

        connector.time_manager.advance_by(SimTime::from_min(1.0));
        layer.pre_exec(&mut connector);

        // First component should still have the food
        assert_eq!(layer.consumed_map.get(0).unwrap().len(), 1);

        // Fiber should have moved to the next component
        assert_eq!(layer.consumed_map.get(1).unwrap().len(), 1);
        assert!(layer.consumed_map.get(1).unwrap().get(0).unwrap().concentration_of(&Substance::Cellulose) > SubstanceConcentration::from_mM(0.0));

        run_layer(&mut layer, &mut connector, &mut components);
        layer.post_exec(&mut connector);

        // each component should have 1 consumed
        assert_eq!(layer.consumed_map.get(0).unwrap().len(), 1);
        assert_eq!(layer.consumed_map.get(1).unwrap().len(), 1);

        // Go through a cycle
        connector.time_manager.advance_by(SimTime::from_min(5.0));
        layer.pre_exec(&mut connector);
        run_layer(&mut layer, &mut connector, &mut components);
        layer.post_exec(&mut connector);
        
        // Each list should be empty now
        assert_eq!(layer.consumed_map.get(0).unwrap().len(), 0);
        assert_eq!(layer.consumed_map.get(1).unwrap().len(), 0);

        connector.time_manager.advance_by(SimTime::from_s(1.0));

        // Should be eliminating the fiber
        let (_, mut evts) = connector.time_manager.next_events().next().unwrap();

        assert!(evts.get(0).is_some());
        assert!(evts.get(0).unwrap().is::<EliminateEvent>());

        let elim = evts.pop().unwrap().downcast::<EliminateEvent>().unwrap();
        assert_eq!(elim.direction, DigestionDirection::FORWARD);
        assert!(elim.excrement.concentration_of(&Substance::Cellulose) > SubstanceConcentration::from_mM(0.0));

        // Food should have dissappeared
        assert!(layer.consumed_map.get(0).unwrap().is_empty());
        assert!(layer.consumed_map.get(1).unwrap().is_empty());

    }

    #[test]
    fn layer_process_sync() {
        let layer = Mutex::new(DigestionLayer::<TestOrganism>::new());
        let connector = Mutex::new(SimConnector::new());
        let mut components = vec![
            TestDigestionComponent::new(),
            TestDigestionComponent::new(),
        ];
        for component in components.iter_mut() {
            layer.lock().unwrap().setup_component_sync(&mut connector.lock().unwrap(), component);
        }

        let food = test_food(200.0);
        let starting_glc = food.concentration_of(&Substance::GLC);

        connector.lock().unwrap().active_events.push(Arc::new(ConsumeEvent(food)));
        connector.lock().unwrap().active_events.push(Arc::new(ConsumeEvent(test_ammonia(50.0))));
        connector.lock().unwrap().active_events.push(Arc::new(ConsumeEvent(test_fiber(150.0))));

        // Initial run (nothing should happen)
        run_layer_sync(&layer, &connector, &mut components);

        // First pre_exec should pull in the 3 ConsumeEvents
        layer.lock().unwrap().pre_exec(&mut connector.lock().unwrap());
        assert!(layer.lock().unwrap().consumed_map.get(0).is_some_and(|x| x.len() == 3));

        run_layer_sync(&layer, &connector, &mut components);

        layer.lock().unwrap().post_exec(&mut connector.lock().unwrap());

        // Make sure to drain the active events so they don't keep
        // getting added
        connector.lock().unwrap().active_events.drain(..);

        connector.lock().unwrap().time_manager.advance_by(SimTime::from_s(10.0));

        // Should schedule an internal trigger event
        let mut next_events = connector.lock().unwrap().time_manager.next_events();
        assert!(next_events.next().unwrap().1.get(0).unwrap().is::<InternalLayerTrigger>());

        // Shouldn't see any EliminateEvents yet
        assert!(next_events.next().is_none());

        layer.lock().unwrap().pre_exec(&mut connector.lock().unwrap());
        run_layer_sync(&layer, &connector, &mut components);
        layer.lock().unwrap().post_exec(&mut connector.lock().unwrap());

        connector.lock().unwrap().time_manager.advance_by(SimTime::from_s(2.0));

        // Should be eliminating the ammonia
        let (exec_time, mut evts) = connector.lock().unwrap().time_manager.next_events().next().unwrap();
        let expected_time = SimTime::from_s(10.0) + DigestionLayer::<TestOrganism>::ELIMINATION_DELAY;
        let threshold = secs!(0.1);
        assert!(
            (exec_time-threshold..exec_time+threshold).contains(&expected_time),
            "Expected time {} != {}",
            expected_time,
            exec_time
        );

        assert!(evts.get(0).is_some());
        assert!(evts.get(0).unwrap().is::<EliminateEvent>());

        let elim = evts.pop().unwrap().downcast::<EliminateEvent>().unwrap();
        assert_eq!(elim.direction, DigestionDirection::BACK);
        assert!(elim.excrement.concentration_of(&Substance::NH3) > SubstanceConcentration::from_mM(0.0));

        layer.lock().unwrap().pre_exec(&mut connector.lock().unwrap());


        // Both of the remaining consumables should still be with the first component
        assert_eq!(layer.lock().unwrap().consumed_map.get(0).unwrap().len(), 2);
        
        // Should be consuming some sugar
        assert!(layer.lock().unwrap().consumed_map.get(0).unwrap().get(0).unwrap().concentration_of(&Substance::GLC) < starting_glc);
        
        // Exec for completeness (shouldn't change anything)
        run_layer_sync(&layer, &connector, &mut components);
        layer.lock().unwrap().post_exec(&mut connector.lock().unwrap());

        connector.lock().unwrap().time_manager.advance_by(SimTime::from_min(1.0));
        layer.lock().unwrap().pre_exec(&mut connector.lock().unwrap());

        // First component should still have the food
        assert_eq!(layer.lock().unwrap().consumed_map.get(0).unwrap().len(), 1);

        // Fiber should have moved to the next component
        assert_eq!(layer.lock().unwrap().consumed_map.get(1).unwrap().len(), 1);
        assert!(layer.lock().unwrap().consumed_map.get(1).unwrap().get(0).unwrap().concentration_of(&Substance::Cellulose) > SubstanceConcentration::from_mM(0.0));

        run_layer_sync(&layer, &connector, &mut components);
        layer.lock().unwrap().post_exec(&mut connector.lock().unwrap());

        // each component should have 1 consumed
        assert_eq!(layer.lock().unwrap().consumed_map.get(0).unwrap().len(), 1);
        assert_eq!(layer.lock().unwrap().consumed_map.get(1).unwrap().len(), 1);

        // Go through a cycle
        connector.lock().unwrap().time_manager.advance_by(SimTime::from_min(5.0));
        layer.lock().unwrap().pre_exec(&mut connector.lock().unwrap());
        run_layer_sync(&layer, &connector, &mut components);
        layer.lock().unwrap().post_exec(&mut connector.lock().unwrap());
        
        // Each list should be empty now
        assert_eq!(layer.lock().unwrap().consumed_map.get(0).unwrap().len(), 0);
        assert_eq!(layer.lock().unwrap().consumed_map.get(1).unwrap().len(), 0);

        connector.lock().unwrap().time_manager.advance_by(SimTime::from_s(1.0));

        // Should be eliminating the fiber
        let (_, mut evts) = connector.lock().unwrap().time_manager.next_events().next().unwrap();

        assert!(evts.get(0).is_some());
        assert!(evts.get(0).unwrap().is::<EliminateEvent>());

        let elim = evts.pop().unwrap().downcast::<EliminateEvent>().unwrap();
        assert_eq!(elim.direction, DigestionDirection::FORWARD);
        assert!(elim.excrement.concentration_of(&Substance::Cellulose) > SubstanceConcentration::from_mM(0.0));

        // Food should have dissappeared
        assert!(layer.lock().unwrap().consumed_map.get(0).unwrap().is_empty());
        assert!(layer.lock().unwrap().consumed_map.get(1).unwrap().is_empty());
    }
}
