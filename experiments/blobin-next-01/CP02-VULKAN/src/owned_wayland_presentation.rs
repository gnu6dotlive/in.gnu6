//! EXPERIMENTAL: Wayland presentation + frame callback on an owned surface.
//! Used only in isolated GPU05-PACER worktree binaries.
//! The eventual Vulkan host must use this exact connection and wl_surface for
//! both its Vulkan WSI and wp_presentation.feedback requests.
//!
//! Dependencies when admitted:
//! wayland-client = {version="=0.31.15",features=["system"]}
//! wayland-protocols = {version="=0.32.13",features=["client"]}
//! raw-window-handle = "0.6.2"
//! Integration requires an audited native window lifetime + wgpu raw-handle wrapper.
#![allow(dead_code)]
#[path = "presentation_tracker.rs"]
mod presentation_tracker;

use presentation_tracker::{FeedbackOutcome, FeedbackTracker, PresentationReceipt};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use std::{
    collections::{HashMap, HashSet},
    os::fd::AsRawFd,
    ptr::NonNull,
};
use wayland_client::{
    delegate_noop,
    protocol::{
        wl_callback, wl_compositor, wl_output, wl_pointer, wl_registry, wl_seat, wl_surface,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::{
    wp::presentation_time::client::{wp_presentation, wp_presentation_feedback},
    xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

#[derive(Debug, Clone, Copy)]
pub struct FrameCallbackReceipt {
    pub frame_sequence: u64,
    pub compositor_callback_ms: u32,
}
#[derive(Debug, Clone, Copy)]
pub struct OpticalPointerButton {
    pub pressed: bool,
    pub compositor_time_ms: u32,
    pub provider_sequence: u64,
    pub surface_x: f64,
    pub surface_y: f64,
}
fn valid_pointer_coordinate(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite() && (0.0..=4096.0).contains(&x) && (0.0..=4096.0).contains(&y)
}
#[cfg(test)]
mod optical_pointer_tests {
    use super::valid_pointer_coordinate;
    #[test]
    fn finite_surface_coordinates_required() {
        assert!(valid_pointer_coordinate(128.25, 64.5));
        assert!(!valid_pointer_coordinate(f64::NAN, 1.0));
        assert!(!valid_pointer_coordinate(1.0, f64::INFINITY));
        assert!(!valid_pointer_coordinate(-1.0, 5.0));
        assert!(!valid_pointer_coordinate(50.0, 4097.0));
    }
}
const MAX_OPTICAL_POINTER_EVENTS: usize = 16;
const MAX_CALLBACKS: usize = 2;
const MAX_QUEUED_CALLBACK_RECEIPTS: usize = 8;

#[derive(Debug)]
struct NativeState {
    compositor: Option<wl_compositor::WlCompositor>,
    optical_probe_requested: bool,
    seat: Option<wl_seat::WlSeat>,
    pointer: Option<wl_pointer::WlPointer>,
    optical_surface: Option<wl_surface::WlSurface>,
    pointer_inside: bool,
    pointer_position: Option<(f64, f64)>,
    pointer_sequence: u64,
    optical_events: Vec<OpticalPointerButton>,
    outputs: HashMap<u32, wl_output::WlOutput>,
    output_names: HashMap<u32, String>,
    shell: Option<xdg_wm_base::XdgWmBase>,
    presentation: Option<wp_presentation::WpPresentation>,
    configured: bool,
    suggested_size: Option<(u32, u32)>,
    closed: bool,
    tracker: FeedbackTracker,
    callback_pending: HashSet<u64>,
    callback_receipts: Vec<FrameCallbackReceipt>,
    protocol_error: Option<&'static str>,
}

impl Default for NativeState {
    fn default() -> Self {
        Self {
            compositor: None,
            optical_probe_requested: false,
            seat: None,
            pointer: None,
            optical_surface: None,
            pointer_inside: false,
            pointer_position: None,
            pointer_sequence: 0,
            optical_events: Vec::with_capacity(MAX_OPTICAL_POINTER_EVENTS),
            outputs: HashMap::new(),
            output_names: HashMap::new(),
            shell: None,
            presentation: None,
            configured: false,
            suggested_size: None,
            closed: false,
            tracker: FeedbackTracker::new(),
            callback_pending: HashSet::new(),
            callback_receipts: Vec::new(),
            protocol_error: None,
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for NativeState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" if state.compositor.is_none() => {
                    state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    ));
                }
                "wl_seat"
                    if state.optical_probe_requested && state.seat.is_none() && version >= 5 =>
                {
                    state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, 5, qh, ()));
                }
                "wl_output" if version >= 4 => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 4, qh, ());
                    state.outputs.insert(output.id().protocol_id(), output);
                }
                "xdg_wm_base" if state.shell.is_none() => {
                    state.shell = Some(registry.bind::<xdg_wm_base::XdgWmBase, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ));
                }
                "wp_presentation" if state.presentation.is_none() => {
                    state.presentation =
                        Some(registry.bind::<wp_presentation::WpPresentation, _, _>(
                            name,
                            version.min(2),
                            qh,
                            (),
                        ));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for NativeState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(caps),
        } = event
        {
            if caps.contains(wl_seat::Capability::Pointer) && state.pointer.is_none() {
                state.pointer = Some(seat.get_pointer(qh, ()));
            }
            if !caps.contains(wl_seat::Capability::Pointer) {
                state.pointer_inside = false;
                state.pointer_position = None;
                // No pointer input can be accepted after capability removal.
            }
        }
    }
}
impl Dispatch<wl_pointer::WlPointer, ()> for NativeState {
    fn event(
        state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface,
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_inside = state
                    .optical_surface
                    .as_ref()
                    .is_some_and(|own| *own == surface);
                state.pointer_position =
                    if state.pointer_inside && valid_pointer_coordinate(surface_x, surface_y) {
                        Some((surface_x, surface_y))
                    } else {
                        None
                    };
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } if state.pointer_inside => {
                state.pointer_position = if valid_pointer_coordinate(surface_x, surface_y) {
                    Some((surface_x, surface_y))
                } else {
                    None
                };
            }
            wl_pointer::Event::Leave { .. } => {
                state.pointer_inside = false;
                state.pointer_position = None;
            }
            wl_pointer::Event::Button {
                button,
                time,
                state: button_state,
                ..
            } if state.pointer_inside && button == 0x110 => {
                let pressed = match button_state {
                    WEnum::Value(wl_pointer::ButtonState::Pressed) => true,
                    WEnum::Value(wl_pointer::ButtonState::Released) => false,
                    _ => return,
                };
                if state.optical_events.len() >= MAX_OPTICAL_POINTER_EVENTS {
                    state.protocol_error = Some("ORACLE02_POINTER_EVENT_QUEUE_FULL");
                    return;
                }
                let Some((surface_x, surface_y)) = state.pointer_position else {
                    state.protocol_error = Some("ORACLE03_BUTTON_WITHOUT_SURFACE_POSITION");
                    return;
                };
                let Some(provider_sequence) = state.pointer_sequence.checked_add(1) else {
                    state.protocol_error = Some("ORACLE03_POINTER_SEQUENCE_EXHAUSTED");
                    return;
                };
                state.pointer_sequence = provider_sequence;
                state.optical_events.push(OpticalPointerButton {
                    pressed,
                    compositor_time_ms: time,
                    provider_sequence,
                    surface_x,
                    surface_y,
                });
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for NativeState {
    fn event(
        state: &mut Self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event {
            state.output_names.insert(output.id().protocol_id(), name);
        }
    }
}

impl Dispatch<wp_presentation::WpPresentation, ()> for NativeState {
    fn event(
        state: &mut Self,
        _: &wp_presentation::WpPresentation,
        event: wp_presentation::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation::Event::ClockId { clk_id } = event {
            if let Err(e) = state.tracker.clock_id(clk_id) {
                state.protocol_error = Some(e);
            }
        }
    }
}

impl Dispatch<wp_presentation_feedback::WpPresentationFeedback, u32> for NativeState {
    fn event(
        state: &mut Self,
        _: &wp_presentation_feedback::WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        feedback_id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let outcome = match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh,
                ..
            } => Some(FeedbackOutcome::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh_ns: refresh,
            }),
            wp_presentation_feedback::Event::Discarded => Some(FeedbackOutcome::Discarded),
            wp_presentation_feedback::Event::SyncOutput { .. } => None,
            _ => None,
        };
        if let Some(outcome) = outcome {
            if let Err(e) = state.tracker.resolved(*feedback_id, outcome) {
                state.protocol_error = Some(e);
            }
        }
    }
}

impl Dispatch<wl_callback::WlCallback, u64> for NativeState {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        frame_sequence: &u64,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            if !state.callback_pending.remove(frame_sequence) {
                state.protocol_error = Some("GPU05_UNSOLICITED_OR_REPEATED_FRAME_CALLBACK");
                return;
            }
            if state.callback_receipts.len() >= MAX_QUEUED_CALLBACK_RECEIPTS {
                state.protocol_error = Some("GPU05_FRAME_CALLBACK_RECEIPTS_NOT_DRAINED");
                return;
            }
            state.callback_receipts.push(FrameCallbackReceipt {
                frame_sequence: *frame_sequence,
                compositor_callback_ms: callback_data,
            });
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for NativeState {
    fn event(
        _: &mut Self,
        shell: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            shell.pong(serial);
        }
    }
}
impl Dispatch<xdg_surface::XdgSurface, ()> for NativeState {
    fn event(
        state: &mut Self,
        xdg: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            xdg.ack_configure(serial);
            if !state.configured {
                println!("GPU05_PACER06_XDG_FIRST_CONFIGURE_ACK serial={serial}");
            }
            state.configured = true;
        }
    }
}
impl Dispatch<xdg_toplevel::XdgToplevel, ()> for NativeState {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Close => state.closed = true,
            xdg_toplevel::Event::Configure { width, height, .. } if width > 0 && height > 0 => {
                state.suggested_size = Some((width as u32, height as u32));
            }
            _ => {}
        }
    }
}
delegate_noop!(NativeState: ignore wl_compositor::WlCompositor);
delegate_noop!(NativeState: ignore wl_surface::WlSurface);

/// Own this object for strictly longer than any Vulkan swapchain created from
/// raw Wayland handles. Never spawn another reader for this connection.
pub struct OwnedWaylandPresentation {
    // Vulkan surface resides in OwnedGpuWindow and drops before this owner.
    surface: wl_surface::WlSurface,
    _xdg_toplevel: xdg_toplevel::XdgToplevel,
    _xdg_surface: xdg_surface::XdgSurface,
    state: NativeState,
    qh: QueueHandle<NativeState>,
    events: EventQueue<NativeState>,
    connection: Connection,
    next_feedback_id: u32,
}

impl OwnedWaylandPresentation {
    pub fn open() -> Result<Self, String> {
        Self::open_on(None)
    }
    pub fn open_on(requested_output: Option<&str>) -> Result<Self, String> {
        Self::open_on_probe(requested_output, false)
    }
    pub fn open_on_probe(
        requested_output: Option<&str>,
        enable_probe: bool,
    ) -> Result<Self, String> {
        let connection = Connection::connect_to_env().map_err(|e| e.to_string())?;
        let mut events = connection.new_event_queue::<NativeState>();
        let qh = events.handle();
        let _registry = connection.display().get_registry(&qh, ());
        let mut state = NativeState {
            optical_probe_requested: enable_probe,
            ..Default::default()
        };
        events.roundtrip(&mut state).map_err(|e| e.to_string())?;
        if requested_output.is_some() {
            // wl_output names arrive only after the registry bind roundtrip.
            events.roundtrip(&mut state).map_err(|e| e.to_string())?;
        }
        let selected_output = requested_output
            .map(|wanted| {
                state
                    .output_names
                    .iter()
                    .find(|(_, name)| name.as_str() == wanted)
                    .and_then(|(id, _)| state.outputs.get(id))
                    .cloned()
                    .ok_or_else(|| format!("GPU05_REQUESTED_OUTPUT_UNAVAILABLE {wanted}"))
            })
            .transpose()?;

        let compositor = state.compositor.clone().ok_or("GPU05_NO_COMPOSITOR")?;
        let shell = state.shell.clone().ok_or("GPU05_NO_XDG_SHELL")?;
        if state.presentation.is_none() {
            return Err("GPU05_NO_WP_PRESENTATION".into());
        }
        if enable_probe {
            if state.seat.is_none() {
                return Err("ORACLE02_NO_WL_SEAT".into());
            }
            events.roundtrip(&mut state).map_err(|e| e.to_string())?;
            if state.pointer.is_none() {
                return Err("ORACLE02_NO_WL_POINTER".into());
            }
        }
        let surface = compositor.create_surface(&qh, ());
        if enable_probe {
            state.optical_surface = Some(surface.clone());
        }
        let xdg_surface = shell.get_xdg_surface(&surface, &qh, ());
        let xdg_toplevel = xdg_surface.get_toplevel(&qh, ());
        xdg_toplevel.set_title("blob.in GPU-05 native owned surface".into());
        if let Some(output) = selected_output.as_ref() {
            // Protocol-owned monitor choice, not a Hyprland configuration edit.
            xdg_toplevel.set_fullscreen(Some(output));
            println!(
                "GPU05_PINNED_OUTPUT name={} protocol=xdg_toplevel.fullscreen",
                requested_output.unwrap_or_default()
            );
        }
        // Initial xdg_surface configure must precede the first Vulkan commit.
        println!("GPU05_PACER06_INITIAL_EMPTY_SURFACE_COMMIT");
        surface.commit();
        for _ in 0..8 {
            if state.configured && state.tracker.clock().is_some() {
                break;
            }
            events.roundtrip(&mut state).map_err(|e| e.to_string())?;
        }
        if !state.configured || state.tracker.clock().is_none() {
            return Err("GPU05_NOT_CONFIGURED_OR_CLOCK_MISSING".into());
        }
        state.tracker.begin_surface(1).map_err(str::to_string)?;
        println!("GPU05_PACER06_CONFIGURED_BEFORE_VULKAN_SWAPCHAIN status=true");
        if let Some(e) = state.protocol_error {
            return Err(e.into());
        }
        Ok(Self {
            surface,
            _xdg_toplevel: xdg_toplevel,
            _xdg_surface: xdg_surface,
            state,
            qh,
            events,
            connection,
            next_feedback_id: 1,
        })
    }

    /// Allocate feedback *for this wl_surface* immediately before the Vulkan
    /// WSI wl_surface.commit triggered by queue.present. There MUST be exactly
    /// one owner event loop for both the Wayland proxies and Vulkan raw handles.
    pub fn arm_before_present(&mut self, frame_seq: u64) -> Result<u32, String> {
        if self.state.closed {
            return Err("GPU05_SURFACE_CLOSED".into());
        }
        if let Some(reason) = self.state.protocol_error {
            return Err(reason.into());
        }
        if self.state.callback_pending.len() >= MAX_CALLBACKS {
            return Err("GPU05_TOO_MANY_PENDING_FRAME_CALLBACKS".into());
        }
        if self.state.callback_pending.contains(&frame_seq) {
            return Err("GPU05_DUPLICATE_FRAME_CALLBACK_SEQUENCE".into());
        }
        let id = self.next_feedback_id;
        let next_id = id.checked_add(1).ok_or("GPU05_FEEDBACK_ID_EXHAUSTED")?;
        let presentation = self
            .state
            .presentation
            .as_ref()
            .ok_or("GPU05_PRESENTATION_PROXY_MISSING")?;
        // Tracker validates before an irreversible protocol request.
        self.state
            .tracker
            .requested(id, frame_seq)
            .map_err(|e| e.to_owned())?;
        // Both requests are directed to the *same* wl_surface before the
        // upcoming Vulkan WSI commit. Never use an independent client.
        self.state.callback_pending.insert(frame_seq);
        let _callback = self.surface.frame(&self.qh, frame_seq);
        let _feedback = presentation.feedback(&self.surface, &self.qh, id);
        if frame_seq == 1 {
            println!("GPU05_PACER06_FIRST_FRAME_FEEDBACK_ARMED seq=1");
        }
        self.connection.flush().map_err(|e| e.to_string())?;
        self.next_feedback_id = next_id;
        Ok(id)
    }

    /// Process compositor events on the owner thread; never read this
    /// connection on another thread or via a foreign-client event queue.
    pub fn poll_feedback(&mut self) -> Result<Vec<PresentationReceipt>, String> {
        // dispatch_pending alone does not read new events off the socket.
        self.events.flush().map_err(|e| e.to_string())?;
        self.events
            .dispatch_pending(&mut self.state)
            .map_err(|e| e.to_string())?;
        if let Some(guard) = self.events.prepare_read() {
            let mut pfd = libc::pollfd {
                fd: guard.connection_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one valid pollfd on stack, timeout=0; no blocking render.
            let available = unsafe { libc::poll(&raw mut pfd, 1, 0) };
            if available < 0 {
                return Err(format!(
                    "GPU05_POLL_FAILED {}",
                    std::io::Error::last_os_error()
                ));
            }
            if available > 0 {
                guard.read().map_err(|e| e.to_string())?;
            }
        }
        self.events
            .dispatch_pending(&mut self.state)
            .map_err(|e| e.to_string())?;
        if let Some(e) = self.state.protocol_error {
            return Err(e.into());
        }
        Ok(self.state.tracker.drain_receipts())
    }

    pub fn drain_frame_callbacks(&mut self) -> Result<Vec<FrameCallbackReceipt>, String> {
        if let Some(reason) = self.state.protocol_error {
            return Err(reason.into());
        }
        Ok(std::mem::take(&mut self.state.callback_receipts))
    }

    pub fn frame_callback_pending(&self) -> usize {
        self.state.callback_pending.len()
    }
    pub fn drain_optical_buttons(&mut self) -> Result<Vec<OpticalPointerButton>, String> {
        if let Some(reason) = self.state.protocol_error {
            return Err(reason.into());
        }
        Ok(std::mem::take(&mut self.state.optical_events))
    }

    /// Only for diagnostics: roundtrip may block and change frame pacing.
    /// Production should poll the owner's Wayland connection with a single
    /// registered FD and call dispatch_pending after reading events.
    pub fn diagnostic_roundtrip(&mut self) -> Result<Vec<PresentationReceipt>, String> {
        self.events
            .roundtrip(&mut self.state)
            .map_err(|e| e.to_string())?;
        if let Some(reason) = self.state.protocol_error {
            return Err(reason.into());
        }
        Ok(self.state.tracker.drain_receipts())
    }

    pub fn surface(&self) -> &wl_surface::WlSurface {
        &self.surface
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn clock_id(&self) -> Option<u32> {
        self.state.tracker.clock()
    }
    pub fn pending_count(&self) -> usize {
        self.state.tracker.unresolved()
    }
    pub fn is_closed(&self) -> bool {
        self.state.closed
    }
    pub fn suggested_size(&self) -> (u32, u32) {
        self.state.suggested_size.unwrap_or((960, 648))
    }
}

/// The Vulkan surface owns no Wayland objects. Field order enforces Vulkan
/// surface destruction before the underlying native Wayland objects.
pub struct OwnedGpuWindow {
    surface: wgpu::Surface<'static>,
    host: OwnedWaylandPresentation,
}
impl OwnedGpuWindow {
    pub fn open(instance: &wgpu::Instance) -> Result<Self, String> {
        Self::open_on(instance, None)
    }
    pub fn open_on(instance: &wgpu::Instance, output: Option<&str>) -> Result<Self, String> {
        Self::open_on_probe(instance, output, false)
    }
    pub fn open_on_probe(
        instance: &wgpu::Instance,
        output: Option<&str>,
        enable_probe: bool,
    ) -> Result<Self, String> {
        let host = OwnedWaylandPresentation::open_on_probe(output, enable_probe)?;
        let display_ptr = NonNull::new(host.connection.backend().display_ptr().cast())
            .ok_or("GPU05_NULL_WAYLAND_DISPLAY")?;
        let wl_ptr =
            NonNull::new(host.surface.id().as_ptr().cast()).ok_or("GPU05_NULL_WAYLAND_SURFACE")?;
        let display = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display_ptr));
        let window = RawWindowHandle::Wayland(WaylandWindowHandle::new(wl_ptr));
        // SAFETY: the host owns both libwayland connection and wl_surface.
        // OwnedGpuWindow keeps their storage live, and its surface drops
        // before the host (Rust struct field drop order).
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(display),
                raw_window_handle: window,
            })
        }
        .map_err(|e| format!("GPU05_VULKAN_SURFACE_CREATION {e}"))?;
        Ok(Self { surface, host })
    }
    pub fn wgpu_surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
    }
    pub fn arm(&mut self, sequence: u64) -> Result<u32, String> {
        self.host.arm_before_present(sequence)
    }
    pub fn poll(&mut self) -> Result<Vec<PresentationReceipt>, String> {
        self.host.poll_feedback()
    }
    pub fn frame_callbacks(&mut self) -> Result<Vec<FrameCallbackReceipt>, String> {
        self.host.drain_frame_callbacks()
    }
    pub fn optical_buttons(&mut self) -> Result<Vec<OpticalPointerButton>, String> {
        self.host.drain_optical_buttons()
    }
    pub fn callback_pending(&self) -> usize {
        self.host.frame_callback_pending()
    }
    pub fn clock(&self) -> Option<u32> {
        self.host.clock_id()
    }
    pub fn pending(&self) -> usize {
        self.host.pending_count()
    }
    pub fn closed(&self) -> bool {
        self.host.is_closed()
    }
    pub fn size(&self) -> (u32, u32) {
        self.host.suggested_size()
    }
    pub fn drain_for_shutdown(&mut self) -> Result<Vec<PresentationReceipt>, String> {
        self.host.diagnostic_roundtrip()
    }
}
