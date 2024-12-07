#[macro_export]
macro_rules! impl_sim {
    ( $name:ident, $organism:ident ) => {
        pub struct $name {
            connector: $crate::sim::SimConnector,
            layer_manager: $crate::sim::layer::LayerManager<$organism>,
            id_gen: $crate::IdGenerator,
            hub: $crate::hub::EventHub<'static>,
        }

        static DEFAULT_ID_GEN: std::sync::OnceLock<std::sync::Mutex<$crate::IdGenerator>> =
            std::sync::OnceLock::new();
        static DEFAULT_FACTORIES: std::sync::OnceLock<
            std::sync::Mutex<
                Vec<(
                    $crate::IdType,
                    $crate::sim::component::ComponentFactory<'_, $organism>,
                )>,
            >,
        > = std::sync::OnceLock::new();

        impl $name {
            fn default_id_gen() -> std::sync::MutexGuard<'static, $crate::IdGenerator> {
                DEFAULT_ID_GEN
                    .get_or_init(|| std::sync::Mutex::new($crate::IdGenerator::new()))
                    .lock()
                    .unwrap()
            }

            fn default_factories() -> std::sync::MutexGuard<
                'static,
                Vec<(u32, $crate::sim::component::ComponentFactory<'static, $organism>)>,
            > {
                DEFAULT_FACTORIES
                    .get_or_init(|| std::sync::Mutex::new(Vec::new()))
                    .lock()
                    .unwrap()
            }

            /// Attaches a default factory function for a component which will be called
            /// whenever a new instance of the `Sim` is created, on which the factory-generated
            /// component will be registered by default.
            ///
            /// WARNING: when setting defaults, it is essential that two different factories
            /// do NOT produce components with the same id() value. In such a scenario,
            /// initialization of a Sim instance will fail since component ids MUST be unique
            /// for each instance.
            pub fn set_default<T: $crate::sim::component::SimComponent<$organism>>(
                factory: impl FnMut() -> T + 'static + Send,
            ) -> $crate::IdType {
                let factory_id = Self::default_id_gen().get_id();
                Self::default_factories().push((
                    factory_id,
                    $crate::sim::component::ComponentFactory::new(factory),
                ));
                factory_id
            }

            pub fn remove_default(
                factory_id: &$crate::IdType,
            ) -> anyhow::Result<()> {
                log::debug!("removing {}", factory_id);
                let mut found_idx = None;
                if let Some((idx, _)) = Self::default_factories()
                    .iter()
                    .enumerate()
                    .find(|(_, (id, _f))| id == factory_id)
                {
                    found_idx = Some(idx.clone());
                };

                if found_idx.is_some() {
                    Self::default_factories().remove(found_idx.unwrap());
                    return Self::default_id_gen().return_id(*factory_id);
                }
                Err(anyhow!("Invalid factory_id provided"))
            }

            pub fn add_component(
                &mut self,
                component: impl $crate::sim::component::SimComponent<$organism>,
            ) -> anyhow::Result<()> {
                self.layer_manager.add_component(&mut self.connector, component)?;
                Ok(())
            }
            
            fn init(mut layer_manager: $crate::sim::layer::LayerManager<$organism>) -> Self {
                let mut connector = $crate::sim::SimConnector::new();

                for (_, factory) in Self::default_factories().iter_mut() {
                    layer_manager.add_component_from_factory(&mut connector, factory).unwrap();
                }
                
                Self {
                    id_gen: $crate::IdGenerator::new(),
                    connector: connector,
                    hub: $crate::hub::EventHub::new(),
                    layer_manager,
                }
            }

            pub fn new() -> Self {
                Self::init($crate::sim::layer::LayerManager::new())
            }
            
            pub fn new_threaded() -> Self {
                Self::init($crate::sim::layer::LayerManager::new_threaded())
            }

            /// Retrieves a typed reference to an `Event` in this state
            ///
            /// returns a `&E` or `None` if no `Event` of this type has been set
            pub fn get_state<T: $crate::event::Event>(&self) -> Option<&T> {
                self.connector.state.get_state::<T>()
            }
    
            /// Retrieves a typed reference to an `Event` in this state
            ///
            /// returns a `&E` or `None` if no `Event` of this type has been set
            pub fn get_state_arc<T: $crate::event::Event>(&self) -> Option<std::sync::Arc<T>> {
                self.connector.state.get_state_arc::<T>()
            }

            /// Retrieves a dyn `Event` in this state
            ///
            /// returns a cloned `Arc<E>` or `None` if no `Event` of this type has been set
            pub fn get_dyn_state(&self, type_id: &std::any::TypeId) -> Option<&std::sync::Arc<dyn $crate::event::Event>> {
                self.connector.state.get_dyn_state(type_id)
            }

            /// Checks whether an `Event` exists in this state for a given `Event` type
            ///
            /// returns `true` if it exists or `false` otherwise
            pub fn has_state<T: $crate::event::Event>(&self) -> bool {
                self.connector.state.has_state::<T>()
            }
        }

        impl $crate::sim::Sim for $name {
            fn time(&self) -> $crate::sim::SimTime {
                self.connector.sim_time()
            }

            fn advance(&mut self) {
                if !self.layer_manager.first_update() {
                    self.layer_manager.update(&mut self.connector);
                }
                self.connector.time_manager.advance();
                self.layer_manager.update(&mut self.connector);
            }

            fn advance_by(&mut self, time_step: $crate::SimTimeSpan) {
                if !self.layer_manager.first_update() {
                    self.layer_manager.update(&mut self.connector);
                }
                self.connector.time_manager.advance_by(time_step);
                self.layer_manager.update(&mut self.connector);
            }

            fn active_components(&self) -> Vec<&'static str> {
                self.layer_manager.components().collect()
            }

            fn has_component(&self, component_id: &str) -> bool {
                self.layer_manager.has_component(component_id)
            }

            fn remove_component(&mut self, component_id: &str) -> anyhow::Result<&str> {
                Ok(self.layer_manager.remove_component(&mut self.connector, component_id)?.id())
            }

            fn schedule_event(
                &mut self,
                wait_time: $crate::SimTimeSpan,
                event: Box<dyn $crate::event::Event>,
            ) -> $crate::IdType {
                self.connector.time_manager.schedule_event(wait_time, event)
            }

            fn unschedule_event(
                &mut self,
                schedule_id: &$crate::IdType,
            ) -> anyhow::Result<()> {
                self.connector.time_manager.unschedule_event(schedule_id)
            }

            fn drain_active(
                &mut self
            ) -> $crate::event::EventDrainIterator {
                $crate::event::EventDrainIterator(self.connector.active_events.drain(..))
            }
        }
    };
}

pub use impl_sim;
