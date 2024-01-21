
#![allow(dead_code)]
#[derive(Debug)]
pub struct InitializerA {}
#[derive(Debug)]
pub struct ConnectorA {}
pub trait ModuleA: Module {
    fn init_a(&mut self, initializer: &mut InitializerA);
    fn connector_a(&mut self) -> &mut ConnectorA;
}

#[derive(Debug)]
pub struct ComponentA {}
impl ComponentA {
    fn init_a(&mut self, module: &mut (impl ModuleA + ?Sized)) {
        module.init_a(&mut InitializerA {});
    }
    fn prepare_a(&mut self, module: &mut (impl ModuleA + ?Sized)) {
        module.connector_a();
    }
    fn process_a(&mut self, module: &mut (impl ModuleA + ?Sized)) {
        module.connector_a();
    }
}

pub struct InitializerB {}
pub struct ConnectorB {}
pub trait ModuleB: Module {
    fn init_b(&mut self, initializer: &mut InitializerB);
    fn connector_b(&mut self) -> &mut ConnectorB;
}

pub struct ComponentB {}
impl ComponentB {
    fn init_b(&mut self, module: &mut (impl ModuleB + ?Sized)) {
        module.init_b(&mut InitializerB {});
    }
    fn prepare_b(&mut self, module: &mut (impl ModuleB + ?Sized)) {
        module.connector_b();
    }
    fn process_b(&mut self, module: &mut (impl ModuleB + ?Sized)) {
        module.connector_b();
    }
}

pub trait Module {
    fn attach(self, keeper: &mut ModuleKeeper);
    fn run(&mut self);
}

pub trait ModuleWrapper: ModuleA + ModuleB {
    fn has_a(&self) -> bool;
    fn has_b(&self) -> bool;
}

#[derive(Debug)]
struct ModuleAWrapper<T: ModuleA + 'static>(T);

impl<T: ModuleA> ModuleWrapper for ModuleAWrapper<T> {
    fn has_a(&self) -> bool {
        true
    }
    fn has_b(&self) -> bool {
        false
    }
}

impl<T: ModuleA> Module for ModuleAWrapper<T> {
    fn attach(self, keeper: &mut ModuleKeeper) {
        keeper.add_a_module(self.0);
    }
    fn run(&mut self) {
        self.0.run()
    }
}

impl<T: ModuleA> ModuleA for ModuleAWrapper<T> {
    fn init_a(&mut self, _initializer: &mut InitializerA) {}
    fn connector_a(&mut self) -> &mut ConnectorA {
        self.0.connector_a()
    }
}

impl<T: ModuleA> ModuleB for ModuleAWrapper<T> {
    fn init_b(&mut self, _initializer: &mut InitializerB) {}
    fn connector_b(&mut self) -> &mut ConnectorB {
        panic!("invalid call for ConnectorB on ModuleAWrapper");
    }
}

struct ModuleBWrapper<T: ModuleB + 'static>(T);

impl<T: ModuleB> ModuleWrapper for ModuleBWrapper<T> {
    fn has_a(&self) -> bool {
        false
    }
    fn has_b(&self) -> bool {
        true
    }
}

impl<T: ModuleB> Module for ModuleBWrapper<T> {
    fn attach(self, keeper: &mut ModuleKeeper) {
        keeper.add_b_module(self.0);
    }
    fn run(&mut self) {
        self.0.run()
    }
}

impl<T: ModuleB> ModuleA for ModuleBWrapper<T> {
    fn init_a(&mut self, _initializer: &mut InitializerA) {}
    fn connector_a(&mut self) -> &mut ConnectorA {
        panic!("invalid call for ConnectorA on ModuleBWrapper");
    }
}

impl<T: ModuleB> ModuleB for ModuleBWrapper<T> {
    fn init_b(&mut self, _initializer: &mut InitializerB) {}
    fn connector_b(&mut self) -> &mut ConnectorB {
        self.0.connector_b()
    }
}

struct ModuleABWrapper<T: ModuleA + ModuleB + 'static>(T);

impl<T: ModuleA + ModuleB> ModuleWrapper for ModuleABWrapper<T> {
    fn has_a(&self) -> bool {
        true
    }
    fn has_b(&self) -> bool {
        true
    }
}

impl<T: ModuleA + ModuleB> Module for ModuleABWrapper<T> {
    fn attach(self, keeper: &mut ModuleKeeper) {
        keeper.add_ab_module(self.0);
    }
    fn run(&mut self) {
        self.0.run()
    }
}

impl<T: ModuleA + ModuleB> ModuleA for ModuleABWrapper<T> {
    fn init_a(&mut self, _initializer: &mut InitializerA) {}
    fn connector_a(&mut self) -> &mut ConnectorA {
        self.0.connector_a()
    }
}

impl<T: ModuleA + ModuleB> ModuleB for ModuleABWrapper<T> {
    fn init_b(&mut self, _initializer: &mut InitializerB) {}
    fn connector_b(&mut self) -> &mut ConnectorB {
        self.0.connector_b()
    }
}

#[derive(Debug)]
pub struct ModuleSoleA {
    connector_a: ConnectorA,
}

impl ModuleA for ModuleSoleA {
    fn init_a(&mut self, _initializer: &mut InitializerA) {
        println!("Initializing ModuleSoleA");
    }
    fn connector_a(&mut self) -> &mut ConnectorA {
        println!("Accessing ModuleSoleA->ConnectorA");
        &mut self.connector_a
    }
}

impl Module for ModuleSoleA {
    fn attach(self, keeper: &mut ModuleKeeper) {
        println!("attaching ModuleSoleA");
        keeper.add_a_module(self);
    }
    fn run(&mut self) {
        println!("Running ModuleSoleA");
    }
}

pub struct ModuleSoleB {
    connector_b: ConnectorB,
}

impl ModuleB for ModuleSoleB {
    fn init_b(&mut self, _initializer: &mut InitializerB) {
        println!("Initializing ModuleSoleB");
    }
    fn connector_b(&mut self) -> &mut ConnectorB {
        println!("Accessing ModuleSoleB->ConnectorB");
        &mut self.connector_b
    }
}

impl Module for ModuleSoleB {
    fn attach(self, keeper: &mut ModuleKeeper) {
        keeper.add_b_module(self);
    }
    fn run(&mut self) {
        println!("Running ModuleSoleB");
    }
}

pub struct ModuleAB {
    connector_a: ConnectorA,
    connector_b: ConnectorB,
}

impl ModuleA for ModuleAB {
    fn init_a(&mut self, _initializer: &mut InitializerA) {
        println!("Initializing ModuleA of ModuleAB");
    }
    fn connector_a(&mut self) -> &mut ConnectorA {
        println!("Accessing ModuleAB->ConnectorA");
        &mut self.connector_a
    }
}

impl ModuleB for ModuleAB {
    fn init_b(&mut self, _initializer: &mut InitializerB) {
        println!("Initializing ModuleB of ModuleAB");
    }
    fn connector_b(&mut self) -> &mut ConnectorB {
        println!("Accessing ModuleAB->ConnectorB");
        &mut self.connector_b
    }
}

impl Module for ModuleAB {
    fn attach(self, keeper: &mut ModuleKeeper) {
        keeper.add_ab_module(self);
    }
    fn run(&mut self) {
        println!("Running ModuleAB");
    }
}

pub struct ModuleKeeper {
    modules: Vec<Box<dyn ModuleWrapper>>,
}

impl ModuleKeeper {
    pub fn add_ab_module<T: ModuleA + ModuleB + 'static>(&mut self, module: T) {
        self.modules.push(Box::new(ModuleABWrapper(module)));
    }
    pub fn add_a_module(&mut self, module: impl ModuleA + 'static) {
        self.modules.push(Box::new(ModuleAWrapper(module)));
    }
    pub fn add_b_module(&mut self, module: impl ModuleB + 'static) {
        self.modules.push(Box::new(ModuleBWrapper(module)));
    }
}

struct MainObject {
    component_a: ComponentA,
    component_b: ComponentB,
    keeper: ModuleKeeper,
}

impl MainObject {
    fn new() -> MainObject {
        MainObject {
            component_a: ComponentA {},
            component_b: ComponentB {},
            keeper: ModuleKeeper {
                modules: Vec::new(),
            },
        }
    }

    fn add_module(&mut self, module: impl Module) {
        module.attach(&mut self.keeper);
    }
    fn execute(&mut self) {
        println!("Executing main...");
        for module in self.keeper.modules.iter_mut() {
            if module.has_a() {
                self.component_a.prepare_a(module.as_mut());
                module.run();
                self.component_a.process_a(module.as_mut());
            }
        }

        for module in self.keeper.modules.iter_mut() {
            if module.has_b() {
                self.component_b.prepare_b(module.as_mut());
                module.run();
                self.component_b.process_b(module.as_mut());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectorA;
    use super::ConnectorB;
    use super::MainObject;
    use super::ModuleAB;
    use super::ModuleSoleA;
    use super::ModuleSoleB;

    #[test]
    fn basic_test() {
        let a = ModuleSoleA {
            connector_a: ConnectorA {},
        };
        let b = ModuleSoleB {
            connector_b: ConnectorB {},
        };
        let ab = ModuleAB {
            connector_a: ConnectorA {},
            connector_b: ConnectorB {},
        };

        let mut main = MainObject::new();
        main.add_module(a);
        main.add_module(b);
        main.add_module(ab);

        main.execute();
    }
}
