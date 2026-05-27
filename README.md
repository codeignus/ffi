# ffi

Rust-owned scratch buffer for Go `//export` handlers.

`types.h` is the single ABI source of truth and is committed.
Rust `#[repr(C)]` and Go boundary code mirror that header.

| Part | Depend via |
|------|------------|
| `rust/` | `ffi = { git = "https://github.com/nevaria-ai/ffi" }` |
| `go/` | `require github.com/nevaria-ai/ffi/go v0.x` |

This library does not call your exports — you wire `extern` (Rust) and `//export` (Go).

## Layout

One pointer per call: `ffi_call_ctx` header + trailing scratch bytes (same allocation).

- `input_off` / `input_len` — request payload in scratch  
- `output_off` / `output_len` — response written by Go  
- `transport_err[256]` + `transport_err_len` — optional transport message  

## Go export

```go
//export my_op_into
func my_op_into(handle C.uintptr_t, callCtx *C.ffi_call_ctx) uint32 {
    ctx := unsafe.Pointer(callCtx)
    read, transportStatus := boundary.ReadCall(ctx)
    if transportStatus != boundary.FfiStatusOk { return transportStatus.Uint32() }

    // decode read.Payload, run domain logic → out []byte

    return boundary.WriteCall(ctx, read.WriteOff, out).Uint32()
}
```

Set `CGO_CFLAGS=-I$(go list -m -f '{{.Dir}}' github.com/nevaria-ai/ffi/go)/..`.

## Rust caller

```rust
let mut call_ctx = ffi::CallCtx::new();
call_ctx.prepare_input(payload);
let code = unsafe { export(h, call_ctx.as_mut_ptr()) };
ffi::check_code(code as u32, &call_ctx)?;
let body = call_ctx.read_output_string()?;
```

Keep Rust/Go mirrors in sync with `types.h`.
