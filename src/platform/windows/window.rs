use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::types::UIProperty;
use uiautomation::variants::Variant;
use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetClassNameW, GetWindowLongPtrW, GWL_EXSTYLE,
    GWL_STYLE, IsWindowVisible, GetWindowThreadProcessId, GetWindowRect, GetParent,
    GetWindow, GW_OWNER, GetMenu, GetWindowPlacement, WINDOWPLACEMENT, SW_SHOWMINIMIZED,
    SW_SHOWMAXIMIZED, SW_SHOWNORMAL
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use log::debug;

use crate::core::{Window, UIElement, UIAutomation};

/// Windows-specific window implementation
pub struct WindowsWindow {
    element: UIAutomationElement,
    automation: Box<dyn UIAutomation>,
    window_info: Option<WindowInfo>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct WindowInfo {
    title: String,
    class_name: String,
    hwnd: HWND,
    parent_hwnd: Option<HWND>,
    owner_hwnd: Option<HWND>,
    is_visible: bool,
    is_minimized: bool,
    is_maximized: bool,
    is_restored: bool,
    rect: RECT,
    dpi: u32,
    style: isize,
    ex_style: isize,
    has_menu: bool,
    process_id: u32,
    thread_id: u32,
    process_name: String,
    process_path: String,
}

impl WindowInfo {
    fn get_current() -> Option<Self> {
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
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return None;
            }
            let dpi = GetDpiForWindow(hwnd);

            // Get window state
            let mut placement = WINDOWPLACEMENT::default();
            placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
            if GetWindowPlacement(hwnd, &mut placement).is_err() {
                return None;
            }
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
            let parent_hwnd = parent_hwnd.filter(|h| !h.is_invalid());
            let owner_hwnd = owner_hwnd.filter(|h| !h.is_invalid());

            // Get process and thread IDs
            let mut process_id = 0u32;
            let thread_id = GetWindowThreadProcessId(hwnd.into(), Some(&mut process_id));

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

impl WindowsWindow {
    pub fn new(element: UIAutomationElement, automation: Box<dyn UIAutomation>) -> Result<Self, Box<dyn Error>> {
        Ok(WindowsWindow { 
            element, 
            automation,
            window_info: WindowInfo::get_current(),
        })
    }
}

impl Window for WindowsWindow {
    fn get_title(&self) -> Result<String, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.title.clone())
        } else {
            let name: Variant = self.element.get_property_value(UIProperty::Name)?;
            Ok(name.get_string()?)
        }
    }

    fn get_class_name(&self) -> Result<String, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.class_name.clone())
        } else {
            Ok(self.element.get_classname()?)
        }
    }

    fn get_process_id(&self) -> Result<u32, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.process_id)
        } else {
            Ok(self.element.get_process_id()? as u32)
        }
    }

    fn get_thread_id(&self) -> Result<u32, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.thread_id)
        } else {
            let mut process_id = 0u32;
            let thread_id = unsafe { GetWindowThreadProcessId(self.element.get_native_window_handle()?.into(), Some(&mut process_id)) };
            Ok(thread_id)
        }
    }

    fn get_process_name(&self) -> Result<String, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.process_name.clone())
        } else {
            let process_id = self.get_process_id()?;
            let _hwnd = self.element.get_native_window_handle()?;
            
            if process_id != 0 {
                if let Ok(process_handle) = unsafe { OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    process_id
                )} {
                    let mut path_buf = [0u16; 512];
                    let path_len = unsafe { GetModuleFileNameExW(Some(process_handle), None, &mut path_buf) };
                    if path_len > 0 {
                        let path = OsString::from_wide(&path_buf[..path_len as usize])
                            .to_string_lossy()
                            .into_owned();
                        
                        // Extract process name from path
                        if let Some(name) = path.split('\\').last() {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
            Ok(String::new())
        }
    }

    fn get_process_path(&self) -> Result<String, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.process_path.clone())
        } else {
            let process_id = self.get_process_id()?;
            
            if process_id != 0 {
                if let Ok(process_handle) = unsafe { OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    process_id
                )} {
                    let mut path_buf = [0u16; 512];
                    let path_len = unsafe { GetModuleFileNameExW(Some(process_handle), None, &mut path_buf) };
                    if path_len > 0 {
                        return Ok(OsString::from_wide(&path_buf[..path_len as usize])
                            .to_string_lossy()
                            .into_owned());
                    }
                }
            }
            Ok(String::new())
        }
    }

    fn is_visible(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.is_visible)
        } else {
            Ok(unsafe { IsWindowVisible(self.element.get_native_window_handle()?.into()).as_bool() })
        }
    }

    fn is_minimized(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.is_minimized)
        } else {
            let mut placement = WINDOWPLACEMENT::default();
            placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
            unsafe { GetWindowPlacement(self.element.get_native_window_handle()?.into(), &mut placement) }?;
            Ok(placement.showCmd as u32 == SW_SHOWMINIMIZED.0 as u32)
        }
    }

    fn is_maximized(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.is_maximized)
        } else {
            let hwnd = self.element.get_native_window_handle()?;
            let mut placement = WINDOWPLACEMENT::default();
            placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
            unsafe { GetWindowPlacement(hwnd.into(), &mut placement) }?;
            Ok(placement.showCmd as u32 == SW_SHOWMAXIMIZED.0 as u32)
        }
    }

    fn get_rect(&self) -> Result<RECT, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.rect)
        } else {
            let mut rect = RECT::default();
            unsafe { GetWindowRect(self.element.get_native_window_handle()?.into(), &mut rect) }?;
            Ok(rect)
        }
    }

    fn get_dpi(&self) -> Result<u32, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.dpi)
        } else {
            let hwnd = self.element.get_native_window_handle()?;
            Ok(unsafe { GetDpiForWindow(hwnd.into()) })
        }
    }

    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>> {
        self.automation.get_focused_element()
    }
} 