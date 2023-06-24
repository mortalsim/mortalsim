
use std::collections::HashMap;
use super::{wrapper::ComponentWrapper, SimComponent};

pub struct ComponentRegistry<'a>(HashMap<&'a str, Box<dyn ComponentWrapper>>);

impl<'a> ComponentRegistry<'a> {
  // TODO: create add_{system}Component methods for each combination of systems
}
