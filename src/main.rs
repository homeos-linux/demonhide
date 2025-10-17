use glib::MainLoop;
use log::{debug, info, warn, error};
use std::ptr;
use wayland_client::protocol::{
    wl_compositor, wl_output, wl_pointer, wl_registry, wl_seat, wl_shell, wl_shell_surface,
    wl_surface,
};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_locked_pointer_v1, zwp_pointer_constraints_v1,
};

fn should_lock_pointer() -> bool {
    // Check if we're in a Wayland session
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        return false; // Not in Wayland
    }

    // Check if there's an X11 display (XWayland)
    if std::env::var("DISPLAY").is_err() {
        return false; // No XWayland
    }

    // Check XWayland for fullscreen applications with hidden cursor
    check_xwayland_fullscreen_with_hidden_cursor()
}

fn check_xwayland_fullscreen_with_hidden_cursor() -> bool {
    unsafe {
        let display = x11::xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            return false;
        }

        let root = x11::xlib::XDefaultRootWindow(display);
        let screen = x11::xlib::XDefaultScreen(display);
        let screen_width = x11::xlib::XDisplayWidth(display, screen);
        let screen_height = x11::xlib::XDisplayHeight(display, screen);

        // Get the currently focused window
        let mut focus_window = 0;
        let mut revert_to = 0;
        x11::xlib::XGetInputFocus(display, &mut focus_window, &mut revert_to);

        if focus_window == 0 || focus_window == root {
            x11::xlib::XCloseDisplay(display);
            return false;
        }

        // Get window attributes
        let mut window_attrs = std::mem::zeroed();
        if x11::xlib::XGetWindowAttributes(display, focus_window, &mut window_attrs) == 0 {
            x11::xlib::XCloseDisplay(display);
            return false;
        }

        // Check if window is fullscreen (covers entire screen)
        let is_fullscreen =
            window_attrs.width >= screen_width && window_attrs.height >= screen_height;

        if !is_fullscreen {
            x11::xlib::XCloseDisplay(display);
            return false;
        }

        // Check if cursor is hidden using XFixes
        let mut event_base = 0;
        let mut error_base = 0;

        if x11::xfixes::XFixesQueryExtension(display, &mut event_base, &mut error_base) == 0 {
            x11::xlib::XCloseDisplay(display);
            return false; // XFixes not available
        }

        let cursor_image = x11::xfixes::XFixesGetCursorImage(display);
        let cursor_hidden = if cursor_image.is_null() {
            true // If we can't get cursor info, assume it might be hidden
        } else {
            let cursor = &*cursor_image;
            // Cursor is considered hidden if it has no dimensions or is 1x1 (common for hidden cursors)
            cursor.width <= 1 && cursor.height <= 1
        };

        if !cursor_image.is_null() {
            x11::xlib::XFree(cursor_image as *mut _);
        }

        x11::xlib::XCloseDisplay(display);

        cursor_hidden
    }
}

struct AppData {
    pointer_constraints: Option<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1>,
    seat: Option<wl_seat::WlSeat>,
    pointer: Option<wl_pointer::WlPointer>,
    compositor: Option<wl_compositor::WlCompositor>,
    surface: Option<wl_surface::WlSurface>,
    shell: Option<wl_shell::WlShell>,
    locked_pointer: Option<zwp_locked_pointer_v1::ZwpLockedPointerV1>,
    // Per-output info: (wl_output, Arc<Mutex<Option<(x,y,width,height,scale)>>>)
    outputs: Vec<(wl_output::WlOutput, std::sync::Arc<std::sync::Mutex<Option<(i32, i32, i32, i32, i32)>>>)>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "zwp_pointer_constraints_v1" => {
                    let pointer_constraints = registry
                        .bind::<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    );
                    debug!("Bound pointer constraints interface");
                    state.pointer_constraints = Some(pointer_constraints);
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                    debug!("Bound seat interface - requesting capabilities...");
                    state.seat = Some(seat);
                }
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
                    debug!("Bound compositor interface");
                    state.compositor = Some(compositor);
                }
                "wl_output" => {
                    // Bind wl_output and keep the object and an associated info slot to receive events
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 3, qh, ());
                    let info = std::sync::Arc::new(std::sync::Mutex::new(None));
                    debug!("Bound wl_output interface");
                    state.outputs.push((output, info));
                }
                "wl_shell" => {
                    let shell = registry.bind::<wl_shell::WlShell, _, _>(name, 1, qh, ());
                    debug!("Bound shell interface");
                    state.shell = Some(shell);
                }
                _ => {}
            }
        }
    }
}

// Implement required trait dispatches
impl Dispatch<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwp_pointer_constraints_v1::ZwpPointerConstraintsV1,
        _: zwp_pointer_constraints_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_shell::WlShell, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shell::WlShell,
        _: wl_shell::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_shell_surface::WlShellSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shell_surface::WlShellSurface,
        _: wl_shell_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities } = event {
                debug!("Seat capabilities received: {:?}", capabilities);

            // Try different ways to check for pointer capability
            let caps_value: u32 = capabilities.into();
            let pointer_bit = u32::from(wl_seat::Capability::Pointer);

                debug!(
                    "Capabilities value: {}, Pointer bit: {}",
                    caps_value, pointer_bit
                );

            if (caps_value & pointer_bit) != 0 {
                let pointer = seat.get_pointer(qh, ());
                info!("Got pointer capability and created pointer device");
                state.pointer = Some(pointer);
            } else {
                warn!("No pointer capability available");
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        state: &mut Self,
        _output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        // Find the matching stored output and update its info
        for (stored_output, info_arc) in &state.outputs {
            if stored_output == _output {
                if let Ok(mut guard) = info_arc.lock() {
                    match event {
                        wl_output::Event::Geometry { x, y, physical_width: _, physical_height: _, subpixel: _, make: _, model: _, transform: _ } => {
                            // store x,y (position); other fields handled elsewhere
                            let (w, h, scale) = guard.unwrap_or((0, 0, 1));
                            *guard = Some((x as i32, y as i32, w, h, scale));
                            debug!("wl_output geometry: x={} y={}", x, y);
                        }
                        wl_output::Event::Mode { flags: _, width, height, refresh: _ } => {
                            let (x, y, _w, _h, scale) = guard.unwrap_or((0, 0, 0, 0, 1));
                            *guard = Some((x, y, width as i32, height as i32, scale));
                            debug!("wl_output mode: {}x{} (stored pos {}x{}) scale={}", width, height, x, y, scale);
                        }
                        wl_output::Event::Scale { factor } => {
                            let (x, y, w, h, _old_scale) = guard.unwrap_or((0, 0, 0, 0, 1));
                            *guard = Some((x, y, w, h, factor as i32));
                            debug!("wl_output scale event: factor={}", factor);
                        }
                        wl_output::Event::Done => {
                            // nothing special
                        }
                        _ => {}
                    }
                }
                break;
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_pointer::WlPointer,
        _: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<zwp_locked_pointer_v1::ZwpLockedPointerV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &zwp_locked_pointer_v1::ZwpLockedPointerV1,
        event: zwp_locked_pointer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        match event {
            zwp_locked_pointer_v1::Event::Locked => {
                info!("🔒 Pointer successfully locked!");
            }
            zwp_locked_pointer_v1::Event::Unlocked => {
                info!("🔓 Pointer unlocked");
                // Clear the locked pointer when it's unlocked
                state.locked_pointer = None;
            }
            _ => {}
        }
    }
}

struct PointerLockDaemon {
    app_data: Option<AppData>,
    event_queue: Option<wayland_client::EventQueue<AppData>>,
    is_locked: bool, // Track current lock state
    warp_thread: Option<std::thread::JoinHandle<()>>, // Thread for warping cursor
    warp_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>, // Signal to stop warping
}

impl PointerLockDaemon {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to connect to Wayland
        match Connection::connect_to_env() {
            Ok(conn) => {
                info!("Connected to Wayland display");

                let mut app_data = AppData {
                    pointer_constraints: None,
                    seat: None,
                    pointer: None,
                    compositor: None,
                    surface: None,
                    shell: None,
                    locked_pointer: None,
                    outputs: Vec::new(),
                };

                let display = conn.display();
                let mut event_queue = conn.new_event_queue();
                let qh = event_queue.handle();

                // Get the registry and bind to global objects
                let _registry = display.get_registry(&qh, ());

                // First roundtrip to get all globals
                event_queue.blocking_dispatch(&mut app_data)?;

                // Second roundtrip to get seat capabilities after binding
                if app_data.seat.is_some() {
                    debug!("Doing second roundtrip to get seat capabilities...");
                    event_queue.blocking_dispatch(&mut app_data)?;
                }

                // Create surface even without shell - just a basic surface for pointer locking
                if let Some(compositor) = &app_data.compositor {
                    let surface = compositor.create_surface(&qh, ());
                    debug!("Created basic surface for pointer locking");

                    // Commit the surface to make it "live"
                    surface.commit();

                    app_data.surface = Some(surface);

                    debug!("Surface committed and ready for pointer locking");
                } else {
                    warn!("Warning: Missing compositor, cannot create surface");
                }

                info!("Wayland protocols initialized successfully");

                Ok(PointerLockDaemon {
                    app_data: Some(app_data),
                    event_queue: Some(event_queue),
                    is_locked: false,
                    warp_thread: None,
                    warp_stop: None,
                })
            }
            Err(e) => {
                println!(
                    "Failed to connect to Wayland: {}, running without pointer constraints",
                    e
                );
                Ok(PointerLockDaemon {
                    app_data: None,
                    event_queue: None,
                    is_locked: false,
                    warp_thread: None,
                    warp_stop: None,
                })
            }
        }
    }

    fn should_lock(&self) -> bool {
        should_lock_pointer()
    }

    fn get_wayland_surface_center(&self) -> Option<(i32, i32)> {
        if let Some(app_data) = &self.app_data {
            if let Some(_surface) = &app_data.surface {
                // 1) Try parsing GNOME monitors.xml into monitor list
                if let Some(monitors) = Self::parse_gnome_monitors_list() {
                    debug!("parse_gnome_monitors_list returned {} monitors", monitors.len());
                    for (i, m) in monitors.iter().enumerate() {
                        debug!("monitor[{}] = x={} y={} w={} h={} scale={}", i, m.0, m.1, m.2, m.3, m.4);
                    }

                    // If only one monitor, use its center
                    if monitors.len() == 1 {
                        let (x, y, w, h, scale) = monitors[0];
                        debug!("Single monitor detected, selecting center");
                        return Some(((w * scale) / 2 + x, (h * scale) / 2 + y));
                    }

                    // If multiple monitors, try to get focused X11 window center and pick containing monitor
                    if let Some((fx, fy)) = Self::get_focused_x11_window_center() {
                        debug!("Focused X11 window center at {}x{}", fx, fy);
                        for (mx, my, mw, mh, scale) in &monitors {
                            let rx = *mx;
                            let ry = *my;
                            let rw = *mw * *scale;
                            let rh = *mh * *scale;
                            let contains = fx >= rx && fx < rx + rw && fy >= ry && fy < ry + rh;
                            debug!("testing monitor rect x={} y={} w={} h={} contains={}", rx, ry, rw, rh, contains);
                            if contains {
                                debug!("Selected monitor containing focused point: x={} y={} w={} h={} scale={}", rx, ry, rw, rh, scale);
                                return Some(((rw) / 2 + rx, (rh) / 2 + ry));
                            }
                        }
                    } else {
                        debug!("No focused X11 window center available to choose monitor");
                    }

                    // Fallback: use primary monitor (first)
                    let (x, y, w, h, scale) = monitors[0];
                    debug!("Falling back to primary monitor from monitors.xml");
                    return Some(((w * scale) / 2 + x, (h * scale) / 2 + y));
                }

                // 2) Prefer Wayland per-output info collected earlier
                if !app_data.outputs.is_empty() {
                    // Try to use focused X11 point to select the right output
                    if let Some((fx, fy)) = Self::get_focused_x11_window_center() {
                        for (_out, info_arc) in &app_data.outputs {
                            if let Ok(guard) = info_arc.lock() {
                                if let Some((ox, oy, ow, oh, scale)) = *guard {
                                    let rw = ow * scale;
                                    let rh = oh * scale;
                                    let contains = fx >= ox && fx < ox + rw && fy >= oy && fy < oy + rh;
                                    debug!("testing stored output x={} y={} w={} h={} scale={} contains={}", ox, oy, rw, rh, scale, contains);
                                    if contains {
                                        let center_x = ox + rw / 2;
                                        let center_y = oy + rh / 2;
                                        debug!("Selected stored output center {}x{}", center_x, center_y);
                                        return Some((center_x, center_y));
                                    }
                                }
                            }
                        }
                    }
                    // otherwise fallback to first output's center
                    if let Some((_out, info_arc)) = app_data.outputs.get(0) {
                        if let Ok(guard) = info_arc.lock() {
                            if let Some((ox, oy, ow, oh, scale)) = *guard {
                                let center_x = ox + (ow * scale) / 2;
                                let center_y = oy + (oh * scale) / 2;
                                debug!("Falling back to first stored output center {}x{}", center_x, center_y);
                                return Some((center_x, center_y));
                            }
                        }
                    }
                }

                // 3) Fallback to environment variables (for older setups) or defaults
                let width = std::env::var("WAYLAND_SCREEN_WIDTH")
                    .ok()
                    .and_then(|w| w.parse::<i32>().ok())
                    .unwrap_or(1920);
                let height = std::env::var("WAYLAND_SCREEN_HEIGHT")
                    .ok()
                    .and_then(|h| h.parse::<i32>().ok())
                    .unwrap_or(1080);
                let scale = std::env::var("WAYLAND_SCREEN_SCALE")
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(1);
                let center_x = (width * scale) / 2;
                let center_y = (height * scale) / 2;
                return Some((center_x, center_y));
            }
        }
        None
    }

    // Parse GNOME monitors.xml into a vector of monitors: (x, y, width, height, scale)
    fn parse_gnome_monitors_list() -> Option<Vec<(i32, i32, i32, i32, i32)>> {
        use std::fs;
        let home = std::env::var("HOME").ok()?;
        let path = format!("{}/.config/monitors.xml", home);
        let contents = fs::read_to_string(path).ok()?;
        let lower = contents.to_lowercase();

        let mut monitors = Vec::new();

        // Very simple parse: find <monitor> or <logicalmonitor> entries and extract position/size/scale
        let mut idx = 0usize;
        while let Some(start) = lower[idx..].find('<') {
            idx += start;
            if lower[idx..].starts_with("<monitor") || lower[idx..].starts_with("<logicalmonitor") {
                let end_tag = if lower[idx..].starts_with("<monitor") { "</monitor>" } else { "</logicalmonitor>" };
                if let Some(end_rel) = lower[idx..].find(end_tag) {
                    let snippet = &lower[idx..idx + end_rel];
                    // extract tags
                    let extract = |tag: &str| -> Option<i32> {
                        let open = format!("<{}>", tag);
                        let close = format!("</{}>", tag);
                        let a = snippet.find(&open)? + open.len();
                        let b = snippet[a..].find(&close)? + a;
                        snippet[a..b].trim().parse::<i32>().ok()
                    };
                    let x = extract("x").unwrap_or(0);
                    let y = extract("y").unwrap_or(0);
                    let width = extract("width").or_else(|| extract("modewidth")).unwrap_or(1920);
                    let height = extract("height").or_else(|| extract("modeheight")).unwrap_or(1080);
                    let scale = extract("scale").unwrap_or(1);
                    monitors.push((x, y, width, height, scale));
                    idx += end_rel;
                    continue;
                }
            }
            idx += 1;
        }

        if monitors.is_empty() { None } else { Some(monitors) }
    }

    // Get center of the currently focused X11 window (root coordinates)
    fn get_focused_x11_window_center() -> Option<(i32, i32)> {
        unsafe {
            let display = x11::xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                return None;
            }
            let mut focus: x11::xlib::Window = 0;
            let mut revert: i32 = 0;
            x11::xlib::XGetInputFocus(display, &mut focus, &mut revert);
            if focus == 0 {
                x11::xlib::XCloseDisplay(display);
                return None;
            }
            let mut attrs: x11::xlib::XWindowAttributes = std::mem::zeroed();
            if x11::xlib::XGetWindowAttributes(display, focus, &mut attrs) == 0 {
                x11::xlib::XCloseDisplay(display);
                return None;
            }
            // Translate window coordinates to root
            let mut root_x = 0i32;
            let mut root_y = 0i32;
            let mut child_return: x11::xlib::Window = 0;
            x11::xlib::XTranslateCoordinates(
                display,
                focus,
                x11::xlib::XDefaultRootWindow(display),
                0,
                0,
                &mut root_x,
                &mut root_y,
                &mut child_return,
            );
            let center_x = root_x + attrs.width / 2;
            let center_y = root_y + attrs.height / 2;
            x11::xlib::XCloseDisplay(display);
            Some((center_x as i32, center_y as i32))
        }
    }

    fn lock_pointer(&mut self) {
        // Prevent multiple lock attempts
        if self.is_locked {
            return;
        }
        if let (Some(app_data), Some(event_queue)) = (&mut self.app_data, &mut self.event_queue) {
            if let (Some(pointer_constraints), Some(pointer), Some(surface)) = (
                &app_data.pointer_constraints,
                &app_data.pointer,
                &app_data.surface,
            ) {
                // Check if we already have a locked pointer object
                if app_data.locked_pointer.is_some() {
                    return;
                }
                info!("🔒 Locking pointer for XWayland fullscreen application with hidden cursor");
                // Lock the pointer to our surface
                let locked_pointer = pointer_constraints.lock_pointer(
                    surface,
                    pointer,
                    None, // No region restriction
                    zwp_pointer_constraints_v1::Lifetime::Persistent,
                    &event_queue.handle(),
                    (),
                );
                // Store the locked pointer
                app_data.locked_pointer = Some(locked_pointer);
                self.is_locked = true;
                // Process events to handle the lock response
                match event_queue.dispatch_pending(app_data) {
                    Ok(_) => {
                        info!("✅ Pointer lock request sent successfully");
                        // Start cursor warping thread
                        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let stop_flag_clone = stop_flag.clone();
                        let center = self.get_wayland_surface_center().unwrap_or((960, 540));
                        self.warp_stop = Some(stop_flag.clone());
                        self.warp_thread = Some(std::thread::spawn(move || {
                            use std::time::Duration;
                            unsafe {
                                let display = x11::xlib::XOpenDisplay(std::ptr::null());
                                if display.is_null() {
                                    error!("Could not open X display for warping");
                                    return;
                                }
                                let screen = x11::xlib::XDefaultScreen(display);
                                let root = x11::xlib::XRootWindow(display, screen);
                                let center_x = center.0;
                                let center_y = center.1;
                                    #[cfg(debug_assertions)]
                                {
                                    debug!("Starting cursor warping thread to ({}, {})", center_x, center_y);
                                }
                                while !stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                                    x11::xlib::XWarpPointer(
                                        display, 0, root, 0, 0, 0, 0, center_x, center_y,
                                    );
                                    x11::xlib::XFlush(display);
                                    std::thread::sleep(Duration::from_millis(250));
                                }
                                x11::xlib::XCloseDisplay(display);
                            }
                        }));
                    }
                    Err(_e) => {
                        error!("❌ Error processing pointer lock events: {}", _e);
                        self.is_locked = false; // Reset on error
                        self.warp_stop = None;
                        self.warp_thread = None;
                    }
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    if app_data.pointer_constraints.is_none() {
                        debug!("❌ Pointer constraints protocol not available");
                    }
                    if app_data.pointer.is_none() {
                        debug!("❌ Pointer device not available");
                    }
                    if app_data.surface.is_none() {
                        debug!("❌ Surface not available");
                    }
                    if app_data.compositor.is_none() {
                        debug!("❌ Compositor not available");
                    }
                }
            }
        }
    }

    fn unlock_pointer(&mut self) {
        // Only unlock if we're currently locked
        if !self.is_locked {
            return;
        }
        if let Some(app_data) = &mut self.app_data {
            if let Some(locked_pointer) = app_data.locked_pointer.take() {
                info!("🔓 Unlocking pointer...");
                locked_pointer.destroy();
                self.is_locked = false;
                // Stop cursor warping thread
                if let Some(stop_flag) = &self.warp_stop {
                    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if let Some(handle) = self.warp_thread.take() {
                    let _ = handle.join();
                }
                self.warp_stop = None;
                // Process events to handle the unlock (non-blocking)
                if let Some(event_queue) = &mut self.event_queue {
                    match event_queue.dispatch_pending(app_data) {
                        Ok(_) => {
                            info!("✅ Pointer unlock processed");
                        }
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            debug!("❌ Error processing pointer unlock events: {}", _e);
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        let should_lock = self.should_lock();

        if should_lock && !self.is_locked {
            self.lock_pointer();
        } else if !should_lock && self.is_locked {
            self.unlock_pointer();
        }
    }
}

fn main() {
    // Simple argument parsing for --help and --version
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                println!("DemonHide v{}", env!("CARGO_PKG_VERSION"));
                println!(
                    "Automatic pointer constraint daemon for XWayland fullscreen applications"
                );
                println!();
                println!("USAGE:");
                println!("    {} [OPTIONS]", args[0]);
                println!();
                println!("OPTIONS:");
                println!("    -h, --help       Print this help message");
                println!("    -V, --version    Print version information");
                return;
            }
            "--version" | "-V" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    info!("Starting demonhide daemon...");

    let daemon = match PointerLockDaemon::new() {
        Ok(daemon) => daemon,
        Err(e) => {
            eprintln!("Failed to initialize daemon: {}", e);
            return;
        }
    };

    info!("Daemon initialized successfully");

    let loop_ = MainLoop::new(None, false);

    // Use a cell to allow interior mutability
    use std::cell::RefCell;
    use std::rc::Rc;
    let daemon_rc = Rc::new(RefCell::new(daemon));
    let daemon_clone = daemon_rc.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let mut daemon = daemon_clone.borrow_mut();
        daemon.update();
        glib::Continue(true)
    });

    info!("Starting main loop...");
    loop_.run();
}
