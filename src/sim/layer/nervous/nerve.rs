use std::any::{Any, TypeId};
use std::borrow::BorrowMut;
use std::collections::hash_set;
use std::{fmt, vec};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use downcast_rs::DowncastSync;
use uuid::Uuid;

use crate::event::Event;
use crate::sim::SimTime;
use crate::sim::layer::AnatomicalRegionIter;
use crate::sim::organism::Organism;
use crate::util::{IdType, IdGenerator};

pub trait Nerve:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Sync + Into<&'static str>
{
    type AnatomyType: Clone;
    fn terminal_nerves<'a>() -> NerveIter<'a, Self>;
    fn uplink<'a>(&self) -> NerveIter<'a, Self>;
    fn downlink<'a>(&self) -> NerveIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

pub struct NerveIter<'a, N: Nerve>(pub core::slice::Iter<'a, N>);

impl<'a, N: Nerve> Iterator for NerveIter<'a, N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        Some(self.0.next()?.clone())
    }
}

impl<'a, N: Nerve> ExactSizeIterator for NerveIter<'a, N> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct NerveSignal<O: Organism>  {
    id: Uuid,
    path: Vec<O::NerveType>,
    message: Box<dyn Event>,
    send_time: SimTime,
    blocked: bool,
}

impl<O: Organism + 'static> NerveSignal<O> {
    pub fn new<T: Event>(message: T, neural_path: Vec<O::NerveType>, send_time: SimTime) -> anyhow::Result<Self> {
        if neural_path.is_empty() {
            return Err(anyhow!("Neural path cannot be empty!"));
        }
        for idx in 0..(neural_path.len()-1) {
            let cur_nerve = neural_path.get(idx).unwrap();
            let next_nerve = neural_path.get(idx+1).unwrap();
            // Ensure each section of the path is valid
            if !cur_nerve.downlink().any(|d| d == *next_nerve) {
                return Err(anyhow!("Invalid link from {} to {}", cur_nerve, next_nerve));
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            path: neural_path,
            message: Box::new(message),
            send_time,
            blocked: false,
        })
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn is_blocked(&self) -> bool {
        self.blocked
    }
    
    pub fn block(&mut self) {
        self.blocked = true;
    }
    
    pub fn unblock(&mut self) {
        self.blocked = false;
    }

    pub fn neural_path(&self) -> NerveIter<O::NerveType> {
        NerveIter(self.path.iter())
    }
    
    pub fn send_time(&self) -> SimTime {
        self.send_time
    }

    pub fn type_id(&self) -> TypeId {
        self.message.type_id()
    }

    pub fn message<T: Event>(&self) -> &'_ T {
        self.message.downcast_ref::<T>().expect("Invalid message type")
    }

    pub fn message_mut<T: Event>(&mut self) -> &'_ mut T {
        self.message.downcast_mut::<T>().expect("Invalid message type")
    }

    pub fn into_message<T: Event>(self) -> anyhow::Result<T> {
        match self.message.downcast::<T>() {
            Ok(msg) => Ok(*msg),
            Err(_) => Err(anyhow!("Invalid message type attempted"))
        }
    }
}
