macro_rules! impl_sim {
    ( $name:ident, $organism:ident ) => {
        pub struct $name {
            connector: crate::sim::SimConnector,
            layer_manager: crate::sim::layer::LayerManager<$organism>,
            id_gen: crate::util::IdGenerator,
            hub: crate::hub::EventHub<'static>,
        }

        static DEFAULT_ID_GEN: std::sync::OnceLock<std::sync::Mutex<crate::util::IdGenerator>> =
            std::sync::OnceLock::new();
        static DEFAULT_FACTORIES: std::sync::OnceLock<
            std::sync::Mutex<
                Vec<(
                    crate::util::IdType,
                    crate::sim::component::ComponentFactory<'_, $organism>,
                )>,
            >,
        > = std::sync::OnceLock::new();

        impl $name {
            fn default_id_gen() -> std::sync::MutexGuard<'static, crate::util::IdGenerator> {
                DEFAULT_ID_GEN
                    .get_or_init(|| std::sync::Mutex::new(crate::util::IdGenerator::new()))
                    .lock()
                    .unwrap()
            }

            fn default_factories() -> std::sync::MutexGuard<
                'static,
                Vec<(u32, crate::sim::component::ComponentFactory<'static, $organism>)>,
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
            pub fn set_default<T: crate::sim::component::SimComponent<$organism>>(
                factory: impl FnMut() -> T + 'static + Send,
            ) -> crate::util::IdType {
                let factory_id = Self::default_id_gen().get_id();
                Self::default_factories().push((
                    factory_id,
                    crate::sim::component::ComponentFactory::new(factory),
                ));
                factory_id
            }

            pub fn remove_default<T: crate::sim::component::SimComponent<$organism>>(
                factory_id: &crate::util::IdType,
            ) -> anyhow::Result<()> {
                if let Some((idx, _)) = Self::default_factories()
                    .iter()
                    .enumerate()
                    .find(|(_, (id, _f))| id == factory_id)
                {
                    Self::default_factories().remove(idx);

                    return Self::default_id_gen().return_id(*factory_id);
                };
                Err(anyhow!("Invalid factory_id provided"))
            }

            pub fn add_component(
                &mut self,
                component: impl crate::sim::component::SimComponent<$organism>,
            ) -> anyhow::Result<()> {
                self.layer_manager.add_component(&mut self.connector, component)?;
                Ok(())
            }
            
            fn init(mut layer_manager: crate::sim::layer::LayerManager<$organism>) -> Self {
                let mut connector = crate::sim::SimConnector::new();

                for (_, factory) in Self::default_factories().iter_mut() {
                    layer_manager.attach_component(&mut connector, |reg| factory.attach(reg))
                }
                
                Self {
                    id_gen: crate::util::IdGenerator::new(),
                    connector: connector,
                    hub: crate::hub::EventHub::new(),
                    layer_manager,
                }
            }

            pub fn new() -> Self {
                Self::init(crate::sim::layer::LayerManager::new())
            }
            
            pub fn new_threaded() -> Self {
                Self::init(crate::sim::layer::LayerManager::new_threaded())
            }
        }

        impl crate::sim::Sim for $name {
            fn time(&self) -> crate::sim::SimTime {
                self.connector.sim_time()
            }

            fn advance(&mut self) {
                self.connector.time_manager.advance();
                self.layer_manager.update(&mut self.connector);
            }

            fn advance_by(&mut self, time_step: crate::sim::SimTime) {
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
                self.layer_manager.remove_component(component_id)
            }

            fn schedule_event(
                &mut self,
                wait_time: crate::sim::SimTime,
                event: Box<dyn crate::event::Event>,
            ) -> crate::util::IdType {
                self.connector.time_manager.schedule_event(wait_time, event)
            }

            fn unschedule_event(
                &mut self,
                schedule_id: &crate::util::IdType,
            ) -> anyhow::Result<()> {
                self.connector.time_manager.unschedule_event(schedule_id)
            }

            fn drain_active(
                &mut self
            ) -> crate::event::EventDrainIterator {
                crate::event::EventDrainIterator(self.connector.active_events.drain(..))
            }
        }
    };
}

pub(crate) use impl_sim;
