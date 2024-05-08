use crate::event::Event;


pub trait NerveSignalTransformer: Send {
    /// Transforms the given message, returning the given reference
    /// back to the caller unless the message should be cancelled
    fn transform<'b>(&mut self, message: &'b mut dyn Event) -> Option<&'b mut dyn Event>;
}

pub struct TransformFn<'a, T>(pub Box<dyn FnMut(&'_ mut T) -> Option<&'_ mut T> + Send + 'a>);

impl<'a, T: Event> NerveSignalTransformer for TransformFn<'a, T> {
    /// Transforms the given message, returning the given reference
    /// back to the caller unless the message should be cancelled
    fn transform<'b>(&mut self, message: &'b mut dyn Event) -> Option<&'b mut dyn Event> {
        self.0(message.downcast_mut::<T>().unwrap()).map(|x| x as &mut dyn Event)
    }
}
