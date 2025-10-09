use glib::MainLoop;
use std::ptr;
use wayland_client::protocol::{
    wl_compositor, wl_pointer, wl_registry, wl_seat, wl_shell, wl_shell_surface, wl_surface,
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
                    println!("Bound pointer constraints interface");
                    state.pointer_constraints = Some(pointer_constraints);
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                    println!("Bound seat interface - requesting capabilities...");
                    state.seat = Some(seat);
                }
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
                    println!("Bound compositor interface");
                    state.compositor = Some(compositor);
                }
                "wl_shell" => {
                    let shell = registry.bind::<wl_shell::WlShell, _, _>(name, 1, qh, ());
                    println!("Bound shell interface");
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
            println!("Seat capabilities received: {:?}", capabilities);

            // Try different ways to check for pointer capability
            let caps_value: u32 = capabilities.into();
            let pointer_bit = u32::from(wl_seat::Capability::Pointer);

            println!(
                "Capabilities value: {}, Pointer bit: {}",
                caps_value, pointer_bit
            );

            if (caps_value & pointer_bit) != 0 {
                let pointer = seat.get_pointer(qh, ());
                println!("Got pointer capability and created pointer device");
                state.pointer = Some(pointer);
            } else {
                println!("No pointer capability available");
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
                println!("🔒 Pointer successfully locked!");
            }
            zwp_locked_pointer_v1::Event::Unlocked => {
                println!("🔓 Pointer unlocked");
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
}

impl PointerLockDaemon {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to connect to Wayland
        match Connection::connect_to_env() {
            Ok(conn) => {
                println!("Connected to Wayland display");

                let mut app_data = AppData {
                    pointer_constraints: None,
                    seat: None,
                    pointer: None,
                    compositor: None,
                    surface: None,
                    shell: None,
                    locked_pointer: None,
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
                    println!("Doing second roundtrip to get seat capabilities...");
                    event_queue.blocking_dispatch(&mut app_data)?;
                }

                // Create surface even without shell - just a basic surface for pointer locking
                if let Some(compositor) = &app_data.compositor {
                    let surface = compositor.create_surface(&qh, ());
                    println!("Created basic surface for pointer locking");

                    // Commit the surface to make it "live"
                    surface.commit();

                    app_data.surface = Some(surface);

                    println!("Surface committed and ready for pointer locking");
                } else {
                    println!("Warning: Missing compositor, cannot create surface");
                }

                println!("Wayland protocols initialized successfully");

                Ok(PointerLockDaemon {
                    app_data: Some(app_data),
                    event_queue: Some(event_queue),
                    is_locked: false,
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
                })
            }
        }
    }

    fn should_lock(&self) -> bool {
        should_lock_pointer()
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

                println!(
                    "🔒 Locking pointer for XWayland fullscreen application with hidden cursor"
                );

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
                        println!("✅ Pointer lock request sent successfully");
                    }
                    Err(_e) => {
                        println!("❌ Error processing pointer lock events: {}", _e);
                        self.is_locked = false; // Reset on error
                    }
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    if app_data.pointer_constraints.is_none() {
                        println!("❌ Pointer constraints protocol not available");
                    }
                    if app_data.pointer.is_none() {
                        println!("❌ Pointer device not available");
                    }
                    if app_data.surface.is_none() {
                        println!("❌ Surface not available");
                    }
                    if app_data.compositor.is_none() {
                        println!("❌ Compositor not available");
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
                println!("🔓 Unlocking pointer...");
                locked_pointer.destroy();
                self.is_locked = false;

                // Process events to handle the unlock (non-blocking)
                if let Some(event_queue) = &mut self.event_queue {
                    match event_queue.dispatch_pending(app_data) {
                        Ok(_) => {
                            println!("✅ Pointer unlock processed");
                        }
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            println!("❌ Error processing pointer unlock events: {}", _e);
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
    println!("Starting demonhide daemon...");

    let daemon = match PointerLockDaemon::new() {
        Ok(daemon) => daemon,
        Err(e) => {
            eprintln!("Failed to initialize daemon: {}", e);
            return;
        }
    };

    println!("Daemon initialized successfully");

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

    println!("Starting main loop...");
    loop_.run();
}
