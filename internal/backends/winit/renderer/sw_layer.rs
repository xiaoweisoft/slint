// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! Software renderer variant that paints into a SCTK-allocated `wl_surface`
//! instead of an `Arc<winit::window::Window>`. Used by layer-shell windows.
//!
//! Forked from `sw.rs`. The pixel format conversion + SoftwareRenderer plumbing
//! is identical; only `resume`/`suspend` and the surface-handle types differ.

use core::num::NonZeroU32;
use core::ops::DerefMut;
use core::ptr::NonNull;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use i_slint_core::graphics::Rgb8Pixel;
use i_slint_core::platform::PlatformError;
use i_slint_renderer_software::{
    PremultipliedRgbaColor, RepaintBufferType, SoftwareRenderer, TargetPixel,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use wayland_client::Proxy;

use crate::layer_shell::LayerSurfaceHandle;

/// Wrapper that exposes the SCTK display+surface to softbuffer via the
/// raw_window_handle traits.
#[derive(Clone)]
struct LayerWindowTarget {
    display_ptr: NonNull<core::ffi::c_void>,
    surface_ptr: NonNull<core::ffi::c_void>,
}

// `wl_display` and `wl_surface` are reference-counted by libwayland and may be
// shared across threads. We never call libwayland from another thread, but
// softbuffer requires `Send + Sync` on its handle source.
unsafe impl Send for LayerWindowTarget {}
unsafe impl Sync for LayerWindowTarget {}

impl HasDisplayHandle for LayerWindowTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(self.display_ptr));
        // SAFETY: pointers stay valid while LayerSurfaceHandle (and therefore
        // the LayerShellManager that owns the connection) is alive. softbuffer
        // borrows them only during this call's lifetime, which is shorter than
        // `&self`.
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

impl HasWindowHandle for LayerWindowTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(self.surface_ptr));
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SoftBufferPixel(pub u32);

impl From<SoftBufferPixel> for PremultipliedRgbaColor {
    #[inline]
    fn from(pixel: SoftBufferPixel) -> Self {
        let v = pixel.0;
        PremultipliedRgbaColor {
            red: (v >> 16) as u8,
            green: (v >> 8) as u8,
            blue: v as u8,
            alpha: (v >> 24) as u8,
        }
    }
}

impl From<PremultipliedRgbaColor> for SoftBufferPixel {
    #[inline]
    fn from(pixel: PremultipliedRgbaColor) -> Self {
        Self(
            ((pixel.alpha as u32) << 24)
                | ((pixel.red as u32) << 16)
                | ((pixel.green as u32) << 8)
                | (pixel.blue as u32),
        )
    }
}

impl TargetPixel for SoftBufferPixel {
    fn blend(&mut self, color: PremultipliedRgbaColor) {
        let mut x = PremultipliedRgbaColor::from(*self);
        x.blend(color);
        *self = x.into();
    }
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(0xff000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }
    fn background() -> Self {
        Self(0)
    }
}

pub struct LayerSoftwareRenderer {
    renderer: SoftwareRenderer,
    /// Held to keep the SCTK surface alive for the renderer's lifetime.
    layer_handle: RefCell<Option<Rc<LayerSurfaceHandle>>>,
    _context: RefCell<Option<softbuffer::Context<Arc<LayerWindowTarget>>>>,
    surface: RefCell<Option<softbuffer::Surface<Arc<LayerWindowTarget>, Arc<LayerWindowTarget>>>>,
}

impl LayerSoftwareRenderer {
    pub fn new() -> Self {
        Self {
            renderer: SoftwareRenderer::new(),
            layer_handle: RefCell::new(None),
            _context: RefCell::new(None),
            surface: RefCell::new(None),
        }
    }

    #[allow(dead_code)] // exposed for future skia/femtovg path symmetry
    pub fn as_core_renderer(&self) -> &dyn i_slint_core::renderer::Renderer {
        &self.renderer
    }

    /// Bind the renderer to a SCTK-allocated layer surface. Must be called
    /// after `LayerShellManager::create_surface` and before the first `render`.
    pub fn resume_layer(&self, handle: Rc<LayerSurfaceHandle>) -> Result<(), PlatformError> {
        let conn = handle.connection_for_renderer();
        // Pull raw pointers out of libwayland's sys backend. Requires the
        // `client_system` feature on wayland-backend (enabled transitively
        // through softbuffer's wayland-dlopen).
        let display_raw = conn.backend().display_ptr() as *mut core::ffi::c_void;
        let surface_raw = handle.wl_surface.id().as_ptr() as *mut core::ffi::c_void;

        let display_ptr = NonNull::new(display_raw)
            .ok_or_else(|| PlatformError::from("layer-shell: null wl_display"))?;
        let surface_ptr = NonNull::new(surface_raw)
            .ok_or_else(|| PlatformError::from("layer-shell: null wl_surface ptr"))?;

        let target = Arc::new(LayerWindowTarget { display_ptr, surface_ptr });
        let context = softbuffer::Context::new(target.clone())
            .map_err(|e| format!("layer-shell: softbuffer context: {e}"))?;
        let surface = softbuffer::Surface::new(&context, target.clone())
            .map_err(|e| format!("layer-shell: softbuffer surface: {e}"))?;

        *self._context.borrow_mut() = Some(context);
        *self.surface.borrow_mut() = Some(surface);
        *self.layer_handle.borrow_mut() = Some(handle);
        Ok(())
    }

    #[allow(dead_code)] // wired in once event_loop suspend path is extended
    pub fn suspend(&self) {
        drop(self.surface.borrow_mut().take());
        drop(self._context.borrow_mut().take());
        drop(self.layer_handle.borrow_mut().take());
    }

    #[allow(dead_code)] // wired in once event_loop occlusion path is extended
    pub fn occluded(&self, _: bool) {
        self.renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
    }

    pub fn render(&self, window: &i_slint_core::api::Window) -> Result<(), PlatformError> {
        let size = window.size();
        let Some((width, height)) = size.width.try_into().ok().zip(size.height.try_into().ok())
        else {
            return Ok(());
        };

        let mut borrowed_surface = self.surface.borrow_mut();
        let Some(surface) = borrowed_surface.as_mut() else {
            return Ok(());
        };

        surface
            .resize(width, height)
            .map_err(|e| format!("layer-shell: softbuffer resize: {e}"))?;

        let mut target_buffer =
            surface.buffer_mut().map_err(|e| format!("layer-shell: softbuffer buffer_mut: {e}"))?;

        // Force full repaint each frame: softbuffer's `age()`-based dirty
        // tracking is not yet validated for layer-shell surfaces, and the
        // OSK-sized buffers we paint are small enough that the cost is modest.
        self.renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);

        let region = if std::env::var_os("SLINT_LINE_BY_LINE").is_none() {
            let buffer: &mut [SoftBufferPixel] =
                bytemuck::cast_slice_mut(target_buffer.deref_mut());
            self.renderer.render(buffer, width.get() as usize)
        } else {
            struct FrameBuffer<'a> {
                buffer: &'a mut [u32],
                line: Vec<i_slint_renderer_software::Rgb565Pixel>,
            }
            impl i_slint_renderer_software::LineBufferProvider for FrameBuffer<'_> {
                type TargetPixel = i_slint_renderer_software::Rgb565Pixel;
                fn process_line(
                    &mut self,
                    line: usize,
                    range: core::ops::Range<usize>,
                    render_fn: impl FnOnce(&mut [Self::TargetPixel]),
                ) {
                    let line_begin = line * self.line.len();
                    let sub = &mut self.line[..range.len()];
                    render_fn(sub);
                    for (dst, src) in self.buffer[line_begin..][range].iter_mut().zip(sub) {
                        let p = Rgb8Pixel::from(*src);
                        *dst =
                            0xff000000 | ((p.r as u32) << 16) | ((p.g as u32) << 8) | (p.b as u32);
                    }
                }
            }
            self.renderer.render_by_line(FrameBuffer {
                buffer: &mut target_buffer,
                line: vec![Default::default(); width.get() as usize],
            })
        };

        let bbox = region.bounding_box_size();
        if let Some((w, h)) = Option::zip(NonZeroU32::new(bbox.width), NonZeroU32::new(bbox.height))
        {
            let pos = region.bounding_box_origin();
            target_buffer
                .present_with_damage(&[softbuffer::Rect {
                    width: w,
                    height: h,
                    x: pos.x as u32,
                    y: pos.y as u32,
                }])
                .map_err(|e| format!("layer-shell: softbuffer present: {e}"))?;
        } else if let (Some(w), Some(h)) =
            (NonZeroU32::new(width.get()), NonZeroU32::new(height.get()))
        {
            // Always attach a buffer on the first frame, otherwise the layer
            // surface stays unmapped. Damage the full size.
            target_buffer
                .present_with_damage(&[softbuffer::Rect { width: w, height: h, x: 0, y: 0 }])
                .map_err(|e| format!("layer-shell: softbuffer present (forced): {e}"))?;
        }
        Ok(())
    }
}
