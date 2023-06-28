#![allow(dead_code, unused_imports, unused_macros)]

extern crate uom;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate strum_macros;

mod sim;
// mod hub;
mod spark;
mod util;

// pub mod substance;
pub mod event;
pub mod hub;
// pub mod closed_circulation;
// pub mod human;

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
