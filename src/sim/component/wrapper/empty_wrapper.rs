// Looked into creating a macro that supports multiple trait bounds,
// but apparently that's not well supported... very annoying. So I
// may need to revisit this in the future.
// See 
//  - https://github.com/rust-lang/rfcs/issues/2520
//  - https://stackoverflow.com/questions/51579647/how-to-match-trait-bounds-in-a-macro

pub static MSG: &'static str = "Component type not implemented!";

macro_rules! empty_core_wrapper {
    ( $target:ty, $component_type:path) => {
        impl<O: crate::sim::Organism, T: $component_type> crate::sim::layer::CoreComponent<O> for $target {
            fn core_init(&mut self, _initializer: &mut crate::sim::layer::CoreComponentInitializer) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn core_connector(&mut self) -> &mut crate::sim::layer::CoreConnector { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path) => {
        impl<O: crate::sim::Organism, T: $component_type + $component_type2> crate::sim::layer::CoreComponent<O> for $target {
            fn core_init(&mut self, _initializer: &mut crate::sim::layer::CoreComponentInitializer) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn core_connector(&mut self) -> &mut crate::sim::layer::CoreConnector { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path, $component_type3:path) => {
        impl<O: crate::sim::Organism, T: $component_type + $component_type2 + $component_type3> crate::sim::layer::CoreComponent<O> for $target {
            fn core_init(&mut self, _initializer: &mut crate::sim::layer::CoreComponentInitializer) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn core_connector(&mut self) -> &mut crate::sim::layer::CoreConnector { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
}

macro_rules! empty_cc_wrapper {
    ( $target:ty, $component_type:path ) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type> crate::sim::layer::ClosedCircComponent<O> for $target {
            fn cc_init(&mut self, _initializer: &mut crate::sim::layer::ClosedCircInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn cc_connector(&mut self) -> &mut crate::sim::layer::ClosedCircConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2> crate::sim::layer::ClosedCircComponent<O> for $target {
            fn cc_init(&mut self, _initializer: &mut crate::sim::layer::ClosedCircInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn cc_connector(&mut self) -> &mut crate::sim::layer::ClosedCircConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path, $component_type3:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2 + $component_type3> crate::sim::layer::ClosedCircComponent<O> for $target {
            fn cc_init(&mut self, _initializer: &mut crate::sim::layer::ClosedCircInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn cc_connector(&mut self) -> &mut crate::sim::layer::ClosedCircConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
}

macro_rules! empty_digestion_wrapper {
    ( $target:ty, $component_type:path ) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type> crate::sim::layer::DigestionComponent<O> for $target {
            fn digestion_init(&mut self, _initializer: &mut crate::sim::layer::DigestionInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn digestion_connector(&mut self) -> &mut crate::sim::layer::DigestionConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2> crate::sim::layer::DigestionComponent<O> for $target {
            fn digestion_init(&mut self, _initializer: &mut crate::sim::layer::DigestionInitializer) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn digestion_connector(&mut self) -> &mut crate::sim::layer::DigestionConnector { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path, $component_type3:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2 + $component_type3> crate::sim::layer::DigestionComponent<O> for $target {
            fn digestion_init(&mut self, _initializer: &mut crate::sim::layer::DigestionInitializer) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn digestion_connector(&mut self) -> &mut crate::sim::layer::DigestionConnector { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
}

macro_rules! empty_nervous_wrapper {
    ( $target:ty, $component_type:path ) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type> crate::sim::layer::NervousComponent<O> for $target {
            fn nervous_init(&mut self, _initializer: &mut crate::sim::layer::NervousInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn nervous_connector(&mut self) -> &mut crate::sim::layer::NervousConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2> crate::sim::layer::NervousComponent<O> for $target {
            fn nervous_init(&mut self, _initializer: &mut crate::sim::layer::NervousInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn nervous_connector(&mut self) -> &mut crate::sim::layer::NervousConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
    ( $target:ty, $component_type:path, $component_type2:path, $component_type3:path) => {
        impl<O: crate::sim::Organism + 'static, T: $component_type + $component_type2 + $component_type3> crate::sim::layer::NervousComponent<O> for $target {
            fn nervous_init(&mut self, _initializer: &mut crate::sim::layer::NervousInitializer<O>) { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
            fn nervous_connector(&mut self) -> &mut crate::sim::layer::NervousConnector<O> { panic!("{}", crate::sim::component::wrapper::empty_wrapper::MSG) }
        }
    };
}

pub(crate) use empty_core_wrapper;
pub(crate) use empty_cc_wrapper;
pub(crate) use empty_digestion_wrapper;
pub(crate) use empty_nervous_wrapper;
