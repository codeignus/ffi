/* Shared Rust/Go FFI layouts. Include as: #include "types.h" */
#ifndef FFI_TYPES_H
#define FFI_TYPES_H

#include <stdint.h>

#define FFI_TRANSPORT_ERR_CAP 256

/**
 * Per-call state for Rust → Go //export handlers.
 * Scratch bytes follow this struct: (uint8_t *)ctx + sizeof(ffi_call_ctx), length scratch_cap.
 */
typedef struct {
    uint32_t scratch_cap;
    uint32_t input_off;
    uint32_t input_len;
    uint32_t output_off;
    uint32_t output_len;
    uint32_t transport_err_len;
    uint8_t transport_err[FFI_TRANSPORT_ERR_CAP];
} ffi_call_ctx;

#endif
