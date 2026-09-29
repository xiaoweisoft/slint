# skia-safe 0.90.0 ownership backport

Imported from the crates.io skia-safe 0.90.0 archive (MIT), SHA256
`6a71c01d325d40b1031dee67d251a5e0132e79e2a9ec272149a4f4a0d4b8b3be`.
The native skia-bindings dependency remains exactly 0.90.0. No native ABI or
Skia version is changed. This is an in-tree dependency fork, not a second Skia
implementation; remove it when upstream exposes equivalent ownership APIs.

Local additions: Data::from_owned_bytes retains an immutable byte owner until
SkData's final release callback; FontMgr::new_from_owned_data mirrors upstream's
owned Data constructor without breaking the 0.90 slice-based constructor.
The unsafe owner API requires a stable immutable byte allocation. Renderer
callers retain reference-counted font blobs and copy-on-write pixel buffers.
Tests cover reference clones, cross-thread release, and exactly-once cleanup.

`native/` contains only the transitive headers required by SkData.h from the
skia-bindings 0.90.0 crate (BSD license retained in native/LICENSE), archive
SHA256 `8f6f96e00735f14a781aac8a6870c862b8cc831df6d8e4ad77ab78e11411b9af`.
The tiny C wrapper returns a raw pointer instead of a C++ sk_sp value: calling
the generated direct C++ return-by-value binding is not ABI-safe. Native Skia
itself is still built exclusively by the original skia-bindings package.
