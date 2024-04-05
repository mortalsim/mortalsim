#![allow(dead_code, unused_imports)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate strum_macros;

mod spark;
mod util;

pub mod event;
pub mod hub;
pub mod sim;
pub mod substance;
pub mod units;
pub use util::IdType;

#[cfg(test)]
mod test {
    use std::sync::Once;
    static INIT: Once = Once::new();

    pub fn init_test() {
        use simple_logger::SimpleLogger;

        INIT.call_once(|| {
            SimpleLogger::new().init().unwrap();
        });
    }
}
