use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;

pub(crate) mod connector;
pub(crate) mod initializer;

pub use connector::NervousConnector;
pub use initializer::NervousInitializer;

pub trait NervousComponent<O: Organism>: SimComponent<O> {
    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn nervous_init(&mut self, nervous_initializer: &mut NervousInitializer<O>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// CirculationConnector, which tracks module interactions
    ///
    /// ### returns
    /// TimeManager to interact with the rest of the simulation
    fn nervous_connector(&mut self) -> &mut NervousConnector<O>;
}

#[cfg(test)]
pub mod test {
    use crate::{event::Event, sim::{component::SimComponent, organism::test::{TestAnatomicalRegion, TestNerve, TestOrganism}, Organism, SimTime}};

    use super::{NervousComponent, NervousConnector};

    #[derive(Debug)]
    struct PainEvent {
        level: u8,
        region: TestAnatomicalRegion,
    }

    impl Event for PainEvent {}

    #[derive(Debug)]
    struct MovementEvent {
        amount: u8,
    }

    impl Event for MovementEvent {}

    struct TestPainReflexComponent {
        nervous_connector: NervousConnector<TestOrganism>,
        head_path: Vec<TestNerve>,
        torso_path: Vec<TestNerve>,
        right_arm_path: Vec<TestNerve>,
        left_arm_path: Vec<TestNerve>,
        right_leg_path: Vec<TestNerve>,
        left_leg_path: Vec<TestNerve>,
    }

    impl TestPainReflexComponent {
        pub fn new() -> Self {
            TestPainReflexComponent {
                nervous_connector: NervousConnector::new(),
                head_path: vec![
                    TestNerve::Brain,
                ],
                torso_path: vec![
                    TestNerve::Brain,
                    TestNerve::SpinalCord,
                ],
                right_arm_path: vec![
                    TestNerve::Brain,
                    TestNerve::SpinalCord,
                    TestNerve::RightC,
                    TestNerve::RightAxillary,
                ],
                left_arm_path: vec![
                    TestNerve::Brain,
                    TestNerve::SpinalCord,
                    TestNerve::LeftC,
                    TestNerve::LeftAxillary,
                ],
                right_leg_path: vec![
                    TestNerve::Brain,
                    TestNerve::SpinalCord,
                    TestNerve::RightL,
                    TestNerve::RightFemoral,
                ],
                left_leg_path: vec![
                    TestNerve::Brain,
                    TestNerve::SpinalCord,
                    TestNerve::LeftL,
                    TestNerve::LeftFemoral,
                ],
            }
        }

        fn get_target_path(&self, region: TestAnatomicalRegion) -> Vec<TestNerve> {
            match region {
                TestAnatomicalRegion::Head => self.head_path.clone(),
                TestAnatomicalRegion::Torso => self.torso_path.clone(),
                TestAnatomicalRegion::RightArm => self.right_arm_path.clone(),
                TestAnatomicalRegion::LeftArm => self.left_arm_path.clone(),
                TestAnatomicalRegion::RightLeg => self.right_arm_path.clone(),
                TestAnatomicalRegion::LeftLeg => self.left_arm_path.clone(),
            }
        }
    }

    impl NervousComponent<TestOrganism> for TestPainReflexComponent {
        fn nervous_init(&mut self, nervous_initializer: &mut super::NervousInitializer<TestOrganism>) {
            nervous_initializer.notify_of::<PainEvent>(TestNerve::Brain);
        }

        fn nervous_connector(&mut self) -> &mut NervousConnector<TestOrganism> {
            &mut self.nervous_connector
        }
    }

    impl SimComponent<TestOrganism> for TestPainReflexComponent {
        fn id(&self) -> &'static str {
            "TestPainReflexComponent"
        }

        fn attach(self, registry: &mut crate::sim::component::ComponentRegistry<TestOrganism>) {
            registry.add_nervous_component(self)
        }

        fn run(&mut self) {
            let mut signals_to_send = Vec::new();
            for (_, pain_signal) in self.nervous_connector.get_messages::<PainEvent>() {
                let mut reflex_amount = 0;
                if pain_signal.level < 5 {
                    println!("It's just a scratch. Ignore.");
                }
                else if pain_signal.level >= 5 && pain_signal.level < 8 {
                    println!("Ow");
                    reflex_amount = 100;
                }
                else {
                    println!("AAAAAAAGGHGHHHAGHAHGHGHGHGH!!!");
                    reflex_amount = 200;
                }

                if reflex_amount > 0 {
                    // adding to a temporary structure here because
                    // we're currently immutably borrowing the nervous_connector
                    // so we can't also mutably borrow at the same time to send
                    // a message
                    signals_to_send.push((
                        MovementEvent { amount: reflex_amount },
                        self.get_target_path(pain_signal.region),
                        self.nervous_connector.sim_time() + SimTime::from_ms(100.0),
                    ));
                }
            }

            for (evt, path, time) in signals_to_send {
                self.nervous_connector.send_message(evt, path, time).unwrap();
            }
        }
    }


    struct TestMovementComponent {
        nervous_connector: NervousConnector<TestOrganism>,
    }

    impl TestMovementComponent {
        pub fn new() -> Self {
            TestMovementComponent {
                nervous_connector: NervousConnector::new(),
            }
        }
        
        fn nerve_to_appendage(nerve: TestNerve) -> &'static str {
            match nerve {
                TestNerve::RightAxillary => "right arm",
                TestNerve::LeftAxillary => "left arm",
                TestNerve::RightFemoral => "right leg",
                TestNerve::LeftFemoral => "left leg",
                _ => "!@#$"
            }
        }

        fn print_movement_amt(amt: u8, appendage: &'static str) {
            // What? Dirty? What are you talking abo- o_o
            if amt > 128 {
                println!("I'm flailing my {} around!", appendage);
            }
            else if amt > 0 {
                println!("My {} moves casually.", appendage);
            }

            println!("My {} is not moving.", appendage);
        }
    }

    impl NervousComponent<TestOrganism> for TestMovementComponent {
        fn nervous_init(&mut self, nervous_initializer: &mut super::NervousInitializer<TestOrganism>) {
            nervous_initializer.notify_of::<MovementEvent>(TestNerve::RightAxillary);
            nervous_initializer.notify_of::<MovementEvent>(TestNerve::LeftAxillary);
            nervous_initializer.notify_of::<MovementEvent>(TestNerve::RightFemoral);
            nervous_initializer.notify_of::<MovementEvent>(TestNerve::LeftFemoral);
        }

        fn nervous_connector(&mut self) -> &mut NervousConnector<TestOrganism> {
            &mut self.nervous_connector
        }
    }

    impl SimComponent<TestOrganism> for TestMovementComponent {
        fn id(&self) -> &'static str {
            "TestMovementComponent"
        }

        fn attach(self, registry: &mut crate::sim::component::ComponentRegistry<TestOrganism>) {
            registry.add_nervous_component(self)
        }

        fn run(&mut self) {
            for (target_nerve, movement_event) in self.nervous_connector.get_messages::<MovementEvent>() {
                Self::print_movement_amt(movement_event.amount, Self::nerve_to_appendage(target_nerve))
            }
        }
    }

    struct TestPainkillerComponent {
        nervous_connector: NervousConnector<TestOrganism>,
    }

    impl TestPainkillerComponent {
        pub fn new() -> Self {
            TestPainkillerComponent {
                nervous_connector: NervousConnector::new(),
            }
        }
        
        fn nerve_to_appendage(nerve: TestNerve) -> &'static str {
            match nerve {
                TestNerve::RightAxillary => "right arm",
                TestNerve::LeftAxillary => "left arm",
                TestNerve::RightFemoral => "right leg",
                TestNerve::LeftFemoral => "left leg",
                _ => "!@#$"
            }
        }

        fn print_movement_amt(amt: u8, appendage: &'static str) {
            // What? Dirty? What are you talking abo- o_o
            if amt > 128 {
                println!("I'm flailing my {} around!", appendage);
            }
            else if amt > 0 {
                println!("My {} moves casually.", appendage);
            }

            println!("My {} is not moving.", appendage);
        }
    }

    impl NervousComponent<TestOrganism> for TestPainkillerComponent {
        fn nervous_init(&mut self, nervous_initializer: &mut super::NervousInitializer<TestOrganism>) {
            // Let's kill pain on the SpinalCord before it even reaches the Brain
            nervous_initializer.transform_message::<PainEvent>(TestNerve::SpinalCord, |msg| {
                // Subtract 111 to a minimum of 1
                if None == msg.level.checked_sub(111) {
                    msg.level = 1;
                }
                Some(())
            });
        }

        fn nervous_connector(&mut self) -> &mut NervousConnector<TestOrganism> {
            &mut self.nervous_connector
        }
    }

    impl SimComponent<TestOrganism> for TestPainkillerComponent {
        fn id(&self) -> &'static str {
            "TestPainkillerComponent"
        }

        fn attach(self, registry: &mut crate::sim::component::ComponentRegistry<TestOrganism>) {
            registry.add_nervous_component(self)
        }

        fn run(&mut self) {
            // Nothing to do here for this one.
        }
    }

}
