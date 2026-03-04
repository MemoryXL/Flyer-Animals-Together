use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::ImguiRenderLoop;
use hudhook::imgui::{Condition, Ui};
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_RSHIFT, VK_ESCAPE, VK_UP, VK_DOWN};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::Win32::System::Threading::GetCurrentProcessId;

mod memory;
use memory::Memory;

const VERSION: &str = "v1.0.1";
const GITHUB_REPO: &str = "https://github.com/MemoryXL/Flyer-Animals-Together";

// Struct to hold notification data
struct Notification {
    message: String,
    created_at: Instant,
}

// Struct to manage the overlay state
pub struct Overlay {
    notifications: Vec<Notification>,
    show_gui: bool,
    last_toggle_time: Instant,
    
    // Cached values for GUI
    height_val: f32,
    y_velocity_val: f32,
    time_val: f32,
    
    // Configurable values
    y_velocity_up_val: f32,
    y_velocity_down_val: f32,
    time_locked: bool,
    notifications_enabled: bool,
    
    // Key bindings
    key_y_velocity_up: i32,
    key_y_velocity_down: i32,
    
    // Key binding state
    waiting_for_key_up: bool,
    waiting_for_key_down: bool,

    // Key states
    was_up_pressed: bool,
    was_down_pressed: bool,

    // Module base addresses
    unity_player_base: Option<usize>,
    game_assembly_base: Option<usize>,
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            notifications: vec![
                Notification {
                    message: "Overlay injected successfully!".to_string(),
                    created_at: Instant::now(),
                }
            ],
            show_gui: false,
            last_toggle_time: Instant::now(),
            height_val: 0.0,
            y_velocity_val: 0.0,
            time_val: 0.0,
            y_velocity_up_val: 10.0,
            y_velocity_down_val: -8.0,
            time_locked: false,
            notifications_enabled: true,
            key_y_velocity_up: VK_UP.0 as i32,
            key_y_velocity_down: VK_DOWN.0 as i32,
            waiting_for_key_up: false,
            waiting_for_key_down: false,
            was_up_pressed: false,
            was_down_pressed: false,
            unity_player_base: None,
            game_assembly_base: None,
        }
    }
}

impl Overlay {
    fn add_notification(&mut self, message: &str) {
        if !self.notifications_enabled { return; }
        
        // Add new notification
        self.notifications.push(Notification {
            message: message.to_string(),
            created_at: Instant::now(),
        });

        // Limit to 3 notifications
        while self.notifications.len() > 3 {
            self.notifications.remove(0);
        }
    }

    fn update_bases(&mut self) {
        if self.unity_player_base.is_none() {
            unsafe {
                self.unity_player_base = Memory::get_module_base("UnityPlayer.dll");
            }
        }
        if self.game_assembly_base.is_none() {
            unsafe {
                self.game_assembly_base = Memory::get_module_base("GameAssembly.dll");
            }
        }
    }

    unsafe fn read_values(&mut self) {
        self.update_bases();

        // Height
        if let Some(base) = self.unity_player_base {
            // Offsets from XML (reversed): 800, 140, 28, 0, 0, 20, 120
            // Chain: Base -> 120 -> 20 -> 0 -> 0 -> 28 -> 140 -> 800
            let offsets = [0x120, 0x20, 0x0, 0x0, 0x28, 0x140, 0x800];
            if let Some(addr) = Memory::get_pointer_address(base, 0x01A9F230, &offsets) {
                if let Some(val) = Memory::read::<f32>(addr) {
                    self.height_val = val;
                }
            }
        }

        // Y-axis Velocity
        if let Some(base) = self.game_assembly_base {
            // Offsets from XML (reversed): 708, 220, 60, 30, 48, 6A0, 40
            // Chain: Base -> 40 -> 6A0 -> 48 -> 30 -> 60 -> 220 -> 708
            let offsets = [0x40, 0x6A0, 0x48, 0x30, 0x60, 0x220, 0x708];
            if let Some(addr) = Memory::get_pointer_address(base, 0x0254BF18, &offsets) {
                if let Some(val) = Memory::read::<f32>(addr) {
                    self.y_velocity_val = val;
                }
            }
        }

        // Time
        if let Some(base) = self.game_assembly_base {
            // Offsets from XML (reversed): 8BC, 220, 28, 10, 120, 28, B8
            // Chain: Base -> B8 -> 28 -> 120 -> 10 -> 28 -> 220 -> 8BC
            let offsets = [0xB8, 0x28, 0x120, 0x10, 0x28, 0x220, 0x8BC];
            if let Some(addr) = Memory::get_pointer_address(base, 0x023C2D80, &offsets) {
                if let Some(val) = Memory::read::<f32>(addr) {
                    self.time_val = val;
                }
            }
        }
    }

    fn get_key_name(vk: i32) -> String {
        match vk {
            0x26 => "UP".to_string(),
            0x28 => "DOWN".to_string(),
            0x25 => "LEFT".to_string(),
            0x27 => "RIGHT".to_string(),
            0x08 => "BACKSPACE".to_string(),
            0x09 => "TAB".to_string(),
            0x0D => "ENTER".to_string(),
            0x10 => "SHIFT".to_string(),
            0x11 => "CTRL".to_string(),
            0x12 => "ALT".to_string(),
            0x1B => "ESC".to_string(),
            0x20 => "SPACE".to_string(),
            vk if (0x30..=0x39).contains(&vk) => format!("{}", (vk as u8 as char)),
            vk if (0x41..=0x5A).contains(&vk) => format!("{}", (vk as u8 as char)),
            _ => format!("VK_{}", vk),
        }
    }

    unsafe fn write_value(&mut self, id: u8, value: f32, silent: bool) {
        self.update_bases();
        let mut success = false;

        match id {
            0 => { // Height
                if let Some(base) = self.unity_player_base {
                    let offsets = [0x120, 0x20, 0x0, 0x0, 0x28, 0x140, 0x800];
                    if let Some(addr) = Memory::get_pointer_address(base, 0x01A9F230, &offsets) {
                        success = Memory::write::<f32>(addr, value);
                    }
                }
            },
            1 => { // Y-axis Velocity
                if let Some(base) = self.game_assembly_base {
                    let offsets = [0x40, 0x6A0, 0x48, 0x30, 0x60, 0x220, 0x708];
                    if let Some(addr) = Memory::get_pointer_address(base, 0x0254BF18, &offsets) {
                        success = Memory::write::<f32>(addr, value);
                    }
                }
            },
            2 => { // Time
                if let Some(base) = self.game_assembly_base {
                    let offsets = [0xB8, 0x28, 0x120, 0x10, 0x28, 0x220, 0x8BC];
                    if let Some(addr) = Memory::get_pointer_address(base, 0x023C2D80, &offsets) {
                        success = Memory::write::<f32>(addr, value);
                    }
                }
            },
            _ => {}
        }

        if success && !silent {
            self.add_notification(&format!("Updated value to {:.2}", value));
        } else if !success && !silent {
            self.add_notification("Failed to write memory!");
        }
    }
}

impl ImguiRenderLoop for Overlay {
    fn render(&mut self, ui: &mut Ui) {
        unsafe {
            // Check focus
            let current_pid = GetCurrentProcessId();
            let fg_window = GetForegroundWindow();
            let mut fg_pid = 0;
            GetWindowThreadProcessId(fg_window, Some(&mut fg_pid));
            
            if current_pid == fg_pid {
                // Toggle GUI with RShift
                let now = Instant::now();
                if (GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000) != 0 {
                    if now.duration_since(self.last_toggle_time).as_millis() > 300 {
                        self.show_gui = !self.show_gui;
                        self.last_toggle_time = now;
                        
                        // Refresh values when opening
                        if self.show_gui {
                            self.read_values();
                        }
                    }
                }

                // Keyboard hotkeys for Y-axis Velocity
                // Check if waiting for binding
                if self.waiting_for_key_up {
                    for vk in 1..255 {
                         if (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 {
                             // Avoid binding modifier keys alone if possible, or RShift/ESC/LMB
                             if vk != VK_RSHIFT.0 as i32 && vk != VK_ESCAPE.0 as i32 && vk != 0x01 {
                                 self.key_y_velocity_up = vk;
                                 self.waiting_for_key_up = false;
                                 self.add_notification(&format!("Bound UP action to {}", Self::get_key_name(vk)));
                                 break;
                             }
                         }
                    }
                } else if self.waiting_for_key_down {
                    for vk in 1..255 {
                         if (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 {
                             if vk != VK_RSHIFT.0 as i32 && vk != VK_ESCAPE.0 as i32 && vk != 0x01 {
                                 self.key_y_velocity_down = vk;
                                 self.waiting_for_key_down = false;
                                 self.add_notification(&format!("Bound DOWN action to {}", Self::get_key_name(vk)));
                                 break;
                             }
                         }
                    }
                } else {
                    // Normal operation
                    let up_pressed = (GetAsyncKeyState(self.key_y_velocity_up) as u16 & 0x8000) != 0;
                    if up_pressed {
                        let silent = self.was_up_pressed;
                        self.write_value(1, self.y_velocity_up_val, silent);
                    }
                    self.was_up_pressed = up_pressed;

                    let down_pressed = (GetAsyncKeyState(self.key_y_velocity_down) as u16 & 0x8000) != 0;
                    if down_pressed {
                        let silent = self.was_down_pressed;
                        self.write_value(1, self.y_velocity_down_val, silent);
                    }
                    self.was_down_pressed = down_pressed;
                }
            }
            
            // Time locking (always active if enabled, regardless of focus?)
            // Usually trainers work even in background, but prompt implies user interaction.
            // Let's keep it active.
            if self.time_locked {
                self.write_value(2, self.time_val, true);
            }
        }

        let [width, height] = ui.io().display_size;

        // Render Notifications (Bottom Right)
        // Filter out old notifications (older than 3s)
        self.notifications.retain(|n| n.created_at.elapsed() < Duration::from_secs(3));

        let notif_width = 300.0;
        let notif_height = 60.0;
        let spacing = 10.0;
        
        // Draw notifications from bottom up (newest at bottom)
        let mut current_y = height - 20.0;

        // Iterate in reverse to draw from bottom up
        for (i, notif) in self.notifications.iter().rev().enumerate() {
            current_y -= notif_height;
            if i > 0 {
                current_y -= spacing;
            }

            ui.window(&format!("Notification##{}", i))
                .position(
                    [width - notif_width - 20.0, current_y],
                    Condition::Always
                )
                .size([notif_width, notif_height], Condition::Always)
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(|| {
                    ui.text(&notif.message);
                    let elapsed = notif.created_at.elapsed().as_secs_f32();
                    ui.text(&format!("Closing in {:.1}s", 3.0 - elapsed));
                });
        }

        // Render Cheat GUI
        if self.show_gui {
            ui.window(&format!("FAT (Flyer Animals Together) {}", VERSION))
                .size([400.0, 480.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Press RShift to close");
                    ui.checkbox("Enable Notifications", &mut self.notifications_enabled);
                    ui.separator();

                    ui.text("Hotkeys Configuration:");
                    
                    // UP Key Binding
                    let up_label = if self.waiting_for_key_up { "Press any key..." } else { "Bind" };
                    if ui.button(&format!("{}##Up", up_label)) {
                        self.waiting_for_key_up = true;
                        self.waiting_for_key_down = false; // Cancel other
                    }
                    ui.same_line();
                    ui.text(&format!("UP Action Key: {}", Self::get_key_name(self.key_y_velocity_up)));
                    
                    ui.input_float("UP Action Value", &mut self.y_velocity_up_val).build();
                    
                    // DOWN Key Binding
                    let down_label = if self.waiting_for_key_down { "Press any key..." } else { "Bind" };
                    if ui.button(&format!("{}##Down", down_label)) {
                        self.waiting_for_key_down = true;
                        self.waiting_for_key_up = false; // Cancel other
                    }
                    ui.same_line();
                    ui.text(&format!("DOWN Action Key: {}", Self::get_key_name(self.key_y_velocity_down)));
                    
                    ui.input_float("DOWN Action Value", &mut self.y_velocity_down_val).build();
                    
                    ui.separator();

                    // Time (Editable with Lock)
                    let mut t = self.time_val;
                    // Note: ui.input_float returns true ONLY when value changed and enter pressed/focus lost
                    // BUT it updates the mutable reference every frame if user is typing.
                    // To prevent auto-set, we only write when `build()` returns true.
                    if ui.input_float("Time (seconds)", &mut t).build() {
                        self.time_val = t;
                        unsafe { self.write_value(2, t, false); }
                    }
                    
                    ui.same_line();
                    if ui.checkbox("Lock Time", &mut self.time_locked) {
                        if self.time_locked {
                            // When locking, ensure we lock current value
                             self.time_val = t;
                             self.add_notification("Time Locked");
                        }
                    }
                    
                    ui.same_line();
                    if ui.button("Set##T") {
                         self.time_val = t;
                         unsafe { self.write_value(2, self.time_val, false); }
                    }
                    
                    ui.separator();
                    ui.text(GITHUB_REPO);
                });
        }
    }
}

hudhook::hudhook!(ImguiDx11Hooks, Overlay::default());
