// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! SCTK side-channel for `zwlr_layer_shell_v1` surfaces.
//!
//! The winit backend keeps its own xdg toplevel connection. This module opens
//! a *second* `wayland_client::Connection` from the same process and uses it
//! to allocate layer-shell surfaces. Validated by the PoC at
//! `/tmp/sctk-winit-poc/`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use smithay_client_toolkit::{
    delegate_registry,
    globals::GlobalData,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    Connection as WlConn, Dispatch, EventQueue, QueueHandle,
    globals::{GlobalList, registry_queue_init},
    protocol::{wl_compositor::WlCompositor, wl_registry, wl_surface::WlSurface},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer as WlrLayer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{
        Anchor as WlrAnchor, Event as LayerSurfaceEvent, KeyboardInteractivity as WlrKbi,
        ZwlrLayerSurfaceV1,
    },
};

// Public role enums — the slint user sets these without depending on
// wayland-protocols-wlr directly.

#[derive(Copy, Clone, Debug)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl From<Layer> for WlrLayer {
    fn from(l: Layer) -> WlrLayer {
        match l {
            Layer::Background => WlrLayer::Background,
            Layer::Bottom => WlrLayer::Bottom,
            Layer::Top => WlrLayer::Top,
            Layer::Overlay => WlrLayer::Overlay,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Anchor(u32);

impl Anchor {
    pub const NONE: Anchor = Anchor(0);
    pub const TOP: Anchor = Anchor(1);
    pub const BOTTOM: Anchor = Anchor(2);
    pub const LEFT: Anchor = Anchor(4);
    pub const RIGHT: Anchor = Anchor(8);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for Anchor {
    type Output = Anchor;
    fn bitor(self, rhs: Anchor) -> Anchor {
        Anchor(self.0 | rhs.0)
    }
}

impl From<Anchor> for WlrAnchor {
    fn from(a: Anchor) -> WlrAnchor {
        WlrAnchor::from_bits_truncate(a.bits())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum KeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}

impl From<KeyboardInteractivity> for WlrKbi {
    fn from(k: KeyboardInteractivity) -> WlrKbi {
        match k {
            KeyboardInteractivity::None => WlrKbi::None,
            KeyboardInteractivity::Exclusive => WlrKbi::Exclusive,
            KeyboardInteractivity::OnDemand => WlrKbi::OnDemand,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayerShellRole {
    pub layer: Layer,
    pub anchor: Anchor,
    pub exclusive_zone: i32,
    /// (width, height). 0 for either dimension means "stretch along anchored edges".
    pub size: (u32, u32),
    pub namespace: String,
    pub keyboard_interactivity: KeyboardInteractivity,
}

// Internal SCTK dispatch state — owns registry + per-surface state.

struct SctkState {
    registry_state: RegistryState,
    pending_configures: Arc<Mutex<Vec<PendingConfigure>>>,
}

struct PendingConfigure {
    serial: u32,
    width: u32,
    height: u32,
    surface_id: usize,
}

impl ProvidesRegistryState for SctkState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![];
}
delegate_registry!(SctkState);

impl Dispatch<wl_registry::WlRegistry, GlobalData> for SctkState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &WlConn,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCompositor, ()> for SctkState {
    fn event(
        _: &mut Self,
        _: &WlCompositor,
        _: <WlCompositor as wayland_client::Proxy>::Event,
        _: &(),
        _: &WlConn,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for SctkState {
    fn event(
        _: &mut Self,
        _: &WlSurface,
        _: <WlSurface as wayland_client::Proxy>::Event,
        _: &(),
        _: &WlConn,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for SctkState {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: <ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &WlConn,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, SurfaceId> for SctkState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: LayerSurfaceEvent,
        sid: &SurfaceId,
        _: &WlConn,
        _: &QueueHandle<Self>,
    ) {
        match event {
            LayerSurfaceEvent::Configure { serial, width, height } => {
                state.pending_configures.lock().unwrap().push(PendingConfigure {
                    serial,
                    width,
                    height,
                    surface_id: sid.0,
                });
                let _ = proxy;
            }
            LayerSurfaceEvent::Closed => {
                eprintln!("slint[layer-shell]: surface_id={} closed by compositor", sid.0);
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct SurfaceId(usize);

// Public manager. One per process; lives on `SharedBackendData`.
//
// Threading model: an internal dispatcher thread owns the SCTK `EventQueue`
// and blocks in `EventQueue::blocking_dispatch()`. After every successful
// dispatch it wakes the main event loop via `EventLoopProxy::send_event` so
// that pending compositor configures + frame callbacks for layer surfaces are
// picked up promptly without busy-polling. The main thread retains the
// connection + queue handle for issuing requests (creating surfaces,
// committing buffers).

pub struct LayerShellManager {
    conn: WlConn,
    qh: QueueHandle<SctkState>,
    compositor: WlCompositor,
    layer_shell: ZwlrLayerShellV1,
    next_id: Cell<usize>,
    pending_configures: Arc<Mutex<Vec<PendingConfigure>>>,
    /// Set to false on Drop to terminate the dispatcher thread.
    dispatcher_alive: Arc<std::sync::atomic::AtomicBool>,
    /// Joined on Drop; the thread exits when `dispatcher_alive` flips false.
    dispatcher_thread: RefCell<Option<std::thread::JoinHandle<()>>>,
}

impl LayerShellManager {
    pub fn new(
        proxy: winit::event_loop::EventLoopProxy<crate::SlintEvent>,
    ) -> Result<Rc<Self>, String> {
        let conn = WlConn::connect_to_env()
            .map_err(|e| format!("layer_shell: connect_to_env failed: {e}"))?;
        let (globals, queue) = registry_queue_init::<SctkState>(&conn)
            .map_err(|e| format!("layer_shell: registry init failed: {e}"))?;
        let qh = queue.handle();

        let compositor: WlCompositor = bind_one(&globals, &qh, 1..=6, "wl_compositor")?;
        let layer_shell: ZwlrLayerShellV1 = bind_one(&globals, &qh, 1..=4, "zwlr_layer_shell_v1")?;

        let pending_configures = Arc::new(Mutex::new(Vec::new()));
        let state = SctkState {
            registry_state: RegistryState::new(&globals),
            pending_configures: pending_configures.clone(),
        };

        let dispatcher_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let dispatcher_thread =
            spawn_dispatcher(conn.clone(), queue, state, proxy, dispatcher_alive.clone());

        Ok(Rc::new(Self {
            conn,
            qh,
            compositor,
            layer_shell,
            next_id: Cell::new(1),
            pending_configures,
            dispatcher_alive,
            dispatcher_thread: RefCell::new(Some(dispatcher_thread)),
        }))
    }

    pub fn create_surface(
        self: &Rc<Self>,
        role: &LayerShellRole,
    ) -> Result<LayerSurfaceHandle, String> {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let wl_surface = self.compositor.create_surface(&self.qh, ());
        let layer_surface = self.layer_shell.get_layer_surface(
            &wl_surface,
            None, // output: let compositor pick
            role.layer.into(),
            role.namespace.clone(),
            &self.qh,
            SurfaceId(id),
        );
        layer_surface.set_anchor(role.anchor.into());
        layer_surface.set_exclusive_zone(role.exclusive_zone);
        layer_surface.set_size(role.size.0, role.size.1);
        layer_surface.set_keyboard_interactivity(role.keyboard_interactivity.into());
        wl_surface.commit();
        // Flush so the compositor sees the request and can respond with configure.
        let _ = self.conn.flush();

        Ok(LayerSurfaceHandle {
            mgr: self.clone(),
            id,
            wl_surface,
            layer_surface,
            current_size: Cell::new(role.size),
        })
    }

    pub fn connection(&self) -> &WlConn {
        &self.conn
    }
}

impl Drop for LayerShellManager {
    fn drop(&mut self) {
        self.dispatcher_alive.store(false, std::sync::atomic::Ordering::Release);
        // Wake the dispatcher so it observes the flag and exits its blocking
        // dispatch. A flush with no pending requests is a no-op on the wire
        // but still kicks libwayland's poll fd.
        let _ = self.conn.flush();
        if let Some(handle) = self.dispatcher_thread.borrow_mut().take() {
            let _ = handle.join();
        }
    }
}

/// Background thread that owns the SCTK queue and wakes the main event loop
/// after every successful dispatch. Terminates when `alive` is set to false.
fn spawn_dispatcher(
    conn: WlConn,
    mut queue: EventQueue<SctkState>,
    mut state: SctkState,
    proxy: winit::event_loop::EventLoopProxy<crate::SlintEvent>,
    alive: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("slint-layer-shell".into())
        .spawn(move || {
            let _conn = conn; // keep the connection alive for the thread's lifetime
            while alive.load(std::sync::atomic::Ordering::Acquire) {
                match queue.blocking_dispatch(&mut state) {
                    Ok(0) => continue, // spurious wakeup, no events to forward
                    Ok(_) => {
                        if proxy
                            .send_event(crate::SlintEvent(
                                crate::event_loop::CustomEvent::LayerShellWake,
                            ))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("slint[layer-shell]: dispatcher exited on error: {e}");
                        break;
                    }
                }
            }
        })
        .expect("spawn slint-layer-shell thread")
}

fn bind_one<I>(
    globals: &GlobalList,
    qh: &QueueHandle<SctkState>,
    range: std::ops::RangeInclusive<u32>,
    name: &str,
) -> Result<I, String>
where
    I: wayland_client::Proxy + 'static,
    SctkState: Dispatch<I, ()>,
{
    globals
        .bind::<I, _, _>(qh, range, ())
        .map_err(|e| format!("layer_shell: bind {name} failed: {e}"))
}

/// Per-surface handle returned to the WinitWindowAdapter. Holds the wl_surface
/// the renderer paints into and exposes configure ack.
pub struct LayerSurfaceHandle {
    mgr: Rc<LayerShellManager>,
    id: usize,
    pub wl_surface: WlSurface,
    layer_surface: ZwlrLayerSurfaceV1,
    current_size: Cell<(u32, u32)>,
}

impl LayerSurfaceHandle {
    pub fn current_size(&self) -> (u32, u32) {
        self.current_size.get()
    }

    /// True if a configure is queued but not yet acked. Cheap O(n) over
    /// pending_configures for this surface_id.
    pub fn has_pending_configure(&self) -> bool {
        let q = self.mgr.pending_configures.lock().unwrap();
        q.iter().any(|c| c.surface_id == self.id)
    }

    /// Pop the latest pending configure for this surface (if any), ack it, and
    /// return the new size. Caller should resize their buffer accordingly.
    pub fn take_pending_configure(&self) -> Option<(u32, u32)> {
        let mut q = self.mgr.pending_configures.lock().unwrap();
        let idx = q.iter().rposition(|c| c.surface_id == self.id)?;
        let c = q.remove(idx);
        // Discard older configures for the same surface — only the newest matters.
        q.retain(|other| other.surface_id != self.id);
        self.layer_surface.ack_configure(c.serial);
        let size = (c.width, c.height);
        self.current_size.set(size);
        Some(size)
    }

    pub fn commit(&self) {
        self.wl_surface.commit();
        let _ = self.mgr.conn.flush();
    }

    /// Internal accessor for the renderer module to grab the underlying
    /// connection (needed to extract raw libwayland pointers for softbuffer).
    pub fn connection_for_renderer(&self) -> &WlConn {
        &self.mgr.conn
    }
}

impl Drop for LayerSurfaceHandle {
    fn drop(&mut self) {
        self.layer_surface.destroy();
        self.wl_surface.destroy();
        let _ = self.mgr.conn.flush();
    }
}
