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
        
        // Get the focused element first
        let focused_element = automation.get_focused_element()?;
        debug!("Got focused element: {}", focused_element.get_name().unwrap_or_default());
        
        // Create a tree walker to navigate up to the window
        let walker = automation.create_tree_walker()?;
        
        // Walk up the tree to find the Window element that contains this focused element
        let mut current_element = focused_element;
        loop {
            // Check if current element is a Window
            if let Ok(control_type_variant) = current_element.get_property_value(UIProperty::ControlType) {
                if let Ok(control_type_id) = <Variant as TryInto<i32>>::try_into(control_type_variant) {
                    if control_type_id == ControlType::Window as i32 {
                        debug!("Found window element: {}", current_element.get_name().unwrap_or_default());
                        return Ok(current_element);
                    }
                }
            }
            
            // Try to get parent element using walker
            match walker.get_parent(&current_element) {
                Ok(parent) => {
                    debug!("Moving to parent: {}", parent.get_name().unwrap_or_default());
                    current_element = parent;
                },
                Err(e) => {
                    warn!("Could not find Window element by walking up tree: {}, using focused element as fallback", e);
                    return Ok(automation.get_focused_element()?);
                }
            }
        }
    }

    /// Get the currently active (foreground) window - the top-level application window
    pub fn get_active_window(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Getting active window");
        let window_info = WindowInfo::get_current()
            .ok_or_else(|| "Failed to get window info using Windows API")?;
        
        info!("Active window - Title: {}, ClassName: {}", 
            window_info.title, window_info.class_name);
        
        let automation = self.automation.lock()?;
        
        // Get the focused element first
        let focused_element = automation.get_focused_element()?;
        debug!("Got focused element: {}", focused_element.get_name().unwrap_or_default());
        
        // Create a tree walker to navigate up to the top-level window
        let walker = automation.create_tree_walker()?;
        
        // Walk up the tree to find the top-level Window element
        let mut current_element = focused_element;
        let mut last_window = None;
        
        loop {
            // Check if current element is a Window
            if let Ok(control_type_variant) = current_element.get_property_value(UIProperty::ControlType) {
                if let Ok(control_type_id) = <Variant as TryInto<i32>>::try_into(control_type_variant) {
                    if control_type_id == ControlType::Window as i32 {
                        debug!("Found window element: {}", current_element.get_name().unwrap_or_default());
                        last_window = Some(current_element.clone());
                    }
                }
            }
            
            // Try to get parent element using walker
            match walker.get_parent(&current_element) {
                Ok(parent) => {
                    debug!("Moving to parent: {}", parent.get_name().unwrap_or_default());
                    current_element = parent;
                },
                Err(_) => {
                    // Reached the top of the tree
                    break;
                }
            }
        }
        
        // Return the top-most window we found, or fall back to the focused element
        match last_window {
            Some(window) => {
                debug!("Returning top-level window: {}", window.get_name().unwrap_or_default());
                Ok(window)
            },
            None => {
                warn!("Could not find top-level Window element, using focused element as fallback");
                Ok(automation.get_focused_element()?)
            }
        }
    }

    /// Get the window that contains the currently focused element
    pub fn get_window_containing_focus(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        // This is the same as the old get_focused_window behavior
        self.get_focused_window()
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
        
        // Convert string type to ControlType enum
        let control_type = match element_type {
            "Button" => ControlType::Button,
            "Edit" => ControlType::Edit,
            "Text" => ControlType::Text,
            "ComboBox" => ControlType::ComboBox,
            "CheckBox" => ControlType::CheckBox,
            "RadioButton" => ControlType::RadioButton,
            "ListItem" => ControlType::ListItem,
            "List" => ControlType::List,
            "TreeItem" => ControlType::TreeItem,
            "Tree" => ControlType::Tree,
            "TabItem" => ControlType::TabItem,
            "Tab" => ControlType::Tab,
            "Table" => ControlType::Table,
            "Document" => ControlType::Document,
            "Pane" => ControlType::Pane,
            "Window" => ControlType::Window,
            "Group" => ControlType::Group,
            "Image" => ControlType::Image,
            "Hyperlink" => ControlType::Hyperlink,
            "Custom" => ControlType::Custom,
            _ => {
                warn!("Unknown element type '{}', using Custom", element_type);
                ControlType::Custom
            }
        };
        
        let control_type_id = control_type as i32;
        debug!("Converted element type '{}' to ControlType::{:?} (ID: {})", element_type, control_type, control_type_id);
        
        let automation = self.automation.lock()?;
        let root = automation.get_root_element()?;
        
        debug!("Creating property condition for ControlType with ID: {}", control_type_id);
        let condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(control_type_id), None)
            .map_err(|e| {
                warn!("Failed to create property condition for ControlType with ID {}: {}", control_type_id, e);
                e
            })?;
        
        debug!("Searching for element with ControlType ID: {}", control_type_id);
        match root.find_first(TreeScope::Descendants, &condition) {
            Ok(element) => {
                debug!("Successfully found element with type '{}'", element_type);
                Ok(element)
            },
            Err(e) => {
                warn!("Failed to find element with type '{}': {}", element_type, e);
                Err(e.into())
            }
        }
    }
}

impl CoreUIAutomation for WindowsUIAutomation {
    fn get_active_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>> {
        let element = self.get_active_window()?;
        Ok(Box::new(WindowsWindow::new(element, Arc::new(self.clone()))?))
    }

    fn get_window_containing_focus(&self) -> Result<Box<dyn Window>, Box<dyn Error>> {
        let element = self.get_window_containing_focus()?;
        Ok(Box::new(WindowsWindow::new(element, Arc::new(self.clone()))?))
    }

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
        debug!("CoreUIAutomation::find_element_by_type - Finding element by type: {}", element_type);
        
        // Convert string type to ControlType enum
        let control_type = match element_type {
            "Button" => ControlType::Button,
            "Edit" => ControlType::Edit,
            "Text" => ControlType::Text,
            "ComboBox" => ControlType::ComboBox,
            "CheckBox" => ControlType::CheckBox,
            "RadioButton" => ControlType::RadioButton,
            "ListItem" => ControlType::ListItem,
            "List" => ControlType::List,
            "TreeItem" => ControlType::TreeItem,
            "Tree" => ControlType::Tree,
            "TabItem" => ControlType::TabItem,
            "Tab" => ControlType::Tab,
            "Table" => ControlType::Table,
            "Document" => ControlType::Document,
            "Pane" => ControlType::Pane,
            "Window" => ControlType::Window,
            "Group" => ControlType::Group,
            "Image" => ControlType::Image,
            "Hyperlink" => ControlType::Hyperlink,
            "Custom" => ControlType::Custom,
            _ => {
                warn!("Unknown element type '{}', using Custom", element_type);
                ControlType::Custom
            }
        };
        
        let control_type_id = control_type as i32;
        debug!("CoreUIAutomation::find_element_by_type - Converted '{}' to ID: {}", element_type, control_type_id);
        
        let automation = self.automation.lock()?;
        let condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(control_type_id), None)?;
        let element = automation.get_root_element()?.find_first(TreeScope::Descendants, &condition)?;
        Ok(self.element_to_ui_element(element))
    }
} 