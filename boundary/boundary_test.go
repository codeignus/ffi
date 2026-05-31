package boundary

import (
	"testing"
	"unsafe"
)

// Layout mirror of ffi_call_ctx (see types.h).
type ffiCallCtx struct {
	scratchCap      uint32
	inputOff        uint32
	inputLen        uint32
	outputOff       uint32
	outputLen       uint32
	transportErrLen uint32
	transportErr    [256]byte
}

func allocCallCtx(scratchCap int) (ffiCallCtx, []byte) {
	var header ffiCallCtx
	header.scratchCap = uint32(scratchCap)
	scratch := make([]byte, scratchCap)
	return header, scratch
}

func ctxPtr(header *ffiCallCtx, scratch []byte) unsafe.Pointer {
	storage := make([]byte, int(unsafe.Sizeof(*header))+len(scratch))
	copy(storage, unsafe.Slice((*byte)(unsafe.Pointer(header)), unsafe.Sizeof(*header)))
	copy(storage[unsafe.Sizeof(*header):], scratch)
	return unsafe.Pointer(&storage[0])
}

func scratchFromPtr(ptr unsafe.Pointer, cap int) []byte {
	base := uintptr(ptr) + unsafe.Sizeof(ffiCallCtx{})
	return unsafe.Slice((*byte)(unsafe.Pointer(base)), cap)
}

func TestReadCallValid(t *testing.T) {
	header, scratch := allocCallCtx(16)
	copy(scratch, []byte("hello-world"))
	header.inputOff = 0
	header.inputLen = 5
	ptr := ctxPtr(&header, scratch)

	read, transportStatus := ReadCall(ptr)
	if transportStatus != FfiStatusOk {
		t.Fatalf("transport status = %v, want ok", transportStatus)
	}
	if string(read.Payload) != "hello" {
		t.Fatalf("payload = %q, want hello", read.Payload)
	}
	if read.WriteOff != 5 {
		t.Fatalf("write offset = %d, want 5", read.WriteOff)
	}
}

func TestReadCallOutOfBounds(t *testing.T) {
	header, scratch := allocCallCtx(4)
	header.inputOff = 2
	header.inputLen = 5
	ptr := ctxPtr(&header, scratch)

	_, transportStatus := ReadCall(ptr)
	if transportStatus != FfiStatusInvalidArg {
		t.Fatalf("transport status = %v, want invalid arg", transportStatus)
	}
}

func TestWriteCallAtOffset(t *testing.T) {
	header, scratch := allocCallCtx(16)
	copy(scratch, []byte("abc"))
	ptr := ctxPtr(&header, scratch)

	transportStatus := WriteCall(ptr, 3, []byte("xy"))
	if transportStatus != FfiStatusOk {
		t.Fatalf("transport status = %v, want ok", transportStatus)
	}
	h := (*ffiCallCtx)(ptr)
	if h.outputOff != 3 || h.outputLen != 2 {
		t.Fatalf("output slot = (%d,%d), want (3,2)", h.outputOff, h.outputLen)
	}
	buf := scratchFromPtr(ptr, 16)
	if string(buf[3:5]) != "xy" {
		t.Fatalf("scratch write = %q, want xy", buf[3:5])
	}
}

func TestWriteCallBufferTooSmall(t *testing.T) {
	header, scratch := allocCallCtx(4)
	ptr := ctxPtr(&header, scratch)

	transportStatus := WriteCall(ptr, 3, []byte("toolong"))
	if transportStatus != FfiStatusBufferTooSmall {
		t.Fatalf("transport status = %v, want buffer too small", transportStatus)
	}
}
