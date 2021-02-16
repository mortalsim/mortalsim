#![ allow( dead_code, unused_imports ) ]

extern crate uom;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate lazy_static;

mod core;
mod util;

pub mod substance;
pub mod event;
pub mod blood;

#[cfg(test)]
use std::sync::Once;
#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
pub fn init_test() {
    use simple_logger::SimpleLogger;

    INIT.call_once(|| {
        SimpleLogger::new().init().unwrap();
    });
}