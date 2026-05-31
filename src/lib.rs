//! Rust ↔ Go scratch FFI. Does not call your exports.
//!
//! Layouts: `github.com/codeignus/ffi/types.h`  
//! Go helpers: `github.com/codeignus/ffi/boundary`

mod abi {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    include!(concat!(env!("OUT_DIR"), "/ffi_types_bindings.rs"));
}

mod scratch;
mod status;

pub use scratch::{CallCtx, CallCtxHeader, ParseError};
pub use status::{check_code, FfiStatusCode, TransportError};
