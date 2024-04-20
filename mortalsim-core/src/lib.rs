#![allow(dead_code, unused_imports)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate strum_macros;

mod spark;
mod id_gen;
mod quantity;
mod util;

pub use id_gen::{IdGenerator, IdType};
pub use quantity::*;
pub(crate) use util::*;

pub mod event;
pub mod hub;
pub mod sim;
pub mod substance;
pub mod math;

pub mod units {
    pub use simple_si_units::*;
}
