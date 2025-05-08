use log::{debug, info, warn};
use uiautomation::controls::ControlType;
use uiautomation::core::UIElement;
use uiautomation::types::UIProperty;
use uiautomation::variants::Variant;
use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HWND, RECT, HANDLE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetClassNameW, GetWindowLongPtrW, GWL_EXSTYLE,
    GWL_STYLE, IsWindowVisible, GetWindowThreadProcessId, GetWindowRect, GetParent,
    GetWindow, GW_OWNER, GetMenu, GetWindowPlacement, WINDOWPLACEMENT, SW_SHOWMINIMIZED,
    SW_SHOWMAXIMIZED, SW_SHOWNORMAL
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use std::ptr::null_mut;

use crate::automation::UIAutomation;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    // Basic window properties
    pub title: String,
    pub class_name: String,
    pub hwnd: HWND,
    pub parent_hwnd: Option<HWND>,
    pub owner_hwnd: Option<HWND>,
    
    // Window state
    pub is_visible: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub is_restored: bool,
    
    // Window geometry
    pub rect: RECT,
    pub dpi: u32,
    
    // Window styles
    pub style: isize,
    pub ex_style: isize,
    pub has_menu: bool,
    
    // Process information
    pub process_id: u32,
    pub thread_id: u32,
    pub process_name: String,
    pub process_path: String,
}

impl WindowInfo {
    pub fn get_current() -> Option<Self> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                debug!("GetForegroundWindow returned NULL handle");
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
                debug!("GetWindowTextW returned 0 length");
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
                debug!("GetClassNameW returned 0 length");
                String::new()
            };

            // Get window geometry
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect);
            let dpi = GetDpiForWindow(hwnd);

            // Get window state
            let mut placement = WINDOWPLACEMENT::default();
            placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
            GetWindowPlacement(hwnd, &mut placement);
            let is_minimized = placement.showCmd as u32 == SW_SHOWMINIMIZED.0 as u32;
            let is_maximized = placement.showCmd as u32 == SW_SHOWMAXIMIZED.0 as u32;
            let is_restored = placement.showCmd as u32 == SW_SHOWNORMAL.0 as u32;

            // Get window styles
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let is_visible = IsWindowVisible(hwnd).as_bool();
            let has_menu = !GetMenu(hwnd).is_invalid();

            // Get parent and owner windows
            let parent_hwnd = GetParent(hwnd).ok();
            let owner_hwnd = GetWindow(hwnd, GW_OWNER).ok();

            // Get process and thread IDs
            let mut process_id = 0u32;
            let thread_id = GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            // Get process name and path
            let mut process_name = String::new();
            let mut process_path = String::new();
            
            if process_id != 0 {
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

            debug!(
                "Window info - Title: '{}', Class: '{}', Visible: {}, Minimized: {}, Maximized: {}, Restored: {}, \
                Position: ({}, {}), Size: {}x{}, DPI: {}, \
                Style: {:x}, ExStyle: {:x}, HasMenu: {}, \
                ProcessID: {}, ThreadID: {}, ProcessName: '{}', ProcessPath: '{}'",
                title, class_name, is_visible, is_minimized, is_maximized, is_restored,
                rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, dpi,
                style, ex_style, has_menu,
                process_id, thread_id, process_name, process_path
            );

            Some(Self { 
                title,
                class_name,
                hwnd,
                parent_hwnd,
                owner_hwnd,
                is_visible,
                is_minimized,
                is_maximized,
                is_restored,
                rect,
                dpi,
                style,
                ex_style,
                has_menu,
                process_id,
                thread_id,
                process_name,
                process_path,
            })
        }
    }
}

/// Window management functionality
pub struct WindowManager<'a> {
    automation: &'a UIAutomation,
}

impl<'a> WindowManager<'a> {
    /// Create a new WindowManager instance
    pub fn new(automation: &'a UIAutomation) -> Self {
        WindowManager { automation }
    }

    /// Get the currently focused window and its information
    pub fn get_focused_window(
        &self,
    ) -> Result<(UIElement, String, String, String), Box<dyn Error>> {
        debug!("Getting currently focused window");
        
        // Get window info using Windows API
        let window_info = WindowInfo::get_current()
            .ok_or_else(|| "Failed to get window info using Windows API")?;
        
        debug!("Successfully got window info using Windows API");
        
        // Create a dummy UIElement since we don't have one from the Windows API
        let dummy_element = self.automation.core().get_root_element()?;
        Ok((
            dummy_element,
            window_info.title,
            "Window".to_string(),
            window_info.class_name,
        ))
    }
}
