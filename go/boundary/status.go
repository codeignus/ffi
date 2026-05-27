package boundary

// FfiStatusCode is layer-1 transport status (matches rust ffi::FfiStatusCode).
type FfiStatusCode uint32

const (
	FfiStatusOk             FfiStatusCode = 0
	FfiStatusInvalidArg     FfiStatusCode = 1001
	FfiStatusInvalidUtf8    FfiStatusCode = 1002
	FfiStatusBufferTooSmall FfiStatusCode = 2001
	FfiStatusInvalidHandle  FfiStatusCode = 3001
	FfiStatusUnknown        FfiStatusCode = 9999
)

func (c FfiStatusCode) Uint32() uint32 { return uint32(c) }
