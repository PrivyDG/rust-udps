#![feature(try_from)]
#[warn(unused_imports)]

#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rand;
extern crate libc;
extern crate rmp_serde as rmps;

use std::vec::Vec;

/**
 * Endpoint logic to send and receive data with the UDPS protocol
*/
pub mod endpoint;

/**
 * Connection logic to store connection state for endpoints
*/
pub mod connection;

/**
 * Package logic to configure and identify UDPS packages
*/
pub mod package;

/**
 * C API function module
 */
pub mod c_api;

/**
 * Prelude module to reexport everything
*/
pub mod prelude {
    pub use crate::VERSION_MAJOR;
    pub use crate::VERSION_MINOR;
    pub use crate::VERSION_PATCH;
    pub use crate::endpoint::*;
    pub use crate::connection::*;
    pub use crate::package::*;
}

/**
 * Major version constant
*/
pub static VERSION_MAJOR: u8 = 0;
/**
 * Minor version constant
*/
pub static VERSION_MINOR: u8 = 4;
/**
 * Patch version constant
*/
pub static VERSION_PATCH: u8 = 1;
