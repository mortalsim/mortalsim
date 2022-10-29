//! Internal time management for the simulation
//!
//! This submodule provides the heart of the simulation by
//! advancing time - either by a specified interval or by jumping
//! immediately to the next `Event`

use std::collections::BTreeMap;
use std::collections::hash_map::{HashMap, Keys};
use std::any::TypeId;
use std::fmt;
use uuid::Uuid;
use uom::si::time::second;
use uom::fmt::DisplayStyle::*;
use anyhow::{Result, Error};
use crate::util::id_gen::{IdType, IdGenerator};
use crate::event::Event;
use crate::util::quantity_wrapper::OrderedTime;

pub type Time = uom::si::f64::Time;

pub struct TimeManager {
    /// Identifier for this TimeManager object
    manager_id: Uuid,
    /// Current simulation time
    sim_time: Time,
    /// Sorted map of events to be executed
    event_queue: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn Event>)>>,
    /// Generator for our listener IDs
    id_gen: IdGenerator,
    /// Used to lookup listeners and Event objects for unscheduling
    id_time_map: HashMap<IdType, OrderedTime>
}

impl<'b> fmt::Debug for TimeManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TimeManager({:?}) {{ sim_time: {:?}, event_queue: {:?} }}",
            self.manager_id,
            self.sim_time,
            self.event_queue)
    }
}

impl TimeManager {
    /// Creates a new TimeManager object starting at t = 0
    pub fn new() -> TimeManager {
        TimeManager {
            manager_id: Uuid::new_v4(),
            sim_time: Time::new::<second>(0.0),
            event_queue: BTreeMap::new(),
            id_gen: IdGenerator::new(),
            id_time_map: HashMap::new()
        }
    }

    /// Returns the current simulation time
    pub fn get_time(&self) -> Time {
        self.sim_time
    }

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    pub fn advance(&mut self) {
        log::debug!("Advancing time for TimeManager {}", self.manager_id);

        // Get the first values for the event queue and scheduled listener queue
        let evt_queue_next = self.event_queue.iter().next();

        if evt_queue_next.is_some() {
            self.sim_time = evt_queue_next.unwrap().0.get_value();
        }
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    pub fn advance_by(&mut self, time_step: Time) {
        // If the time_step is zero or negative, advance to the next
        // point in the simulation
        if time_step <= Time::new::<second>(0.0) {
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
    pub fn schedule_event(&mut self, wait_time: Time, event: Box<dyn Event>) -> IdType {
        let exec_time = OrderedTime(self.sim_time + wait_time);
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
            Some(time) => {
                match self.event_queue.get_mut(time) {
                    Some(evt_list) => {
                        evt_list.retain(|item| item.0 != *schedule_id);
                        Ok(())
                    }
                    None => {
                        panic!("Scheduled ID {} refers to an invalid time on TimeManager {}!", schedule_id, self.manager_id);
                    }
                }
            }
            None => {
                Err(anyhow!("Invalid schedule_id {} passed to `unschedule_event` for TimeManager {}", schedule_id, self.manager_id))
            }
        }
    }

    pub(super) fn next_events(&mut self) -> Option<(OrderedTime, Vec<Box<dyn Event>>)> {
        let mut evt_times = self.event_queue.keys().cloned();
        let next_evt_time = evt_times.next();

        if next_evt_time.is_some() && next_evt_time.unwrap() <= OrderedTime(self.sim_time) {
            let evt_time = next_evt_time.unwrap();
            let evt_list = self.event_queue.remove(&evt_time).unwrap();

            // Drop the registration token when returning the result vector
            let result: Vec<Box<dyn Event>> = evt_list.into_iter().map(|(_, evt)| evt).rev().collect();
            Some((evt_time, result))
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::any::TypeId;
    use super::Time;
    use super::second;
    use super::TimeManager;
    use super::Event;
    use crate::event::test::TestEventA;
    use crate::event::test::TestEventB;
    use uom::si::f64::Length;
    use uom::si::f64::AmountOfSubstance;
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;

    #[test]
    fn advance_test() {

        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);

        // Time should start at zero seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(0.0));

        // Advance time by 1s
        time_manager.advance_by(one_sec);

        // time should now be at 1 second
        assert_eq!(time_manager.get_time(), one_sec);
        
        // Advance another second
        time_manager.advance_by(one_sec);

        // Time should now be at 2 seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(2.0));

        // Advance again, but this time by 3 seconds
        time_manager.advance_by(Time::new::<second>(3.0));

        // Time should now be at 5 seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(5.0));
    }

    #[test]
    fn emit_events_test() {
        let a_evt = TestEventA::new(Length::new::<meter>(3.5));
        let b_evt = TestEventB::new(AmountOfSubstance::new::<mole>(123456.0));
        
        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);
        
        // Schedule the events to be emitted later
        time_manager.schedule_event(Time::new::<second>(2.0), Box::new(a_evt));
        time_manager.schedule_event(Time::new::<second>(6.0), Box::new(b_evt));

        // Advance by 1s. No events yet.
        time_manager.advance_by(one_sec);
        assert_eq!(time_manager.get_time(), Time::new::<second>(1.0));
        assert!(time_manager.next_events().is_none());
        
        // Advance again. First event should fire.
        time_manager.advance_by(one_sec);
        for evt in time_manager.next_events().unwrap().1.into_iter() {
            assert_eq!(evt.type_id(), TypeId::of::<TestEventA>());
            assert_eq!(evt.downcast::<TestEventA>().unwrap().len, Length::new::<meter>(3.5));
        }
        assert_eq!(time_manager.get_time(), Time::new::<second>(2.0));
        
        // Advance again automatically. Should fire the second event.
        time_manager.advance();
        for evt in time_manager.next_events().unwrap().1.into_iter() {
            assert_eq!(evt.type_id(), TypeId::of::<TestEventB>());
            assert_eq!(evt.downcast::<TestEventB>().unwrap().amt, AmountOfSubstance::new::<mole>(123456.0));
        }
        assert_eq!(time_manager.get_time(), Time::new::<second>(6.0));
    }
}
