//! Internal time management for the simulation
//!
//! This submodule provides the heart of the simulation by
//! advancing time - either by a specified interval or by jumping
//! immediately to the next `Event`

use crate::event::Event;
use crate::hub::event_transformer::{EventTransformer, TransformerItem};
use crate::units::base::Time;
use crate::id_gen::{IdGenerator, IdType, InvalidIdError};
use crate::{SimTime, SimTimeSpan};
use anyhow::{Error, Result};
use std::any::TypeId;
use std::collections::hash_map::HashMap;
use std::collections::BTreeMap;
use std::fmt;

pub struct TimeManager {
    /// Current simulation time
    sim_time: SimTime,
    /// Sorted map of events to be executed
    event_queue: BTreeMap<SimTime, Vec<(IdType, Box<dyn Event>)>>,
    /// Map of event transformer functions
    event_transformers: HashMap<TypeId, Vec<Box<dyn EventTransformer>>>,
    /// Map of event transformer ids to event types for quick lookup
    transformer_type_map: HashMap<IdType, TypeId>,
    /// Generator for our listener IDs
    id_gen: IdGenerator,
    /// Used to lookup listeners and Event objects for unscheduling
    id_time_map: HashMap<IdType, SimTime>,
}

impl<'b> fmt::Debug for TimeManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TimeManager {{ sim_time: {:?}, event_queue: {:?} }}",
            self.sim_time, self.event_queue
        )
    }
}

impl TimeManager {
    /// Creates a new TimeManager object starting at t = 0
    pub fn new() -> TimeManager {
        TimeManager {
            sim_time: SimTime::from_s(0.0),
            event_queue: BTreeMap::new(),
            event_transformers: HashMap::new(),
            transformer_type_map: HashMap::new(),
            id_gen: IdGenerator::new(),
            id_time_map: HashMap::new(),
        }
    }

    /// Returns the current simulation time
    pub fn get_time(&self) -> SimTime {
        self.sim_time
    }

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    ///
    /// If there are no Events or listeners in the queue, time will remain unchanged
    pub fn advance(&mut self) {
        // Get the first values for the event queue and scheduled listener queue
        let evt_queue_next = self.event_queue.iter().next();

        if evt_queue_next.is_some() {
            self.sim_time = evt_queue_next.unwrap().0.clone();
        }
    }

    /// Advances simulation time by the provided time step
    ///
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    ///
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    pub fn advance_by(&mut self, time_step: SimTimeSpan) {
        // If the time_step is zero or negative, advance to the next
        // point in the simulation
        if time_step <= SimTimeSpan::from_s(0.0) {
            return self.advance();
        }

        // otherwise, advance time and trigger listeners
        self.sim_time = self.sim_time + time_step;
    }

    /// Schedules an `Event` for future emission
    ///
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    ///
    /// Returns the schedule ID
    pub fn schedule_event(&mut self, wait_time: SimTimeSpan, event: Box<dyn Event>) -> IdType {
        let exec_time = self.sim_time + wait_time;
        let mut evt_list = self.event_queue.get_mut(&exec_time);

        // If we haven't already created a vector for this time step,
        // then create one now and get the populated Option
        if evt_list.is_none() {
            self.event_queue.insert(exec_time, Vec::new());
            evt_list = self.event_queue.get_mut(&exec_time);
        }

        // Generate an id for the event
        let id = self.id_gen.get_id();

        // Add the (id, event) tuple to the event list which should
        // certainly exist by this point
        evt_list.unwrap().push((id, event));

        // Insert a mapping for the id to the execution time for
        // faster lookup later
        self.id_time_map.insert(id, exec_time);

        // Return the generated id
        id
    }

    /// Unschedules a previously scheduled `Event`
    ///
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    ///
    /// Returns an Err Result if the provided ID is invalid
    pub fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<(), Error> {
        match self.id_time_map.get(&schedule_id) {
            Some(time) => match self.event_queue.get_mut(time) {
                Some(evt_list) => {
                    evt_list.retain(|item| item.0 != *schedule_id);
                    Ok(())
                }
                None => {
                    Err(anyhow!("Scheduled ID {} refers to an invalid time!", schedule_id))
                }
            },
            None => Err(anyhow!(
                "Invalid schedule_id {} passed to `unschedule_event`!",
                schedule_id,
            )),
        }
    }

    /// Gets an iterator of all events that are ready for emission
    /// with their associated emission time.
    pub fn next_events(&mut self) -> impl Iterator<Item = (SimTime, Vec<Box<dyn Event>>)> {
        let mut evt_times = self.event_queue.keys();
        let mut next_evt_time = evt_times.next();
        let mut times_to_remove = Vec::new();

        while let Some(evt_time) = next_evt_time {
            // stop when we reach the first event time that
            // hasn't occurred yet
            if *evt_time > self.sim_time {
                break;
            }
            times_to_remove.push(*evt_time);
            next_evt_time = evt_times.next();
        }

        let mut results = Vec::new();

        for evt_time in times_to_remove {
            let mut evt_list = self.event_queue.remove(&evt_time).unwrap();

            for (id, evt) in evt_list.iter_mut().rev() {
                // Call any transformers on the event
                for transformers in self.event_transformers.get_mut(&evt.type_id()).iter_mut() {
                    for transformer in transformers.iter_mut() {
                        transformer.transform(evt.as_mut());
                    }
                }

                // Remove the ids from our mapping and recycle them
                self.id_time_map.remove(id);
                self.id_gen.return_id(*id).expect(format!("Error returning id {}", id).as_str());
            }
            
            // Drop the registration token when returning the result vector
            let result: Vec<Box<dyn Event>> =
                evt_list.into_iter().map(|(_, e)| e).rev().collect();

            if !result.is_empty() {
                results.push((evt_time, result));
            }
        }
        results.into_iter()
    }

    /// Registers a transformer for a specific Event.
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    ///
    /// Returns the registration ID for the transformer
    pub fn transform<E: Event>(&mut self, handler: impl FnMut(&mut E) + Send + 'static) -> IdType {
        self.insert_transformer(Box::new(TransformerItem::new(handler)))
    }

    /// Registers a transformer for a specific Event with the given priority. Higher
    /// priority transformers are executed first.
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Priority of the transformer
    ///
    /// Returns the registration ID for the transformer
    pub fn transform_prioritized<E: Event>(
        &mut self,
        priority: i32,
        handler: impl FnMut(&mut E) + Send + 'static,
    ) -> IdType {
        self.insert_transformer(Box::new(TransformerItem::new_prioritized(
            handler, priority,
        )))
    }

    pub(super) fn insert_transformer(&mut self, transformer: Box<dyn EventTransformer>) -> IdType {
        let transformer_id = transformer.transformer_id();
        let type_key = transformer.type_id();

        match self.event_transformers.get(&type_key) {
            Some(transformers) => match transformers.binary_search(&transformer) {
                Ok(_) => panic!("Duplicate Transformer id {}", transformer.transformer_id()),
                Err(pos) => {
                    self.event_transformers
                        .get_mut(&type_key)
                        .unwrap()
                        .insert(pos, transformer);
                }
            },
            None => {
                self.event_transformers.insert(type_key, vec![transformer]);
            }
        }

        // Add the id -> type mapping for quick removal if needed later
        self.transformer_type_map.insert(transformer_id, type_key);

        transformer_id
    }

    /// Unregisters a transformer for a specific Event with the given registration ID returned
    /// from the call to `transform` or `transform_prioritized`.
    ///
    /// ### Arguments
    /// * `transformer_id` - transformer registration ID
    ///
    /// Returns Ok if successful, or Err if the provided ID is invalid.
    pub fn unset_transform(&mut self, transformer_id: &IdType) -> Result<()> {
        match self.transformer_type_map.get(&transformer_id) {
            Some(type_key) => {
                let transformers = self.event_transformers.get_mut(type_key).unwrap();
                match transformers
                    .iter()
                    .position(|l| l.transformer_id() == *transformer_id)
                {
                    Some(pos) => {
                        transformers.remove(pos);
                        self.transformer_type_map.remove(transformer_id);
                        Ok(())
                    }
                    None => Err(anyhow::Error::new(InvalidIdError::new(
                        format!("{:?}", self),
                        *transformer_id,
                    ))),
                }
            }
            None => Err(anyhow::Error::new(InvalidIdError::new(
                format!("{:?}", self),
                *transformer_id,
            ))),
        }
    }
}


mod tests {
    use super::Time;
    use super::TimeManager;
    use crate::event::test::TestEventA;
    use crate::event::test::TestEventB;
    use crate::event::Event;
    use crate::hub::event_transformer::{EventTransformer, TransformerItem};
    use crate::sim::SimTime;
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use crate::secs;
    use crate::SimTimeSpan;
    use std::any::TypeId;

    #[test]
    fn advance_test() {
        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = SimTime::from_s(1.0);
        let one_sec_span = SimTimeSpan::from_s(1.0);

        // Time should start at zero seconds
        assert_eq!(time_manager.get_time(), SimTime::from_s(0.0));

        // Advance time by 1s
        time_manager.advance_by(one_sec_span);

        // time should now be at 1 second
        assert_eq!(time_manager.get_time(), one_sec);

        // Advance another second
        time_manager.advance_by(one_sec_span);

        // Time should now be at 2 seconds
        assert_eq!(time_manager.get_time(), SimTime::from_s(2.0));

        // Advance again, but this time by 3 seconds
        time_manager.advance_by(SimTimeSpan::from_s(3.0));

        // Time should now be at 5 seconds
        assert_eq!(time_manager.get_time(), SimTime::from_s(5.0));
    }

    #[test]
    fn emit_events_test() {
        let a_evt = TestEventA::new(Distance::from_m(3.5));
        let b_evt = TestEventB::new(Amount::from_mol(123456.0));

        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();

        // Schedule the events to be emitted later
        time_manager.schedule_event(SimTimeSpan::from_s(2.0), Box::new(a_evt));
        time_manager.schedule_event(SimTimeSpan::from_s(6.0), Box::new(b_evt));

        // Advance by 1s. No events yet.
        time_manager.advance_by(SimTimeSpan::from_s(1.0));
        assert_eq!(time_manager.get_time(), SimTime::from_s(1.0));

        let mut next_events: Vec<(SimTime, Vec<Box<dyn Event>>)> =
            time_manager.next_events().collect();
        assert!(next_events.is_empty());

        // Advance again. First event should fire.
        time_manager.advance_by(SimTimeSpan::from_s(1.1));

        next_events = time_manager.next_events().collect();
        assert!(!next_events.is_empty());

        for evt in time_manager.next_events().map(|x| x.1).flatten() {
            assert_eq!(evt.type_id(), TypeId::of::<TestEventA>());
            assert_eq!(
                evt.downcast::<TestEventA>().unwrap().len,
                Distance::from_m(3.5)
            );
        }
        assert_eq!(time_manager.get_time(), SimTime::from_s(2.1));

        // Advance again automatically. Should fire the second event.
        time_manager.advance();

        next_events = time_manager.next_events().collect();
        assert!(!next_events.is_empty());

        for evt in time_manager.next_events().map(|x| x.1).flatten() {
            assert_eq!(evt.type_id(), TypeId::of::<TestEventB>());
            assert_eq!(
                evt.downcast::<TestEventB>().unwrap().amt,
                Amount::from_mol(123456.0)
            );
        }
        assert_eq!(time_manager.get_time(), SimTime::from_s(6.0));
    }

    #[test]
    fn unschedule_test() {
        let a_evt = TestEventA::new(Distance::from_m(3.5));
        let b_evt = TestEventB::new(Amount::from_mol(123456.0));

        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();

        // Schedule the events to be emitted later
        let id_a = time_manager.schedule_event(SimTimeSpan::from_s(2.0), Box::new(a_evt));
        let id_b = time_manager.schedule_event(SimTimeSpan::from_s(6.0), Box::new(b_evt));

        // Advance a little, but not past the first event
        time_manager.advance_by(SimTimeSpan::from_s(1.0));

        // Unschedule the A event
        time_manager.unschedule_event(&id_a).expect("Error unscheduling a valid event");

        time_manager.advance_by(SimTimeSpan::from_s(2.0));
        
        let next_events: Vec<(SimTime, Vec<Box<dyn Event>>)> =
            time_manager.next_events().collect();
        assert!(next_events.is_empty());

        time_manager.advance_by(SimTimeSpan::from_s(10.0));

        // Try unscheduling an id which is no longer valid
        time_manager.unschedule_event(&id_b).unwrap();
    }

    #[test]
    fn transformer_test() {
        let mut listener = TransformerItem::new(|evt: &mut TestEventA| {
            evt.len = Distance::from_m(10.0);
        });

        let mut evt = TestEventA::new(Distance::from_m(5.0));
        assert_eq!(evt.len, Distance::from_m(5.0));

        listener.transform(&mut evt);
        assert_eq!(evt.len, Distance::from_m(10.0));
    }
}
