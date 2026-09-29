// MIT: see LICENSE. Native ownership bridge for the pinned Skia ABI.
#include "include/core/SkData.h"

extern "C" SkData* C_SkData_MakeOwnedBytes(const void* bytes, size_t length,
                                         SkData::ReleaseProc release, void* owner) {
    return SkData::MakeWithProc(bytes, length, release, owner).release();
}
