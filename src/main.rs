use glib::MainLoop;
use sysinfo::{System, RefreshKind, ProcessRefreshKind};
use std::os::raw::c_void;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_client::protocol::{wl_registry, wl_seat, wl_pointer, wl_surface, wl_compositor, wl_shell, wl_shell_surface};
use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_pointer_constraints_v1, zwp_locked_pointer_v1
};

// We'll keep the C extern declarations for now but won't use them
extern "C" {
    fn init_pointer_constraints(display: *mut c_void);
    fn lock_pointer(surface: *mut c_void, pointer: *mut c_void);
}

struct AppData {
    pointer_constraints: Option<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1>,
    seat: Option<wl_seat::WlSeat>,
    pointer: Option<wl_pointer::WlPointer>,
    compositor: Option<wl_compositor::WlCompositor>,
    surface: Option<wl_surface::WlSurface>,
    shell: Option<wl_shell::WlShell>,
    shell_surface: Option<wl_shell_surface::WlShellSurface>,
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
        if let wl_registry::Event::Global { name, interface, .. } = event {
            match &interface[..] {
                "zwp_pointer_constraints_v1" => {
                    let pointer_constraints = registry.bind::<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, _, _>(
                        name, 1, qh, ()
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
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
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
    ) {}
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {}
}

impl Dispatch<wl_shell::WlShell, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shell::WlShell,
        _: wl_shell::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {}
}

impl Dispatch<wl_shell_surface::WlShellSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shell_surface::WlShellSurface,
        event: wl_shell_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        match event {
            wl_shell_surface::Event::Configure { .. } => {
                println!("Shell surface configured");
            }
            _ => {}
        }
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
    ) {}
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
            
            println!("Capabilities value: {}, Pointer bit: {}", caps_value, pointer_bit);
            
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
    ) {}
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
    cursor_hidden: bool,
    conn: Option<Connection>,
    app_data: Option<AppData>,
    event_queue: Option<wayland_client::EventQueue<AppData>>,
    is_locked: bool,  // Track current lock state
    last_game_state: bool,  // Track previous game detection state
    game_state_stable_count: u32,  // Count stable state cycles for debouncing
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
                    shell_surface: None,
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
                    cursor_hidden: true,
                    conn: Some(conn),
                    app_data: Some(app_data),
                    event_queue: Some(event_queue),
                    is_locked: false,
                    last_game_state: false,
                    game_state_stable_count: 0,
                })
            },
            Err(e) => {
                println!("Failed to connect to Wayland: {}, running without pointer constraints", e);
                Ok(PointerLockDaemon { 
                    cursor_hidden: true,
                    conn: None,
                    app_data: None,
                    event_queue: None,
                    is_locked: false,
                    last_game_state: false,
                    game_state_stable_count: 0,
                })
            }
        }
    }
    
    fn is_cursor_hidden(&self) -> bool {
        self.cursor_hidden
    }

    fn is_game_running(&self) -> bool {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything())
        );
        
        // Debug: Show some potentially interesting processes (only in debug builds)
        #[cfg(debug_assertions)]
        {
            static mut DEBUG_COUNTER: u32 = 0;
            unsafe {
                DEBUG_COUNTER += 1;
                if DEBUG_COUNTER % 10 == 0 {  // Every 5 seconds
                    let interesting_processes: Vec<_> = sys.processes().values()
                        .filter(|p| {
                            let name = p.name().to_string_lossy();
                            name.contains("wine") || name.contains("steam") || name.contains("proton") || 
                            name.contains("lutris") || name.contains(".exe")
                        })
                        .map(|p| format!("{}", p.name().to_string_lossy()))
                        .collect();
                    if !interesting_processes.is_empty() {
                        println!("🔍 Interesting processes: {:?}", interesting_processes);
                    }
                }
            }
        }
        
        let game_processes: Vec<_> = sys.processes().values()
            .filter(|p| {
                let name = p.name().to_string_lossy();
                let cmd_string = p.cmd().iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ");
                
                // Exclude Steam helper processes first
                if name.contains("steamwebhelper") || 
                   name.contains("steam.exe") ||
                   name.contains("steamcmd") ||
                   name.contains("SteamChildMonitor") ||
                   name.contains("GameOverlayUI") {
                    return false;
                }
                
                // More specific game detection
                let is_game = 
                    // Wine games (but not wine itself)
                    (name.contains("wine64") && cmd_string.contains(".exe")) ||
                    // Proton games
                    (name.contains("proton") && cmd_string.contains(".exe")) ||
                    // Games in steamapps directory
                    cmd_string.contains("steamapps/common") ||
                    // Lutris games
                    (name.contains("lutris") && cmd_string.contains(".exe")) ||
                    // Direct .exe execution (but exclude system tools)
                    (cmd_string.contains(".exe") && 
                     !cmd_string.contains("steam") && 
                     !cmd_string.contains("helper") &&
                     !cmd_string.contains("launcher"));
                
                is_game
            })
            .map(|p| format!("{}[{}]", p.name().to_string_lossy(), p.pid()))
            .collect();
            
        if !game_processes.is_empty() {
            #[cfg(debug_assertions)]
            println!("🎮 Found game processes: {:?}", game_processes);
            true
        } else {
            false
        }
    }

    fn lock_pointer(&mut self) {
        // Prevent multiple lock attempts
        if self.is_locked {
            return;
        }
        
        if let (Some(app_data), Some(event_queue)) = (&mut self.app_data, &mut self.event_queue) {
            #[cfg(debug_assertions)]
            {
                println!("Debug: compositor available: {}", app_data.compositor.is_some());
                println!("Debug: surface available: {}", app_data.surface.is_some());
                println!("Debug: pointer_constraints available: {}", app_data.pointer_constraints.is_some());
                println!("Debug: pointer available: {}", app_data.pointer.is_some());
            }
            
            if let (Some(pointer_constraints), Some(pointer), Some(surface)) = 
                (&app_data.pointer_constraints, &app_data.pointer, &app_data.surface) {
                    
                // Check if we already have a locked pointer object
                if app_data.locked_pointer.is_some() {
                    #[cfg(debug_assertions)]
                    println!("Pointer lock object already exists");
                    return;
                }
                
                println!("🎯 Locking pointer to surface...");
                
                // Lock the pointer to our surface
                let locked_pointer = pointer_constraints.lock_pointer(
                    surface,
                    pointer,
                    None, // No region restriction
                    zwp_pointer_constraints_v1::Lifetime::Persistent,
                    &event_queue.handle(),
                    ()
                );
                
                // Store the locked pointer
                app_data.locked_pointer = Some(locked_pointer);
                self.is_locked = true;
                
                // Process events to handle the lock response with timeout
                match event_queue.dispatch_pending(app_data) {
                    Ok(_) => {
                        println!("✅ Pointer lock request sent successfully");
                    }
                    Err(e) => {
                        println!("❌ Error processing pointer lock events: {}", e);
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
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            println!("❌ Error processing pointer unlock events: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    fn update_pointer_state(&mut self) {
        let game_running = self.is_game_running();
        
        // Implement debouncing - require 3 stable readings before changing state
        const STABLE_CYCLES_REQUIRED: u32 = 3;
        
        if game_running == self.last_game_state {
            self.game_state_stable_count += 1;
        } else {
            // State changed, reset counter
            self.game_state_stable_count = 0;
            self.last_game_state = game_running;
        }
        
        // Add debug output every few cycles (only in debug builds)
        #[cfg(debug_assertions)]
        {
            static mut DEBUG_COUNTER: u32 = 0;
            unsafe {
                DEBUG_COUNTER += 1;
                if DEBUG_COUNTER % 6 == 0 {  // Print every 3 seconds
                    println!("🔍 Status: Game: {}, Locked: {}, Stable: {}/{}", 
                        game_running, self.is_locked, self.game_state_stable_count, STABLE_CYCLES_REQUIRED);
                }
            }
        }
        
        // Only act when state has been stable for required cycles
        if self.game_state_stable_count >= STABLE_CYCLES_REQUIRED {
            if game_running && !self.is_locked {
                println!("🎮 Game confirmed running - locking pointer");
                self.lock_pointer();
            } else if !game_running && self.is_locked {
                println!("🎮 Game confirmed stopped - unlocking pointer");
                self.unlock_pointer();
            }
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
        
        // Update pointer state based on game detection
        if daemon.is_cursor_hidden() {
            daemon.update_pointer_state();
        } else if daemon.is_locked {
            // Unlock if cursor should not be hidden
            daemon.unlock_pointer();
        }
        
        glib::Continue(true)
    });

    println!("Starting main loop...");
    loop_.run();
}
