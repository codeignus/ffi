# ffi

Rust-owned scratch buffer for Go `//export` handlers.

`types.h` at the repo root is the single ABI source of truth.
Rust bindgen and Go `boundary` both include it from there.

| Part | Depend via |
|------|------------|
| Rust crate | `ffi = { git = "https://github.com/codeignus/ffi" }` |
| Go package | `require github.com/codeignus/ffi v0.x` |

This library does not call your exports — you wire `extern` (Rust) and `//export` (Go).

## Layout

```
ffi/
  types.h       # ABI source of truth
  Cargo.toml    # Rust library crate (repo root)
  src/
  go.mod        # Go module (repo root)
  boundary/     # Go cgo helpers
```

One pointer per call: `ffi_call_ctx` header + trailing scratch bytes (same allocation).

- `input_off` / `input_len` — request payload in scratch  
- `output_off` / `output_len` — response written by Go  
- `transport_err[256]` + `transport_err_len` — optional transport message  

## Go export

```go
import "github.com/codeignus/ffi/boundary"

//export my_op_into
func my_op_into(handle C.uintptr_t, callCtx unsafe.Pointer) uint32 {
    read, transportStatus := boundary.ReadCall(callCtx)
    if transportStatus != boundary.FfiStatusOk { return transportStatus.Uint32() }
    // decode read.Payload, run domain logic → out []byte
    return boundary.WriteCall(callCtx, read.WriteOff, out).Uint32()
}
```

## Rust caller

```rust
let mut call_ctx = ffi::CallCtx::new();
call_ctx.prepare_input(payload);
let code = unsafe { export(h, call_ctx.as_mut_ptr()) };
ffi::check_code(code as u32, &call_ctx)?;
let body = call_ctx.read_output_string()?;
```

Keep Rust/Go mirrors in sync with `types.h`.
