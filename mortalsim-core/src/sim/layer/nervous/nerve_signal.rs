use std::any::TypeId;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use either::Either;

use crate::event::Event;
use crate::sim::{Organism, SimTime};
use crate::IdGenerator;
use crate::IdType;

use super::{Nerve, NerveIter};

static ID_GEN: OnceLock<Mutex<IdGenerator>> = OnceLock::new();

#[derive(Clone)]
pub struct NerveSignal<O: Organism> {
    id: IdType,
    path: Arc<Vec<O::NerveType>>,
    message: Arc<dyn Event>,
    send_time: SimTime,
    message_type_id: TypeId,
}

impl<O: Organism> NerveSignal<O> {
    pub fn new<T: Event>(
        message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<Self> {
        if send_time < SimTime::from_s(0.0) {
            return Err(anyhow!("Invalid send time provided: {}", send_time));
        }
        Self::check_neural_path(&neural_path)?;

        Ok(Self {
            id: ID_GEN
                .get_or_init(|| Mutex::new(IdGenerator::new()))
                .lock()
                .unwrap()
                .get_id(),
            path: Arc::new(neural_path),
            message: Arc::new(message),
            send_time,
            message_type_id: TypeId::of::<T>()
        })
    }

    fn check_neural_path(neural_path: &Vec<O::NerveType>) -> anyhow::Result<()> {
        if neural_path.is_empty() {
            return Err(anyhow!("Neural path cannot be empty!"));
        }
        for idx in 0..(neural_path.len() - 1) {
            let cur_nerve = neural_path.get(idx).unwrap();
            let next_nerve = neural_path.get(idx + 1).unwrap();
            // Ensure each section of the path is valid
            if !cur_nerve.downlink().any(|d| d == *next_nerve) {
                return Err(anyhow!("Invalid link from {} to {}", cur_nerve, next_nerve));
            }
        }
        Ok(())
    }

    pub fn id(&self) -> IdType {
        self.id
    }

    pub fn neural_path(&self) -> NerveIter<O::NerveType> {
        NerveIter(self.path.iter())
    }

    pub fn terminating_nerve(&self) -> O::NerveType {
        *self.path.last().unwrap()
    }

    pub fn send_time(&self) -> SimTime {
        self.send_time
    }

    pub fn message_type_id(&self) -> TypeId {
        self.message_type_id
    }
    
    pub fn message_is<T: Event>(&self) -> bool {
        self.message.is::<T>()
    }

    pub fn message<T: Event>(&self) -> &'_ T {
        self.message
            .downcast_ref::<T>()
            .expect("Invalid message type")
    }

    pub fn dyn_message(&self) -> &dyn Event {
        self.message.as_ref()
    }

    /// Attempts to get a mutable reference to the message
    /// event. If there are existing Arc instances of it
    /// this will panic.
    pub fn dyn_message_mut(&mut self) -> &mut dyn Event {
        Arc::get_mut(&mut self.message).unwrap()
    }
}

impl<O: Organism> Drop for NerveSignal<O> {
    fn drop(&mut self) {
        if let Some(gen) = ID_GEN.get() {
            gen.lock().unwrap().return_id(self.id()).unwrap();
        }
    }
}

#[cfg(test)]
pub mod test {
    use simple_si_units::base::Distance;

    use crate::event::test::TestEventA;
    use crate::sim::organism::test::{TestNerve, TestOrganism};
    use crate::sim::SimTime;

    use super::NerveSignal;

    #[test]
    fn new_signal() {
        assert!(NerveSignal::<TestOrganism>::new(
            TestEventA::new(Distance::from_m(1.0)),
            vec![TestNerve::Brain, TestNerve::SpinalCord],
            SimTime::from_s(1.0)
        ).is_ok());
    }

    #[test]
    fn new_empty_signal() {
        assert!(NerveSignal::<TestOrganism>::new(
            TestEventA::new(Distance::from_m(1.0)),
            vec![],
            SimTime::from_s(1.0)
        ).is_err());
    }

    #[test]
    fn new_neg_signal() {
        assert!(NerveSignal::<TestOrganism>::new(
            TestEventA::new(Distance::from_m(1.0)),
            vec![TestNerve::Brain, TestNerve::SpinalCord],
            SimTime::from_s(-1.0)
        ).is_err());
    }

    #[test]
    fn new_invalid_signal() {
        assert!(NerveSignal::<TestOrganism>::new(
            TestEventA::new(Distance::from_m(1.0)),
            vec![TestNerve::Brain, TestNerve::LeftFemoral],
            SimTime::from_s(1.0)
        ).is_err());
    }
}