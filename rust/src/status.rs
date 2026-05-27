//! Layer-1 transport status from Go exports.

use crate::scratch::CallCtx;

/// Wire status (`uint32`). Domain errors belong in scratch output, not here.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiStatusCode {
    Ok = 0,
    InvalidArg = 1001,
    InvalidUtf8 = 1002,
    BufferTooSmall = 2001,
    InvalidHandle = 3001,
    Unknown = 9999,
}

impl FfiStatusCode {
    pub fn from_raw(v: u32) -> Self {
        match v {
            0 => Self::Ok,
            1001 => Self::InvalidArg,
            1002 => Self::InvalidUtf8,
            2001 => Self::BufferTooSmall,
            3001 => Self::InvalidHandle,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportError {
    pub code: FfiStatusCode,
    pub message: Option<String>,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{:?}: {m}", self.code),
            None => write!(f, "{:?}", self.code),
        }
    }
}

impl std::error::Error for TransportError {}

/// After `unsafe { your_export(...) }` returns.
pub fn check_code(code: u32, ctx: &CallCtx) -> Result<(), TransportError> {
    let code = FfiStatusCode::from_raw(code);
    if code == FfiStatusCode::Ok {
        return Ok(());
    }
    Err(TransportError {
        code,
        message: ctx.transport_error_message(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_code_mapping_guardrail() {
        assert_eq!(FfiStatusCode::from_raw(0), FfiStatusCode::Ok);
        assert_eq!(FfiStatusCode::from_raw(1001), FfiStatusCode::InvalidArg);
        assert_eq!(FfiStatusCode::from_raw(2001), FfiStatusCode::BufferTooSmall);
        assert_eq!(FfiStatusCode::from_raw(3001), FfiStatusCode::InvalidHandle);
        assert_eq!(FfiStatusCode::from_raw(42), FfiStatusCode::Unknown);
    }
}
