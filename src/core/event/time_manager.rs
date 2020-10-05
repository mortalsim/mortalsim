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
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::util::quantity_wrapper::OrderedTime;

fn execute_listeners<'a>(manager_id: Uuid, time: OrderedTime,
    listeners: &mut Vec<(u32, std::boxed::Box<dyn std::ops::FnMut() + 'a>)>) {

    for (_, listener) in listeners {
        log::debug!("TimeManager {} executing listener scheduled for {}",
            manager_id, time.get_value().into_format_args(second, Abbreviation));
        listener();
    }
}

fn emit_events<'a>(manager_id: Uuid, time: OrderedTime, events: &mut Vec<(IdType, Box<dyn Event + 'a>)>,
    event_listener: &mut Option<Box<dyn FnMut(Box<dyn Event + 'a>) + 'a>>) {

    log::debug!("TimeManager {} emitting events scheduled for {}",
        manager_id, time.get_value().into_format_args(second, Abbreviation));

    // Get a mutable reference to the inner function
    match event_listener.as_mut() {
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

struct TimeManager<'a> {
    /// Identifier for this TimeManager object
    manager_id: Uuid,
    /// Current simulation time
    sim_time: Time,
    /// Sorted map of events to be executed
    event_queue: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn Event + 'a>)>>,
    /// Generator for our listener IDs
    id_gen: IdGenerator,
    /// Map of listeners for any time advance
    advance_listeners: HashMap<IdType, Box<dyn FnMut() + 'a>>,
    /// Map of listeners to execute at future simulation times
    scheduled_listeners: BTreeMap<OrderedTime, Vec<(IdType, Box<dyn FnMut() + 'a>)>>,
    /// Vector of Event listeners to call when events are emitted
    event_listener: Option<Box<dyn FnMut(Box<dyn Event + 'a>) + 'a>>,
    /// Used to lookup listeners and Event objects for unscheduling
    id_time_map: HashMap<IdType, OrderedTime>
}

impl<'a> TimeManager<'a> {
    /// Creates a new TimeManager object starting at t = 0
    pub fn new() -> TimeManager<'a> {
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
    pub fn schedule_event(&mut self, wait_time: Time, event: impl Event + 'a) -> IdType {
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

        // Add the id, event tuple to the event list
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
                        panic!("A scheduled ID refers to an invalid time!");
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
    pub fn on_event(&mut self, listener: impl FnMut(Box<dyn Event + 'a>) + 'a) {
        self.event_listener = Some(Box::new(listener));
    }

    // /// Get a reference to the internal `Event` queue (implemented as a BTreeMap)
    // /// 
    // /// Returns a reference to the current `Event` queue
    // pub fn event_queue_ref(&self) -> &BTreeMap<OrderedTime, Vec<Box<dyn Event>>> {
    //     &self.event_queue
    // }

    /// Registers a listener for time advances
    /// 
    /// # Arguments
    /// * `listener` - function to call when time advances
    pub fn on_advance(&self, listener: impl FnMut() + 'a) -> IdType {
        0
    }

    /// Unregisters a previously attached time advance listener
    /// 
    /// # Arguments
    /// * `listener_id` - identifier returned from the call to `on_advance`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn off_advance(&self, listener_id: IdType) -> Result<()> {
        Ok(())
    }

    /// Schedules a callback to be called at a future simulation time
    /// 
    /// # Arguments
    /// * `wait_time` - amount of simulation time to wait before calling the listener
    /// * `listener` - function to call at the scheduled time
    pub fn schedule_callback(&self, wait_time: Time, listener: impl FnMut() + 'a) -> IdType {
        0
    }

    /// Unschedules a previously scheduled listener
    /// 
    /// # Arguments
    /// * `listener_id` - The identifier returned from the call to `schedule_callback`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn unschedule_callback(&self, listener_id: IdType) -> Result<()> {
        Ok(())
    }


    /// Internal function for triggering
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
        let lis_iter = self.scheduled_listeners.iter_mut();
        let evt_iter = self.event_queue.iter_mut();

        // Keep track of the maximum time reached while we're iterating both
        let mut reached_time = Time::new::<second>(0.0);

        for ((lis_time, listeners), (evt_time, events)) in lis_iter.zip(evt_iter) {

            // If both times are beyond the current simulation time, exit the loop
            if lis_time.get_value() > self.sim_time && evt_time.get_value() > self.sim_time {
                break;
            }

            if lis_time < evt_time {
                // The listeners come first, so we need to call all of those associated
                // with the given time step
                execute_listeners(self.manager_id, *lis_time, listeners);

                // Add this time to the list of completed times
                times_completed.push(*lis_time);
                reached_time = lis_time.get_value();
            }
            else {
                // The events come first, so we need to emit all of those to any
                // event listeners
                emit_events(self.manager_id, *evt_time, events, &mut self.event_listener);
                
                // Add this time to the list of completed times
                times_completed.push(*evt_time);
                reached_time = evt_time.get_value();
            }
        }


        // iterate over the scheduled_listeners until we reach a time which is
        // still in the future. Call each listener in those times and keep track
        // of completed times so we can remove them later.
        for (time, listeners) in self.scheduled_listeners.iter_mut() {
            if time.get_value() > self.sim_time {
                // Go until we reach 
                break;
            }
            if time.get_value() < reached_time {
                // If we've already processed these, continue to the next iteration
                continue;
            }
            execute_listeners(self.manager_id, *time, listeners);

            // Add this time to the list of completed times
            times_completed.push(*time);
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

}

