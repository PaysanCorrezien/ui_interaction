use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::collections::HashMap;
use log::debug;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetClassNameW, IsWindowVisible, 
    GetWindowThreadProcessId, GetWindow, GW_OWNER, GetParent
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

use crate::core::{ApplicationManager, ApplicationInfo, Window};
use super::automation::WindowsUIAutomation;

/// Windows-specific application manager
pub struct WindowsApplicationManager {
    automation: WindowsUIAutomation,
}

impl WindowsApplicationManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let automation = WindowsUIAutomation::new()?;
        Ok(Self { automation })
    }

    fn get_process_info(process_id: u32) -> (String, String) {
        let mut process_name = String::new();
        let mut process_path = String::new();
        
        if process_id != 0 {
            unsafe {
                if let Ok(process_handle) = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    process_id
                ) {
                    // Get process path
                    let mut path_buf = [0u16; 512];
                    let path_len = GetModuleFileNameExW(Some(process_handle), None, &mut path_buf);
                    if path_len > 0 {
                        process_path = OsString::from_wide(&path_buf[..path_len as usize])
                            .to_string_lossy()
                            .into_owned();
                        
                        // Extract process name from path
                        if let Some(name) = process_path.split('\\').last() {
                            process_name = name.to_string();
                        }
                    }
                }
            }
        }
        
        (process_name, process_path)
    }

    fn get_window_info(hwnd: HWND) -> Option<(String, String, u32, bool)> {
        unsafe {
            // Skip invalid windows
            if hwnd.is_invalid() {
                return None;
            }

            // Get window title
            let mut title_buf = [0u16; 512];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            let title = if title_len > 0 {
                OsString::from_wide(&title_buf[..title_len as usize])
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };

            // Get window class name
            let mut class_buf = [0u16; 512];
            let class_len = GetClassNameW(hwnd, &mut class_buf);
            let class_name = if class_len > 0 {
                OsString::from_wide(&class_buf[..class_len as usize])
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };

            // Get process ID
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            // Check if window is visible
            let is_visible = IsWindowVisible(hwnd).as_bool();

            // Only consider top-level windows (no parent, no owner)
            let parent = GetParent(hwnd).ok();
            let owner = GetWindow(hwnd, GW_OWNER).ok();
            
            let is_top_level = parent.map_or(true, |h| h.is_invalid()) && 
                              owner.map_or(true, |h| h.is_invalid());

            if is_top_level && is_visible && !title.is_empty() && process_id != 0 {
                Some((title, class_name, process_id, is_visible))
            } else {
                None
            }
        }
    }
}

// Global state for window enumeration
struct EnumWindowsState {
    windows: Vec<(String, String, u32, bool)>,
}

extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let state_ptr = lparam.0 as *mut EnumWindowsState;
        let state = &mut *state_ptr;
        
        if let Some(window_info) = WindowsApplicationManager::get_window_info(hwnd) {
            state.windows.push(window_info);
        }
    }
    BOOL(1) // Continue enumeration
}

impl ApplicationManager for WindowsApplicationManager {
    fn get_all_applications(&self) -> Result<Vec<ApplicationInfo>, Box<dyn Error>> {
        debug!("Enumerating all applications");
        
        let mut state = EnumWindowsState {
            windows: Vec::new(),
        };
        
        unsafe {
            let state_ptr = &mut state as *mut EnumWindowsState;
            if EnumWindows(Some(enum_windows_proc), LPARAM(state_ptr as isize)).is_err() {
                return Err("Failed to enumerate windows".into());
            }
        }

        debug!("Found {} windows", state.windows.len());

        // Group windows by process ID and get process info
        let mut apps_map: HashMap<u32, ApplicationInfo> = HashMap::new();
        
        for (title, class_name, process_id, is_visible) in state.windows {
            if !apps_map.contains_key(&process_id) {
                let (process_name, process_path) = Self::get_process_info(process_id);
                
                if !process_name.is_empty() {
                    apps_map.insert(process_id, ApplicationInfo {
                        process_id,
                        process_name,
                        process_path,
                        main_window_title: title,
                        main_window_class: class_name,
                        is_visible,
                    });
                }
            }
        }

        let apps: Vec<ApplicationInfo> = apps_map.into_values().collect();
        debug!("Found {} unique applications", apps.len());
        
        Ok(apps)
    }

    fn find_applications_by_name(&self, name: &str) -> Result<Vec<ApplicationInfo>, Box<dyn Error>> {
        debug!("Finding applications by name: {}", name);
        let all_apps = self.get_all_applications()?;
        
        let filtered_apps: Vec<ApplicationInfo> = all_apps
            .into_iter()
            .filter(|app| {
                app.process_name.to_lowercase().contains(&name.to_lowercase()) ||
                app.process_name.to_lowercase() == name.to_lowercase()
            })
            .collect();
            
        debug!("Found {} applications matching name '{}'", filtered_apps.len(), name);
        Ok(filtered_apps)
    }

    fn find_applications_by_title(&self, title: &str) -> Result<Vec<ApplicationInfo>, Box<dyn Error>> {
        debug!("Finding applications by title: {}", title);
        let all_apps = self.get_all_applications()?;
        
        let filtered_apps: Vec<ApplicationInfo> = all_apps
            .into_iter()
            .filter(|app| {
                app.main_window_title.to_lowercase().contains(&title.to_lowercase())
            })
            .collect();
            
        debug!("Found {} applications matching title '{}'", filtered_apps.len(), title);
        Ok(filtered_apps)
    }

    fn get_window_by_process_id(&self, process_id: u32) -> Result<Box<dyn Window>, Box<dyn Error>> {
        debug!("Getting window for process ID: {}", process_id);
        
        // Find the HWND for the main window of this process
        struct FindWindowState {
            target_process_id: u32,
            found_hwnd: Option<HWND>,
        }
        
        extern "system" fn find_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let state_ptr = lparam.0 as *mut FindWindowState;
                let state = &mut *state_ptr;
                
                let mut window_process_id = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut window_process_id));
                
                if window_process_id == state.target_process_id {
                    // Check if this is a main window (visible, has title, no parent/owner)
                    let mut title_buf = [0u16; 512];
                    let title_len = GetWindowTextW(hwnd, &mut title_buf);
                    let has_title = title_len > 0;
                    
                    let is_visible = IsWindowVisible(hwnd).as_bool();
                    let parent = GetParent(hwnd).ok();
                    let owner = GetWindow(hwnd, GW_OWNER).ok();
                    
                    let is_top_level = parent.map_or(true, |h| h.is_invalid()) && 
                                      owner.map_or(true, |h| h.is_invalid());
                    
                    if is_visible && has_title && is_top_level {
                        state.found_hwnd = Some(hwnd);
                        return BOOL(0); // Stop enumeration
                    }
                }
            }
            BOOL(1) // Continue enumeration
        }
        
        let mut find_state = FindWindowState {
            target_process_id: process_id,
            found_hwnd: None,
        };
        
        unsafe {
            let state_ptr = &mut find_state as *mut FindWindowState;
            let _ = EnumWindows(Some(find_window_proc), LPARAM(state_ptr as isize));
        }
        
        if let Some(hwnd) = find_state.found_hwnd {
            debug!("Found HWND for process ID {}: {:?}", process_id, hwnd);
            
            // Get UIAutomation element from HWND
            let automation = self.automation.automation.lock()
                .map_err(|e| format!("Failed to lock automation: {}", e))?;
            
            let element = automation.element_from_handle(hwnd.into())
                .map_err(|e| format!("Failed to get element from HWND: {}", e))?;
            
            debug!("Successfully created UIAutomation element from HWND");
            
            // Create WindowsWindow with the specific element
            let window = super::window::WindowsWindow::new(element, std::sync::Arc::new(self.automation.clone()))?;
            
            Ok(Box::new(window))
        } else {
            Err(format!("No main window found for process ID {}", process_id).into())
        }
    }

    fn get_window_by_process_name(&self, name: &str) -> Result<Box<dyn Window>, Box<dyn Error>> {
        debug!("Getting window for process name: {}", name);
        
        let apps = self.find_applications_by_name(name)?;
        if apps.is_empty() {
            return Err(format!("No application found with name '{}'", name).into());
        }

        // Use the first matching application
        let app = &apps[0];
        debug!("Found application: {} (PID: {})", app.process_name, app.process_id);

        // Use the proper get_window_by_process_id method
        self.get_window_by_process_id(app.process_id)
    }
} 