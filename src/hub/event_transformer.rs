use crate::event::Event;
use crate::util::id_gen::{IdGenerator, IdType};
use std::any::TypeId;
use std::cmp;
use std::fmt;
use std::sync::{Mutex, MutexGuard, OnceLock};

static ID_GEN: OnceLock<Mutex<IdGenerator>> = OnceLock::new();

pub trait EventTransformer: Send + Sync {
    /// Calls this transformer's handler function with the given Event
    ///
    /// ### Arguments
    /// * `evt` - Event to dispatch to the handler function
    fn transform(&mut self, evt: &mut dyn Event);

    /// Retrieves the priority value for this transformer
    fn priority(&self) -> i32;

    /// Retrieves the id for this listener
    fn transformer_id(&self) -> IdType;

    /// Retrieves the TypeId for the underlying Event type
    fn type_id(&self) -> TypeId;
}

impl<'a> fmt::Debug for dyn EventTransformer + 'a {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EventTransformer<{:?}> {{ priority: {:?} }}",
            self.transformer_id(),
            self.priority()
        )
    }
}

// Implement all the traits we need to support Ord
impl<'a> PartialEq for dyn EventTransformer + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.transformer_id() == other.transformer_id()
    }
}

impl<'a> PartialOrd for dyn EventTransformer + 'a {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if other.priority() == self.priority() {
            if self.eq(other) {
                Some(cmp::Ordering::Equal)
            } else {
                self.transformer_id().partial_cmp(&other.transformer_id())
            }
        } else {
            other.priority().partial_cmp(&self.priority())
        }
    }
}

impl<'a> Eq for dyn EventTransformer + 'a {}

impl<'a> Ord for dyn EventTransformer + 'a {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if other.priority() == self.priority() {
            if self.eq(other) {
                cmp::Ordering::Equal
            } else {
                self.transformer_id().cmp(&other.transformer_id())
            }
        } else {
            other.priority().cmp(&self.priority())
        }
    }
}

pub struct TransformerItem<'a, T: Event> {
    /// Unique identifier for this listener
    transformer_id: IdType,
    /// Container for the Event transforming function
    handler: Box<dyn FnMut(&mut T) + Send + Sync + 'a>,
    /// Priority for this transformer
    priority: i32,
}

impl<'a, T: Event> TransformerItem<'a, T> {
    fn id_gen() -> MutexGuard<'static, IdGenerator> {
        ID_GEN.get_or_init(|| {
            Mutex::new(IdGenerator::new())
        }).lock().unwrap()
    }

    /// Creates a new TransformerItem for the given handler with
    /// the default priority of 0
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    pub fn new(handler: impl FnMut(&mut T) + Send + Sync + 'a) -> TransformerItem<'a, T> {
        TransformerItem {
            transformer_id: Self::id_gen().get_id(),
            handler: Box::new(handler),
            priority: 0,
        }
    }
    /// Creates a new TransformerItem for the given handler and
    /// priority of execution
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Event transforming function
    /// * `priority` - determines this transformer's priority when Events
    ///                are dispatched. Higher priority transformers are
    ///                executed first.
    pub fn new_prioritized(
        handler: impl FnMut(&mut T) + Send + Sync + 'a,
        priority: i32,
    ) -> TransformerItem<'a, T> {
        TransformerItem {
            transformer_id: Self::id_gen().get_id(),
            handler: Box::new(handler),
            priority: priority,
        }
    }
}

impl<'a, T: Event> Drop for TransformerItem<'a, T> {
    fn drop(&mut self) {
        // Return ids back to the pool when listeners are dropped
        Self::id_gen()
            .return_id(self.transformer_id)
            .unwrap();
    }
}

impl<'a, T: Event> EventTransformer for TransformerItem<'a, T> {
    fn transform(&mut self, evt: &mut dyn Event) {
        match evt.downcast_mut::<T>() {
            Some(typed_evt) => (*self.handler)(typed_evt),
            None => panic!("Ahhh! Transformer {} is on fire!!!", self.transformer_id),
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn transformer_id(&self) -> IdType {
        self.transformer_id
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::EventTransformer;
    use super::TransformerItem;
    use crate::event::test::TestEventA;
    use crate::units::base::Distance;

    #[test]
    fn test_handle() {
        let mut listener = TransformerItem::new(|evt: &mut TestEventA| {
            evt.len = Distance::from_m(10.0);
        });

        let mut evt = TestEventA::new(Distance::from_m(5.0));
        assert_eq!(evt.len, Distance::from_m(5.0));

        listener.transform(&mut evt);
        assert_eq!(evt.len, Distance::from_m(10.0));
    }

    #[test]
    fn test_ord() {
        let transformer1 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 0);
        let transformer2 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 5);
        let transformer3 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, -2);
        let transformer4 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 3);

        let mut v = Vec::<Box<dyn EventTransformer>>::new();

        v.push(Box::new(transformer1));
        v.push(Box::new(transformer2));
        v.push(Box::new(transformer3));
        v.push(Box::new(transformer4));

        v.sort();

        assert_eq!(v[0].priority(), 5);
        assert_eq!(v[1].priority(), 3);
        assert_eq!(v[2].priority(), 0);
        assert_eq!(v[3].priority(), -2);
    }
}
