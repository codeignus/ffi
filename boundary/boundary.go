// Package boundary helps Go //export handlers use scratch-based FFI layouts.
package boundary

/*
#cgo CFLAGS: -I${SRCDIR}/..
#include "types.h"
*/
import "C"

import (
	"fmt"
	"unsafe"
)

// ReadResult is the decoded input region from scratch plus where output may be written next.
type ReadResult struct {
	Payload  []byte
	WriteOff uint32
}

// ReadCall reads ctx.input_off/len from ctx scratch.
func ReadCall(ctx unsafe.Pointer) (ReadResult, FfiStatusCode) {
	if ctx == nil {
		return ReadResult{}, FfiStatusInvalidArg
	}
	c := (*C.ffi_call_ctx)(ctx)
	clearCallError(c)
	return readInput(c)
}

// WriteCall writes out at writeOff and sets ctx.output_off/len.
func WriteCall(ctx unsafe.Pointer, writeOff uint32, out []byte) FfiStatusCode {
	if ctx == nil {
		return FfiStatusInvalidArg
	}
	c := (*C.ffi_call_ctx)(ctx)
	clearCallError(c)
	return writeOutput(c, int(writeOff), out)
}

func scratchBytes(c *C.ffi_call_ctx) []byte {
	cap := int(c.scratch_cap)
	if cap == 0 {
		return nil
	}
	base := uintptr(unsafe.Pointer(c)) + uintptr(C.sizeof_ffi_call_ctx)
	return unsafe.Slice((*byte)(unsafe.Pointer(base)), cap)
}

func readInput(c *C.ffi_call_ctx) (ReadResult, FfiStatusCode) {
	buf := scratchBytes(c)
	off := int(c.input_off)
	n := int(c.input_len)
	if n == 0 {
		return ReadResult{WriteOff: uint32(off)}, FfiStatusOk
	}
	if off < 0 || n < 0 || off+n > len(buf) {
		writeCallError(c, fmt.Sprintf("input out of bounds: off=%d len=%d cap=%d", off, n, len(buf)))
		return ReadResult{}, FfiStatusInvalidArg
	}
	payload := make([]byte, n)
	copy(payload, buf[off:off+n])
	return ReadResult{Payload: payload, WriteOff: uint32(off + n)}, FfiStatusOk
}

func writeOutput(c *C.ffi_call_ctx, off int, data []byte) FfiStatusCode {
	buf := scratchBytes(c)
	if len(data) == 0 {
		c.output_off, c.output_len = 0, 0
		return FfiStatusOk
	}
	if off < 0 || off+len(data) > len(buf) {
		writeCallError(
			c,
			fmt.Sprintf("output too large: off=%d out_len=%d cap=%d", off, len(data), len(buf)),
		)
		return FfiStatusBufferTooSmall
	}
	copy(buf[off:], data)
	c.output_off = C.uint32_t(uint32(off))
	c.output_len = C.uint32_t(uint32(len(data)))
	return FfiStatusOk
}

func clearCallError(c *C.ffi_call_ctx) {
	if c == nil {
		return
	}
	c.transport_err_len = 0
}

// WriteCallError stores a short transport message in ctx.transport_err.
func WriteCallError(ctx unsafe.Pointer, msg string) {
	if ctx == nil {
		return
	}
	writeCallError((*C.ffi_call_ctx)(ctx), msg)
}

func writeCallError(c *C.ffi_call_ctx, msg string) {
	if c == nil {
		return
	}
	max := int(C.FFI_TRANSPORT_ERR_CAP)
	n := len(msg)
	if n > max {
		n = max
	}
	dst := (*[C.FFI_TRANSPORT_ERR_CAP]byte)(unsafe.Pointer(&c.transport_err[0]))
	copy(dst[:n], msg[:n])
	c.transport_err_len = C.uint32_t(n)
}
