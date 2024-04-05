use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use downcast_rs::Downcast;

use crate::event::Event;
use crate::sim::layer::nervous::NerveSignal;
use crate::sim::layer::nervous::transform::{TransformFn, NerveSignalTransformer};
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::util::{IdGenerator, IdType, OrderedTime};

pub struct NervousConnector<O: Organism> {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, Vec<NerveSignal<O>>>,
    /// Outgoing signals
    pub(crate) outgoing: Vec<NerveSignal<O>>,
    /// Scheduled signals
    pub(crate) scheduled_signals: HashMap<IdType, OrderedTime>,
    /// Transformations to add
    pub(crate) adding_transforms:
        HashMap<O::NerveType, HashMap<TypeId, Box<dyn NerveSignalTransformer>>>,
    /// Map of registered transformations
    pub(crate) registered_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// Map of removing transformations
    pub(crate) removing_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// List of signal ids to unschedule
    pub(crate) pending_unschedules: Vec<(OrderedTime, IdType)>,
    /// Empty Event list for ergonomic message use
    empty: Vec<NerveSignal<O>>,
}

impl<O: Organism> NervousConnector<O> {
    pub fn new() -> Self {
        Self {
            sim_time: SimTime::from_s(0.0),
            incoming: HashMap::new(),
            outgoing: Vec::new(),
            scheduled_signals: HashMap::new(),
            adding_transforms: HashMap::new(),
            registered_transforms: HashMap::new(),
            removing_transforms: HashMap::new(),
            pending_unschedules: Vec::new(),
            empty: Vec::new(),
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    fn extract_message<T: Event>(s: &NerveSignal<O>) -> (O::NerveType, &'_ T) {
        (s.terminating_nerve(), s.message::<T>())
    }
    
    pub fn get_messages<T: Event>(&self) -> impl Iterator<Item = (O::NerveType, &'_ T)> {
        match self.incoming.get(&TypeId::of::<T>()) {
            Some(signals) => either::Left(signals.iter().map(Self::extract_message)),
            None => either::Right(self.empty.iter().map(Self::extract_message)),
        }
    }
    
    pub fn send_message<T: Event>(
        &mut self,
        message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<IdType> {
        if send_time <= self.sim_time {
            return Err(anyhow!(
                "Invalid send_time: time must be greater than the current time!"
            ));
        }

        let signal = NerveSignal::new(message, neural_path, send_time)?;

        self.scheduled_signals.insert(signal.id(), OrderedTime(signal.send_time()));
        let signal_id = signal.id();
        self.outgoing.push(signal);
        Ok(signal_id)
    }

    pub fn transform_message<T: Event>(
        &mut self,
        nerve: O::NerveType,
        handler: impl (FnMut(&mut T) -> Option<&mut T>) + Send + 'static,
    ) {

        self.adding_transforms
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>(), Box::new(TransformFn(Box::new(handler))));
    }

    pub fn stop_transform<T: 'static>(&mut self, nerve: O::NerveType) -> anyhow::Result<()> {
        if let Some(type_map) = self.registered_transforms.get(&nerve) {
            if type_map.contains_key(&TypeId::of::<T>()) {
                let type_map = self.registered_transforms.remove(&nerve).unwrap();
                self.removing_transforms.insert(nerve, type_map);
                return Ok(());
            }
        }
        Err(anyhow!("Transformation not registered for {}", nerve))
    }

    /// Unschedules an `Event` which has been scheduled previously.
    ///
    /// ### Arguments
    /// * `signal_id` - id of the signal to unschedule
    ///
    /// Returns Ok if the id is valid, and Err otherwise
    pub fn unschedule_signal(&mut self, signal_id: IdType) -> anyhow::Result<()> {
        if let Some(signal_time) = self.scheduled_signals.remove(&signal_id) {
            if signal_time.0 > self.sim_time {
                self.pending_unschedules.push((signal_time, signal_id));
                return Ok(())
            }
            else {
                return Err(anyhow!("Invalid schedule_id provided"))
            }
        }
        Err(anyhow!("Invalid schedule_id provided"))
    }
}

#[cfg(test)]
pub mod test {
    use std::any::TypeId;
    use std::collections::HashMap;

    use crate::sim::layer::circulation::component::connector;
    use crate::sim::layer::nervous::component::test::{MovementEvent, PainEvent, TestPainReflexComponent};
    use crate::sim::layer::nervous::NerveSignal;
    use crate::sim::organism::test::{TestAnatomicalRegion, TestNerve, TestOrganism};
    use crate::sim::SimTime;

    use super::NervousConnector;

    #[test]
    fn get_messages() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        connector.incoming.insert(TypeId::of::<PainEvent>(), vec![
            NerveSignal::new(
                PainEvent {level: 50, region: TestAnatomicalRegion::RightArm},
                TestPainReflexComponent::right_arm_path(),
                SimTime::from_s(1.0),
            ).unwrap(),
            NerveSignal::new(
                PainEvent {level: 70, region: TestAnatomicalRegion::LeftArm},
                TestPainReflexComponent::left_arm_path(),
                SimTime::from_s(1.0),
            ).unwrap(),
        ]);
    }

    #[test]
    fn send_message() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        connector.send_message(
            MovementEvent {amount: 1},
            TestPainReflexComponent::left_arm_path(),
            SimTime::from_min(1.0),
        ).unwrap();

        assert_eq!(connector.outgoing.len(), 1);
        assert!(connector.outgoing.get(0).unwrap().message_is::<MovementEvent>());
    }

    #[test]
    fn send_bad_message() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        assert!(connector.send_message(
            MovementEvent {amount: 1},
            TestPainReflexComponent::left_arm_path(),
            SimTime::from_min(-1.0),
        ).is_err());
    }

    #[test]
    fn transform_message() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        connector.transform_message::<MovementEvent>(TestNerve::SpinalCord, |e| {
            e.amount += 10;
            Some(e)
        });
    }

    #[test]
    fn stop_transform() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        let mut id_map = HashMap::new();
        id_map.insert(TypeId::of::<MovementEvent>(), 1);
        connector.registered_transforms.insert(TestNerve::SpinalCord, id_map);

        assert!(connector.stop_transform::<MovementEvent>(TestNerve::SpinalCord).is_ok());

        assert_eq!(connector.removing_transforms.len(), 1);
        assert!(connector.removing_transforms.get(&TestNerve::SpinalCord).is_some_and(|x| {
            x.get(&TypeId::of::<MovementEvent>()).unwrap() == &1
        }));
    }

    #[test]
    fn stop_invalid_transform() {
        let mut connector = NervousConnector::<TestOrganism>::new();
        assert!(connector.stop_transform::<MovementEvent>(TestNerve::SpinalCord).is_err());
    }
}
