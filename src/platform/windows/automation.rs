use std::error::Error;
use std::sync::{Arc, Mutex};
use log::{debug, info, warn};
use uiautomation::UIElement as UIAutomationElement;
use uiautomation::types::{TreeScope, UIProperty};
use uiautomation::variants::Variant;
use uiautomation::controls::ControlType;
use uiautomation::patterns::{UIValuePattern, UITextPattern};
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

use crate::core::{Window, UIAutomation as CoreUIAutomation, UIElement as CoreUIElement};
use super::window::WindowsWindow;
use super::element::WindowsElement;

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

// Thread-safe wrapper for our non-thread-safe types
#[derive(Clone)]
pub struct ThreadSafe<T>(Arc<Mutex<T>>);

impl<T> ThreadSafe<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<T>, Box<dyn Error>> {
        self.0.lock().map_err(|_| "Failed to lock automation".into())
    }
}

// Make ThreadSafe Send + Sync
unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}

thread_local! {
    pub static AUTOMATION: std::cell::RefCell<Option<ThreadSafe<uiautomation::core::UIAutomation>>> = std::cell::RefCell::new(None);
}

/// Windows-specific UIAutomation implementation
#[derive(Clone)]
pub struct WindowsUIAutomation {
    pub automation: ThreadSafe<uiautomation::core::UIAutomation>,
}

impl WindowsUIAutomation {
    /// Create a new instance of WindowsUIAutomation
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let automation = uiautomation::UIAutomation::new()?;
        Ok(WindowsUIAutomation {
            automation: ThreadSafe::new(automation),
        })
    }

    /// Convert a UIAutomationElement to a Box<dyn UIElement>
    pub fn element_to_ui_element(&self, element: UIAutomationElement) -> Box<dyn CoreUIElement> {
        Box::new(WindowsElement::new(element, None)) as Box<dyn CoreUIElement>
    }

    /// Get the currently focused window
    pub fn get_focused_window(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Getting focused window");
        let window_info = WindowInfo::get_current()
            .ok_or_else(|| "Failed to get window info using Windows API")?;
        
        info!("Focused window - Title: {}, ClassName: {}", 
            window_info.title, window_info.class_name);
        
        let automation = self.automation.lock()?;
        automation.get_focused_element().map_err(|e| e.into())
    }

    /// Get the currently focused element with its details
    pub fn get_focused_element_details(&self) -> Result<(UIAutomationElement, String, String), Box<dyn Error>> {
        debug!("Getting focused element details");
        let automation = self.automation.lock()?;
        let element = automation.get_focused_element()?;
        let name = element.get_name()?;
        let class_name = element.get_classname()?;
        Ok((element, name, class_name))
    }

    /// Get the currently focused input element
    pub fn get_focused_input_element(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Getting currently focused input element");
        let automation = self.automation.lock()?;
        let focused_element = automation.get_focused_element()?;
        
        let control_type: Variant = focused_element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        let is_input = match ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom) {
            ControlType::Edit => true,
            ControlType::ComboBox => true,
            ControlType::CheckBox => true,
            ControlType::RadioButton => true,
            ControlType::Slider => true,
            ControlType::Text => true,
            ControlType::Document => true,
            ControlType::Pane => {
                focused_element.get_pattern::<UIValuePattern>().is_ok() ||
                focused_element.get_pattern::<UITextPattern>().is_ok()
            },
            ControlType::Custom => {
                focused_element.get_pattern::<UIValuePattern>().is_ok()
            },
            _ => false,
        };
        
        if is_input {
            info!("Found focused input element with type: {:?}", ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom));
            Ok(focused_element)
        } else {
            warn!("Focused element is not an input control (Type: {:?})", ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom));
            Err("Focused element is not an input control".into())
        }
    }

    /// Find an element by its name
    pub fn find_element_by_name(&self, name: &str) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Finding element by name: {}", name);
        let automation = self.automation.lock()?;
        let root = automation.get_root_element()?;
        let condition = automation.create_property_condition(UIProperty::Name, Variant::from(name), None)?;
        root.find_first(TreeScope::Descendants, &condition)
            .map_err(|e| e.into())
    }

    /// Find an element by its type
    pub fn find_element_by_type(&self, element_type: &str) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Finding element by type: {}", element_type);
        let automation = self.automation.lock()?;
        let root = automation.get_root_element()?;
        let condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(element_type), None)?;
        root.find_first(TreeScope::Descendants, &condition)
            .map_err(|e| e.into())
    }
}

impl CoreUIAutomation for WindowsUIAutomation {
    fn get_focused_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>> {
        let element = self.get_focused_window()?;
        Ok(Box::new(WindowsWindow::new(element, Arc::new(self.clone()))?))
    }

    fn get_focused_element(&self) -> Result<Box<dyn CoreUIElement>, Box<dyn Error>> {
        let element = self.automation.lock()?.get_focused_element()?;
        Ok(self.element_to_ui_element(element))
    }

    fn find_element_by_name(&self, name: &str) -> Result<Box<dyn CoreUIElement>, Box<dyn Error>> {
        let automation = self.automation.lock()?;
        let condition = automation.create_property_condition(UIProperty::Name, Variant::from(name), None)?;
        let element = automation.get_root_element()?.find_first(TreeScope::Descendants, &condition)?;
        Ok(self.element_to_ui_element(element))
    }

    fn find_element_by_type(&self, element_type: &str) -> Result<Box<dyn CoreUIElement>, Box<dyn Error>> {
        let automation = self.automation.lock()?;
        let condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(element_type), None)?;
        let element = automation.get_root_element()?.find_first(TreeScope::Descendants, &condition)?;
        Ok(self.element_to_ui_element(element))
    }
} 