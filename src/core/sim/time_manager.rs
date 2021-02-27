//! Internal time management for the simulation
//!
//! This submodule provides the heart of the simulation by
//! advancing time - either by a specified interval or by jumping
//! immediately to the next `Event`

use std::collections::BTreeMap;
use std::collections::hash_map::HashMap;
use uuid::Uuid;
use uom::si::f64::Time;
use uom::si::time::second;
use uom::fmt::DisplayStyle::*;
use anyhow::{Result, Error};
use crate::util::id_gen::{IdType, IdGenerator};
use crate::event::Event;
use crate::util::quantity_wrapper::OrderedTime;

struct TimeManager<'b> {
    /// Identifier for this TimeManager object
    manager_id: Uuid,
    /// Current simulation time
    sim_time: Time,
    /// Sorted map of events to be executed
    event_queue: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn Event>)>>,
    /// Generator for our listener IDs
    id_gen: IdGenerator,
    /// Map of listeners for any time advance
    advance_listeners: HashMap<IdType, Box<dyn FnMut() + 'b>>,
    /// Map of listeners to execute at future simulation times
    scheduled_listeners: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn FnOnce() + 'b>)>>,
    /// Event listener to call when events are emitted
    event_listener: Option<Box<dyn FnMut(Box<dyn Event>) + 'b>>,
    /// Used to lookup listeners and Event objects for unscheduling
    id_time_map: HashMap<IdType, OrderedTime>
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
            event_listener: None,
            id_time_map: HashMap::new()
        }
    }

    /// Returns the current simulation time
    pub fn get_time(&self) -> Time {
        self.sim_time
    }

    /// Advances simulation time to the next `Event` in the queue, if any.
    /// 
    /// If there are no Events in the queue, time will remain unchanged
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

        self.trigger_listeners();
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// # Arguments
    /// * `time_step` - Amount of time to advance by
    pub fn advance_by(&mut self, time_step: Time) {
        // If the time_step is zero or negative, advance to the next
        // point in the simulation
        if time_step <= Time::new::<second>(0.0) {
            return self.advance();
        }

        // otherwise, advance time and trigger listeners
        self.sim_time = self.sim_time + time_step;
        self.trigger_listeners();
    }

    /// Schedules an `Event` for future emission
    /// 
    /// # Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    pub fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType {
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
        evt_list.unwrap().push((id, Box::new(event)));

        // Insert a mapping for the id to the execution time for
        // faster lookup later
        self.id_time_map.insert(id, exec_time);
        
        // Return the generated id
        id
    }

    /// Unschedules a previously scheduled `Event`
    /// 
    /// # Arguments
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

    /// Registers a listener for `Event` objects
    /// 
    /// Events are always emitted one at a time in time order. If Events
    /// are emitted at identical times, they are emitted to listeners in
    /// the order they were scheduled.
    /// 
    /// If an event listener is already registered, calling this again
    /// will replace it
    /// 
    /// # Arguments
    /// * `listener` - an EventListener function to call when `Event`
    ///                objects are emitted
    pub fn on_event(&mut self, listener: impl FnMut(Box<dyn Event>) + 'b) {
        self.event_listener = Some(Box::new(listener));
    }

    /// Registers a listener for time advances
    /// 
    /// # Arguments
    /// * `listener` - function to call when time advances
    pub fn on_advance(&mut self, listener: impl FnMut() + 'b) -> IdType {
        let lis_id = self.id_gen.get_id();
        self.advance_listeners.insert(lis_id, Box::new(listener));
        lis_id
    }

    /// Unregisters a previously attached time advance listener
    /// 
    /// # Arguments
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
    /// # Arguments
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
    /// # Arguments
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

    /// Internal function for triggering scheduled listeners and event listeners
    fn trigger_listeners(&mut self) {
        log::debug!("Triggering listeners for TimeManager {}", self.manager_id);

        for (_, listener) in self.advance_listeners.iter_mut() {
            listener();
        }

        // Keep a list of scheduled listener times which will be completed
        let mut times_completed: Vec<OrderedTime> = Vec::new();

        // We need to iterate over the event_queue and scheduled_listeners queue
        // simultaneously so that we can execute everything in the appropriate
        // simulation time order
        let mut lis_times: Vec<OrderedTime> = self.scheduled_listeners.keys().cloned().collect();
        let mut evt_times: Vec<OrderedTime> = self.event_queue.keys().cloned().collect();

        // Keep track of the maximum time reached while we're iterating both
        // simultaneously
        let mut reached_time = Time::new::<second>(0.0);

        for (lis_time, evt_time) in lis_times.into_iter().zip(evt_times.into_iter()) {

            // If both times are beyond the current simulation time, exit the loop
            if lis_time.get_value() > self.sim_time && evt_time.get_value() > self.sim_time {
                break;
            }

            if lis_time < evt_time {
                // The listeners come first, so we need to call all of those associated
                // with the given time step
                self.execute_listeners(lis_time);

                // Add this time to the list of completed times
                times_completed.push(lis_time);
                reached_time = lis_time.get_value();
            }
            else {
                // The events come first, so we need to emit all of those to any
                // event listeners
                self.emit_events(evt_time);
                
                // Add this time to the list of completed times
                times_completed.push(evt_time);
                reached_time = evt_time.get_value();
            }
        }

        // Repopulate the array of times again... there's probably a more efficient
        // way to do this, but this keeps the borrow checker happy.
        lis_times = self.scheduled_listeners.keys().cloned().collect();

        // iterate over the scheduled_listeners until we reach a time which is
        // still in the future. Call each listener in those times and keep track
        // of completed times so we can remove them later.
        for time in lis_times {
            if time.get_value() > self.sim_time {
                // Go until we reach 
                break;
            }
            if time.get_value() < reached_time {
                // If we've already processed these, continue to the next iteration
                continue;
            }
            self.execute_listeners(time);

            // Add this time to the list of completed times
            times_completed.push(time);
        }
        
        // Repopulate the array of times again... there's probably a more efficient
        // way to do this, but this keeps the borrow checker happy.
        evt_times = self.event_queue.keys().cloned().collect();

        // iterate over the events until we reach a time which is
        // still in the future. Call each listener in those times and keep track
        // of completed times so we can remove them later.
        for time in evt_times {
            if time.get_value() > self.sim_time {
                // Go until we reach 
                break;
            }
            if time.get_value() < reached_time {
                // If we've already processed these, continue to the next iteration
                continue;
            }
            self.emit_events(time);

            // Add this time to the list of completed times
            times_completed.push(time);
        }

        // Remove the completed times from the scheduled listeners queue. Done
        // afterwards since we can't modify scheduled_listeners while iterating
        // (requires two mutable borrows)
        for time in &times_completed {
            self.scheduled_listeners.remove(&time);
        }

        // Remove the completed times from the event_queue. Also done
        // afterwards since we can't modify while iterating
        for time in &times_completed {
            self.event_queue.remove(&time);
        }
    }

    fn execute_listeners(&mut self, time: OrderedTime) {

        for (_, listener) in self.scheduled_listeners.remove(&time).unwrap() {
            log::debug!("TimeManager {} executing listener scheduled for {}",
                self.manager_id, time.get_value().into_format_args(second, Abbreviation));
            listener();
        }
    }

    fn emit_events(&mut self, time: OrderedTime) {

        log::debug!("TimeManager {} emitting events scheduled for {}",
            self.manager_id, time.get_value().into_format_args(second, Abbreviation));

        let mut events = self.event_queue.remove(&time).unwrap();

        // Get a mutable reference to the inner function
        match self.event_listener.as_mut() {
            Some(listener_fn) => {
                while let Some((_, event)) = events.pop() {
                    // Call the listener function with the given event
                    listener_fn(event);
                }
            }
            None => {
                log::debug!("... or not because no one is listening");
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
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

        // Nothing should have been called
        assert!(!call_flag_a.get(), "Call flag 'a' hasn't been fired yet");
        assert!(!call_flag_b.get(), "Call flag 'b' hasn't been fired yet");

        // Advance automatically to the next scheduled thing
        time_manager.advance();

        // Time should now be at 5.0s
        assert_eq!(time_manager.get_time(), Time::new::<second>(5.0));
        assert!(call_flag_a.get(), "Call flag 'a' should have fired");
        assert!(!call_flag_b.get(), "Call flag 'b' should NOT have fired");

        // Advance again
        time_manager.advance();

        // Time should now be at 7.0s
        assert_eq!(time_manager.get_time(), Time::new::<second>(7.0));
        assert!(call_flag_b.get(), "Call flag 'b' should have fired now");
    }


    #[test]
    fn emit_events_test() {

        // Create target Cells to grab our Events when they are emitted
        let a_evt_target: Cell<Option<Box<TestEventA>>> = Cell::new(None);
        let b_evt_target: Cell<Option<Box<TestEventB>>> = Cell::new(None);
        let call_flag_a: Cell<bool> = Cell::new(false);
        let call_flag_b: Cell<bool> = Cell::new(false);

        // Scope the time manager so we can just pull out the events at the end
        {
            let a_evt = TestEventA::new(Length::new::<meter>(3.5));
            let b_evt = TestEventB::new(AmountOfSubstance::new::<mole>(123456.0));
        
            // Create a time manager and a handy reusable
            // variable representing one second
            let mut time_manager = TimeManager::new();
            let one_sec = Time::new::<second>(1.0);
        
            // Set up our listener
            time_manager.on_event(|evt| {
                    if evt.is::<TestEventA>() {
                        match evt.downcast::<TestEventA>() {
                            Ok(evt_a) => {
                                a_evt_target.set(Some(evt_a));
                                call_flag_a.set(true);
                            },
                            Err(_) => {/*ignore*/}
                        }
                    }
                    else {
                        match evt.downcast::<TestEventB>() {
                            Ok(evt_b) => {
                                b_evt_target.set(Some(evt_b));
                                call_flag_b.set(true);
                            },
                            Err(_) => {/*ignore*/}
                        }
                    }
                });

            // Schedule the events to be emitted later
            time_manager.schedule_event(Time::new::<second>(2.0), a_evt);
            time_manager.schedule_event(Time::new::<second>(6.0), b_evt);

            // Advance by 1s. No events yet.
            time_manager.advance_by(one_sec);
            assert_eq!(time_manager.get_time(), Time::new::<second>(1.0));
            assert!(!call_flag_a.get(), "A Event should not have fired.");
            assert!(!call_flag_b.get(), "B Event should not have fired.");
            
            // Advance again. First event should fire.
            time_manager.advance_by(one_sec);
            assert_eq!(time_manager.get_time(), Time::new::<second>(2.0));
            assert!(call_flag_a.get(), "A Event should have fired.");
            assert!(!call_flag_b.get(), "B Event should not have fired.");
            
            // Advance again automatically. Should fire both now.
            time_manager.advance();
            assert!(call_flag_a.get(), "A Event should have fired.");
            assert!(call_flag_b.get(), "B Event should have fired.");
        }

        // Ensure we can pull the events and get their values
        assert!(a_evt_target.into_inner().unwrap().len == Length::new::<meter>(3.5));
        assert!(b_evt_target.into_inner().unwrap().amt == AmountOfSubstance::new::<mole>(123456.0));
    }
    #[test]

    fn evt_lis_mixed_test() {

        // Create target Cells to grab our Events when they are emitted
        let call_flag_a: Cell<bool> = Cell::new(false);
        let call_flag_b: Cell<bool> = Cell::new(false);
        let call_flag_c: Cell<bool> = Cell::new(false);
        let call_flag_d: Cell<bool> = Cell::new(false);

        let a_evt = TestEventA::new(Length::new::<meter>(3.5));
        let b_evt = TestEventB::new(AmountOfSubstance::new::<mole>(123456.0));
        
        // Create a time manager and a handy reusable
        // variable representing one second
        let mut time_manager = TimeManager::new();
        let one_sec = Time::new::<second>(1.0);
        
        // Set up our listener
        time_manager.on_event(|evt| {
                if evt.is::<TestEventA>() {
                    match evt.downcast::<TestEventA>() {
                        Ok(_) => {
                            call_flag_a.set(true);
                        },
                        Err(_) => {/*ignore*/}
                    }
                }
                else {
                    match evt.downcast::<TestEventB>() {
                        Ok(_) => {
                            call_flag_b.set(true);
                        },
                        Err(_) => {/*ignore*/}
                    }
                }
            });

        // Schedule a callback at 1s
        time_manager.schedule_callback(one_sec, || {
            call_flag_c.set(true);
        });

        // Schedule the events to be emitted later
        time_manager.schedule_event(Time::new::<second>(2.0), a_evt);
        time_manager.schedule_event(Time::new::<second>(6.0), b_evt);
        
        // Schedule a callback at 7s
        time_manager.schedule_callback(Time::new::<second>(7.0), || {
            call_flag_d.set(true);
        });

        // Advance by 1s. First callback should fire
        time_manager.advance_by(one_sec);
        assert!(!call_flag_a.get(), "A Event should not have fired.");
        assert!(!call_flag_b.get(), "B Event should not have fired.");
        assert!(call_flag_c.get(), "C Event should have fired.");
        assert!(!call_flag_d.get(), "D Event should not have fired.");
        
        // Advance again. Event A should fire now.
        time_manager.advance_by(one_sec);
        assert!(call_flag_a.get(), "A Event should have fired.");
        assert!(!call_flag_b.get(), "B Event should not have fired.");
        assert!(!call_flag_d.get(), "D Event should not have fired.");
        
        // Advance again automatically. Should fire B now.
        time_manager.advance();
        assert!(call_flag_b.get(), "B Event should have fired.");
        assert!(!call_flag_d.get(), "D Event should not have fired.");
        
        // Last advance. Should fire D now.
        time_manager.advance();
        assert!(call_flag_d.get(), "D Event should have fired.");
    }
}