use std::ffi::*;
use std::os::raw::*;
use std::collections::HashMap;
use std::mem::transmute;

use libc::c_char;

use crate::prelude::*;

#[repr(C)]
pub struct CEndpoint {
    real_endpoint: *mut Endpoint
}


#[no_mangle]
pub extern "C" fn endpoint_new(addr: *mut c_char, buffer_size: c_int, read_timeout: c_int)  -> *mut CEndpoint {
    let c_string = unsafe { CString::from_raw(addr) };
    let c_endp = CEndpoint {
        real_endpoint: unsafe {
            let address_res = CString::from_raw(addr).into_string();
            if address_res.is_err() {
                return transmute(&0);
            }
            let address = address_res.unwrap();
            let endpoint_config = EndpointConfig::new(&address);
            let endpoint_arc = Endpoint::new(endpoint_config); 
            transmute(
                Box::new(
                    endpoint_arc
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