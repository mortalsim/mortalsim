use crate::event::Event;


pub trait NerveSignalTransformer: Send {
    fn transform(&mut self, message: &'_ mut dyn Event) -> Option<()>;
}

pub struct TransformFn<'a, T>(pub Box<dyn FnMut(&'_ mut T) -> Option<()> + Send + 'a>);

impl<'a, T: Event> NerveSignalTransformer for TransformFn<'a, T> {
    fn transform(&mut self, message: &mut dyn Event) -> Option<()> {
        self.0(message.downcast_mut::<T>().unwrap())
    }
}
