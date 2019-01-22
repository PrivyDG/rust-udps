#![feature(try_from, integer_atomics)]
#[warn(unused_imports)]

#[macro_use]extern crate serde;
#[macro_use]extern crate serde_derive;
extern crate rand;
extern crate libc;
extern crate rmp_serde as rmps;
extern crate openssl;
extern crate twofish;

/**
 * Prelude module to reexport everything
*/
#[macro_use]
pub mod prelude {
    pub use crate::connection::*;
    pub use crate::endpoint::*;
    pub use crate::package::*;
    pub use crate::util::*;

    pub use crate::VERSION;
    pub use crate::VERSION_MAJOR;
    pub use crate::VERSION_MINOR;
    pub use crate::VERSION_PATCH;
}

/**
 * Utility module
*/
#[macro_use]
pub mod util;

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
/**
 * Version constant
*/
pub static VERSION: [u8; 3] = [
    VERSION_MAJOR,
    VERSION_MINOR,
    VERSION_PATCH
];
