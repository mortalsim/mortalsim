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

pub struct TimeManager<'b> {
    /// Identifier for this TimeManager object
    manager_id: Uuid,
    /// Current simulation time
    sim_time: Time,
    /// Sorted map of events to be executed
    event_queue: BTreeMap<OrderedTime, Vec<(IdType, TypeId, Box<dyn Event>)>>,
    /// Generator for our listener IDs
    id_gen: IdGenerator,
    /// Map of listeners for any time advance
    advance_listeners: HashMap<IdType, Box<dyn FnMut() + 'b>>,
    /// Map of listeners to execute at future simulation times
    scheduled_listeners: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn FnOnce() + 'b>)>>,
    /// Used to lookup listeners and Event objects for unscheduling
    id_time_map: HashMap<IdType, OrderedTime>
}

impl<'b> fmt::Debug for TimeManager<'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TimeManager({:?}) {{ sim_time: {:?}, event_queue: {:?}, advance_listener_ids: {:?}, scheduled_listener_times: {:?} }}",
            self.manager_id,
            self.sim_time,
            self.event_queue,
            self.advance_listeners.keys(),
            self.scheduled_listeners.keys())
    }
}

impl<'b> TimeManager<'b> {
    /// Creates a new TimeManager object starting at t = 0
    pub fn new() -> TimeManager<'b> {
        TimeManager {
            manager_id: Uuid::new_v4(),
            sim_time: Time::new::<second>(0.0),
            event_queue: BTreeMap::new(),
            id_gen: IdGenerator::new(),
            advance_listeners: HashMap::new(),
            scheduled_listeners: BTreeMap::new(),
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
        let sched_listener_next = self.scheduled_listeners.iter().next();

        if evt_queue_next.is_some() && sched_listener_next.is_some() {
            // We have both an event and a listener scheduled so we need to figure out
            // which one comes first first
            let (evt_time, _) = evt_queue_next.unwrap();
            let (lis_time, _) = sched_listener_next.unwrap();

            if evt_time < lis_time {
                self.sim_time = evt_time.get_value();
            }
            else {
                self.sim_time = lis_time.get_value();
            }
        }
        else if evt_queue_next.is_some() {
            self.sim_time = evt_queue_next.unwrap().0.get_value();
        }
        else if sched_listener_next.is_some() {
            self.sim_time = sched_listener_next.unwrap().0.get_value();
        }
        else {
            // If both queues are empty, there's nothing to do
            return;
        }

        self.execute_advance_listeners();
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
        self.execute_advance_listeners();
    }

    /// Schedules an `Event` for future emission
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    pub fn schedule_event<T: Event>(&mut self, wait_time: Time, event: T) -> IdType {
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
        evt_list.unwrap().push((id, TypeId::of::<T>(), Box::new(event)));

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
    pub fn unschedule_event(&mut self, schedule_id: IdType) -> Result<(), Error> {
        match self.id_time_map.get(&schedule_id) {
            Some(time) => {
                match self.event_queue.get_mut(time) {
                    Some(evt_list) => {
                        evt_list.retain(|item| item.0 != schedule_id);
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

    /// Registers a listener for time advances
    /// 
    /// ### Arguments
    /// * `listener` - function to call when time advances
    pub fn on_advance(&mut self, listener: impl FnMut() + 'b) -> IdType {
        let lis_id = self.id_gen.get_id();
        self.advance_listeners.insert(lis_id, Box::new(listener));
        lis_id
    }

    /// Unregisters a previously attached time advance listener
    /// 
    /// ### Arguments
    /// * `listener_id` - identifier returned from the call to `on_advance`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn off_advance(&mut self, listener_id: IdType) -> Result<()> {
        match self.advance_listeners.remove(&listener_id) {
            Some(_) => Ok(()),
            None => Err(anyhow!("Invalid listener id {} provided to TimeManager {}", listener_id, self.manager_id))
        }
    }

    /// Schedules a callback to be called at a future simulation time
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before calling the listener
    /// * `listener` - function to call at the scheduled time
    /// 
    /// Returns an ID for the scheduled listener
    pub fn schedule_callback(&mut self, wait_time: Time, listener: impl FnOnce() + 'b) -> IdType {
        let exec_time = OrderedTime(self.sim_time + wait_time);
        let mut listeners = self.scheduled_listeners.get_mut(&exec_time);

        // If we haven't already created a vector for this time step, 
        // then create one now and get the populated Option
        if listeners.is_none() {
            self.scheduled_listeners.insert(exec_time, Vec::new());
            listeners = self.scheduled_listeners.get_mut(&exec_time);
        }
        
        // Generate a new ID for the scheduled callback
        let lis_id = self.id_gen.get_id();

        // Add the listener and its ID to the listeners array, which
        // should certainly exist by this point
        listeners.unwrap().push((lis_id, Box::new(listener)));

        // Add the mapping of the listener id to the execution time
        // for faster lookup later
        self.id_time_map.insert(lis_id, exec_time);

        // return the new id
        lis_id
    }

    /// Unschedules a previously scheduled listener
    /// 
    /// ### Arguments
    /// * `listener_id` - The identifier returned from the call to `schedule_callback`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn unschedule_callback(&mut self, listener_id: IdType) -> Result<()> {
        match self.id_time_map.get(&listener_id) {
            Some(time) => {
                match self.scheduled_listeners.get_mut(time) {
                    Some(listeners) => {
                        listeners.retain(|item| item.0 != listener_id);
                        Ok(())
                    }
                    None => {
                        panic!("Scheduled listener ID {} refers to an invalid time on TimeManager {}!", listener_id, self.manager_id);
                    }
                }
            }
            None => {
                Err(anyhow!("Invalid schedule_id {} passed to `unschedule_event` for TimeManager {}", listener_id, self.manager_id))
            }
        }
    }

    pub(super) fn next_events(&mut self) -> Option<(OrderedTime, Vec<(TypeId, Box<dyn Event>)>)> {
        let mut evt_times = self.event_queue.keys().cloned();
        let next_evt_time = evt_times.next();

        if next_evt_time.is_some() && next_evt_time.unwrap() <= OrderedTime(self.sim_time) {
            let evt_time = next_evt_time.unwrap();
            let evt_list = self.event_queue.remove(&evt_time).unwrap();

            // Drop the registration token when returning the result vector
            let result: Vec<(TypeId, Box<dyn Event>)> = evt_list.into_iter().map(|(_, type_key, evt)| (type_key, evt)).rev().collect();
            Some((evt_time, result))
        }
        else {
            None
        }
    }

    pub(super) fn next_listeners(&mut self) -> Option<(OrderedTime, Vec<Box<dyn FnOnce() + 'b>>)> {
        let mut lis_times = self.scheduled_listeners.keys().cloned();
        let next_lis_time = lis_times.next();

        if next_lis_time.is_some() && next_lis_time.unwrap() <= OrderedTime(self.sim_time) {
            let lis_time = next_lis_time.unwrap();
            let listener_list = self.scheduled_listeners.remove(&lis_time).unwrap();

            // Drop the registration token when returning the result vector
            let result: Vec<Box<dyn FnOnce() + 'b>> = listener_list.into_iter().map(|(_, listener)| listener).rev().collect();
            Some((lis_time, result))
        }
        else {
            None
        }
    }

    fn execute_advance_listeners(&mut self) {
        for (_, listener) in self.advance_listeners.iter_mut() {
            listener();
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
        // Track advance counts for each of our listeners
        // note that order is important here because
        // counter_a and counter_b are captured by the
        // callbacks passed to the time_manager, which
        // then must exist for at least the duration of
        // time_manager. Otherwise the compiler gets upset.
        let counter_a: Cell<u32> = Cell::new(0);
        let counter_b: Cell<u32> = Cell::new(0);

        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);
        
        // Count the number of advances through
        // the callback
        let a_id = time_manager.on_advance(|| {
            counter_a.set(counter_a.get() + 1);
        });

        // Time should start at zero seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(0.0));

        // Advance time by 1s
        time_manager.advance_by(one_sec);

        // The callback should have been called
        assert_eq!(counter_a.get(), 1);

        // time should now be at approx. 1 second
        // (within possible floating point errors)
        assert_eq!(time_manager.get_time(), one_sec);
        
        // Add a seconed listener
        let b_id = time_manager.on_advance(|| {
            counter_b.set(counter_b.get() + 1);
        });
        
        // Advance another second
        time_manager.advance_by(one_sec);

        // Both callbacks should have been called
        assert_eq!(counter_a.get(), 2);
        assert_eq!(counter_b.get(), 1);

        // Time should now be at 2 seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(2.0));

        // Remove a callback
        time_manager.off_advance(a_id).unwrap();

        // Advance again, but this time by 3 seconds
        time_manager.advance_by(Time::new::<second>(3.0));

        // Only the b callback should have been called
        assert_eq!(counter_a.get(), 2);
        assert_eq!(counter_b.get(), 2);
        
        // Time should now be at 5 seconds
        assert_eq!(time_manager.get_time(), Time::new::<second>(5.0));

        // Remove the other callback
        time_manager.off_advance(b_id).unwrap();

        // Advance one last time
        time_manager.advance_by(one_sec);

        // This time b shouldn't have been triggered
        assert_eq!(counter_b.get(), 2);

        // Try off'ing one of our ids again. This should
        // give us an Err
        assert!(time_manager.off_advance(a_id).is_err());
    }

    #[test]
    fn schedule_test() {
        let call_flag_a: Cell<bool> = Cell::new(false);
        let call_flag_b: Cell<bool> = Cell::new(false);

        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);
        
        // Schedule our test callbacks
        time_manager.schedule_callback(Time::new::<second>(5.0), || {
            call_flag_a.set(true);
        });
        time_manager.schedule_callback(Time::new::<second>(7.0), || {
            call_flag_b.set(true);
        });

        // Advance by one second
        time_manager.advance_by(one_sec);

        // No listeners should be ready to call
        assert!(time_manager.next_listeners().is_none());

        // Advance automatically to the next scheduled thing
        time_manager.advance();
        
        // Execute listeners
        for listener in time_manager.next_listeners().unwrap().1 {
            listener();
        }

        // Time should now be at 5.0s
        assert_eq!(time_manager.get_time(), Time::new::<second>(5.0));
        assert!(call_flag_a.get(), "Call flag 'a' should have fired");
        assert!(!call_flag_b.get(), "Call flag 'b' should NOT have fired");

        // Advance again
        time_manager.advance();
        
        // Execute listeners
        for listener in time_manager.next_listeners().unwrap().1 {
            listener();
        }

        // Time should now be at 7.0s
        assert_eq!(time_manager.get_time(), Time::new::<second>(7.0));
        assert!(call_flag_b.get(), "Call flag 'b' should have fired now");
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
        time_manager.schedule_event(Time::new::<second>(2.0), a_evt);
        time_manager.schedule_event(Time::new::<second>(6.0), b_evt);

        // Advance by 1s. No events yet.
        time_manager.advance_by(one_sec);
        assert_eq!(time_manager.get_time(), Time::new::<second>(1.0));
        assert!(time_manager.next_events().is_none());
        
        // Advance again. First event should fire.
        time_manager.advance_by(one_sec);
        for (evt_type, evt) in time_manager.next_events().unwrap().1.into_iter() {
            assert_eq!(evt_type, TypeId::of::<TestEventA>());
            assert_eq!(evt.downcast::<TestEventA>().unwrap().len, Length::new::<meter>(3.5));
        }
        assert_eq!(time_manager.get_time(), Time::new::<second>(2.0));
        
        // Advance again automatically. Should fire the second event.
        time_manager.advance();
        for (evt_type, evt) in time_manager.next_events().unwrap().1.into_iter() {
            assert_eq!(evt_type, TypeId::of::<TestEventB>());
            assert_eq!(evt.downcast::<TestEventB>().unwrap().amt, AmountOfSubstance::new::<mole>(123456.0));
        }
        assert_eq!(time_manager.get_time(), Time::new::<second>(6.0));
    }
    #[test]

    fn evt_lis_mixed_test() {

        // Create target Cells to grab our Events when they are emitted
        let call_flag_c: Cell<bool> = Cell::new(false);
        let call_flag_d: Cell<bool> = Cell::new(false);

        let a_evt = TestEventA::new(Length::new::<meter>(3.5));
        let b_evt = TestEventB::new(AmountOfSubstance::new::<mole>(123456.0));
        
        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);

        // Schedule a callback at 1s
        time_manager.schedule_callback(one_sec, || {
            call_flag_c.set(true);
        });

        // Schedule the events to be emitted later
        time_manager.schedule_event(Time::new::<second>(2.0), a_evt);
        time_manager.schedule_event(Time::new::<second>(2.0), b_evt);
        
        // Schedule a callback at 7s
        time_manager.schedule_callback(Time::new::<second>(7.0), || {
            call_flag_d.set(true);
        });

        // Advance by 1s. First callback should fire
        time_manager.advance_by(one_sec);
        for listener in time_manager.next_listeners().unwrap().1 {
            listener();
        }
        assert!(time_manager.next_events().is_none());
        assert!(call_flag_c.get(), "C Event should have fired.");
        assert!(!call_flag_d.get(), "D Event should not have fired.");
        
        // Advance again. Event A and B should fire now.
        time_manager.advance_by(one_sec);
        assert!(time_manager.next_listeners().is_none());
        for (evt_type, evt) in time_manager.next_events().unwrap().1.into_iter() {
            if evt.is::<TestEventA>() {
                assert_eq!(evt_type, TypeId::of::<TestEventA>());
                assert_eq!(evt.downcast::<TestEventA>().unwrap().len, Length::new::<meter>(3.5));
            }
            else {
                assert_eq!(evt_type, TypeId::of::<TestEventB>());
                assert_eq!(evt.downcast::<TestEventB>().unwrap().amt, AmountOfSubstance::new::<mole>(123456.0));
            }
        }
        assert!(!call_flag_d.get(), "D Event should not have fired.");
        
        // Last advance. Should fire D now.
        time_manager.advance();
        for listener in time_manager.next_listeners().unwrap().1 {
            listener();
        }
        assert!(call_flag_d.get(), "D Event should have fired.");
    }
}