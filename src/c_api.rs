use std::ffi::*;
use std::os::raw::*;
use std::collections::HashMap;
use std::mem::transmute;

use libc::c_char;

use crate::endpoint::Endpoint;

#[repr(C)]
pub struct CEndpoint {
    real_endpoint: *mut Endpoint
}


#[no_mangle]
pub extern "C" fn endpoint_new(addr: *mut c_char, buffer_size: c_int, read_timeout: c_int)  -> *mut CEndpoint {
    let c_string = unsafe { CString::from_raw(addr) };
    let c_endp = CEndpoint {
        real_endpoint: unsafe {
            transmute(
                Box::new(
                    Endpoint::new(
                        c_string.to_string_lossy().to_string(),
                        buffer_size,
                        read_timeout
                    ).unwrap()
                )
            )
        }
    };
    unsafe {
        transmute(
            Box::new(
                c_endp
            )
        )
    }
}

#[no_mangle]
pub extern "C" fn endpoint_delete(endpoint: *mut Endpoint) {
    let endpoint_box: Box<Endpoint> = unsafe {
        transmute(endpoint)
    };
    //Object is destroyed here
}