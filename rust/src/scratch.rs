//! `ffi_call_ctx` allocation and scratch I/O (layouts from `types.h`).

use std::fmt;

pub use crate::abi::ffi_call_ctx;
pub type CallCtxHeader = ffi_call_ctx;

const DEFAULT_SCRATCH_CAP: usize = 8 * 1024;
const MAX_SCRATCH_CAP: usize = 8 * 1024 * 1024;
const TRANSPORT_ERR_CAP: usize = 256;
const SCRATCH_OFFSET: usize = std::mem::size_of::<ffi_call_ctx>();

/// Rust-owned `ffi_call_ctx` + trailing scratch bytes for one export call.
pub struct CallCtx {
    storage: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

impl Default for CallCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl CallCtx {
    pub fn new() -> Self {
        Self::with_scratch_cap(DEFAULT_SCRATCH_CAP)
    }

    pub fn with_scratch_cap(scratch_cap: usize) -> Self {
        let scratch_cap = scratch_cap.clamp(1, MAX_SCRATCH_CAP);
        let mut storage = vec![0u8; SCRATCH_OFFSET + scratch_cap];
        let header = Self::header_mut(&mut storage);
        header.scratch_cap = scratch_cap as u32;
        Self { storage }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi_call_ctx {
        self.storage.as_mut_ptr() as *mut ffi_call_ctx
    }

    pub fn scratch_cap(&self) -> usize {
        self.header().scratch_cap as usize
    }

    pub fn grow_scratch(&mut self, new_cap: usize) {
        let new_cap = new_cap.clamp(1, MAX_SCRATCH_CAP);
        if new_cap <= self.scratch_cap() {
            return;
        }
        let header = *self.header();
        let old = self.scratch().to_vec();
        self.storage.resize(SCRATCH_OFFSET + new_cap, 0);
        let header_mut = Self::header_mut(&mut self.storage);
        *header_mut = header;
        header_mut.scratch_cap = new_cap as u32;
        let copy_len = old.len().min(new_cap);
        self.scratch_mut()[..copy_len].copy_from_slice(&old[..copy_len]);
    }

    /// Write payload into scratch and set input_off/input_len.
    pub fn prepare_input(&mut self, payload: &str) {
        let bytes = payload.as_bytes();
        let header = Self::header_mut(&mut self.storage);
        header.input_off = 0;
        header.input_len = bytes.len() as u32;
        header.output_off = 0;
        header.output_len = 0;
        header.transport_err_len = 0;
        if bytes.len() > self.scratch_cap() {
            self.grow_scratch(bytes.len());
        }
        self.scratch_mut()[..bytes.len()].copy_from_slice(bytes);
    }

    pub fn read_output_string(&self) -> Result<String, ParseError> {
        let header = self.header();
        let off = header.output_off as usize;
        let len = header.output_len as usize;
        if len == 0 {
            return Err(ParseError("empty output".into()));
        }
        let end = off
            .checked_add(len)
            .ok_or_else(|| ParseError("empty output".into()))?;
        let bytes = self
            .scratch()
            .get(off..end)
            .ok_or_else(|| ParseError("empty output".into()))?;
        String::from_utf8(bytes.to_vec()).map_err(|e| ParseError(format!("utf-8: {e}")))
    }

    pub fn transport_error_message(&self) -> Option<String> {
        let header = self.header();
        let len = header.transport_err_len as usize;
        if len == 0 {
            return None;
        }
        let n = len.min(TRANSPORT_ERR_CAP);
        String::from_utf8(header.transport_err[..n].to_vec()).ok()
    }

    fn header(&self) -> &ffi_call_ctx {
        unsafe { &*(self.storage.as_ptr() as *const ffi_call_ctx) }
    }

    fn header_mut(storage: &mut [u8]) -> &mut ffi_call_ctx {
        unsafe { &mut *(storage.as_mut_ptr() as *mut ffi_call_ctx) }
    }

    fn scratch(&self) -> &[u8] {
        let cap = self.scratch_cap();
        &self.storage[SCRATCH_OFFSET..SCRATCH_OFFSET + cap]
    }

    fn scratch_mut(&mut self) -> &mut [u8] {
        let cap = self.scratch_cap();
        &mut self.storage[SCRATCH_OFFSET..SCRATCH_OFFSET + cap]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_input_and_read_output() {
        let mut ctx = CallCtx::new();
        ctx.prepare_input("hello");
        let header = unsafe { &*ctx.as_mut_ptr() };
        assert_eq!(header.input_len, 5);

        let header = unsafe { &mut *ctx.as_mut_ptr() };
        header.output_off = 0;
        header.output_len = 5;
        assert_eq!(ctx.read_output_string().unwrap(), "hello");
    }

    #[test]
    fn invalid_output_slot_is_rejected() {
        let mut ctx = CallCtx::new();
        let header = unsafe { &mut *ctx.as_mut_ptr() };
        header.output_off = 999_999;
        header.output_len = 12;
        let err = ctx.read_output_string().unwrap_err();
        assert_eq!(err.0, "empty output");
    }
}
