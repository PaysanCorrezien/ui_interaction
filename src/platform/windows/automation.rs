use crate::core::{UIElement, Window, UIAutomation as CoreUIAutomation};
use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::types::{UIProperty, TreeScope};
use uiautomation::controls::ControlType;
use uiautomation::variants::Variant;
use uiautomation::patterns::{UIValuePattern, UITextPattern};
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
use super::element::WindowsElement;
use super::window::WindowsWindow;
use log::{debug, info, warn};

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

/// Windows-specific UIAutomation implementation
#[derive(Clone)]
pub struct WindowsUIAutomation {
    automation: uiautomation::core::UIAutomation,
}

impl WindowsUIAutomation {
    /// Create a new instance of WindowsUIAutomation
    pub fn new() -> Result<Self, Box<dyn Error>> {
        debug!("Initializing WindowsUIAutomation instance");
        Ok(WindowsUIAutomation {
            automation: uiautomation::core::UIAutomation::new()?,
        })
    }

    /// Get the currently focused window
    pub fn get_focused_window(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Getting focused window");
        let window_info = WindowInfo::get_current()
            .ok_or_else(|| "Failed to get window info using Windows API")?;
        
        info!("Focused window - Title: {}, ClassName: {}", 
            window_info.title, window_info.class_name);
        
        self.automation.get_focused_element().map_err(|e| e.into())
    }

    /// Get the currently focused element with its details
    pub fn get_focused_element_details(&self) -> Result<(UIAutomationElement, String, String), Box<dyn Error>> {
        debug!("Getting currently focused element details");
        let element = self.automation.get_focused_element()?;
        
        let name: Variant = element.get_property_value(UIProperty::Name)?;
        let control_type: Variant = element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        let control_type_enum = ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom);

        info!(
            "Found focused element: Name: {}, Type: {:?}",
            name.get_string()?,
            control_type_enum
        );

        Ok((element, name.get_string()?, format!("{:?}", control_type_enum)))
    }

    /// Get the currently focused input element
    pub fn get_focused_input_element(&self) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Getting currently focused input element");
        
        let focused_element = self.automation.get_focused_element()?;
        
        let control_type: Variant = focused_element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        
        let name: Variant = focused_element.get_property_value(UIProperty::Name)?;
        let class_name = focused_element.get_classname()?;
        debug!(
            "Focused element details - Name: {}, Type: {:?}, ClassName: {}",
            name.get_string()?,
            ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom),
            class_name
        );
        
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
        let root = self.automation.get_root_element()?;
        let condition = self.automation.create_property_condition(UIProperty::Name, Variant::from(name), None)?;
        root.find_first(TreeScope::Descendants, &condition)
            .map_err(|e| e.into())
    }

    /// Find an element by its type
    pub fn find_element_by_type(&self, element_type: &str) -> Result<UIAutomationElement, Box<dyn Error>> {
        debug!("Finding element by type: {}", element_type);
        
        // First check if the focused element matches the requested type
        if let Ok(focused_element) = self.automation.get_focused_element() {
            let control_type: Variant = focused_element.get_property_value(UIProperty::ControlType)?;
            let control_type_id: i32 = control_type.try_into()?;
            let focused_type = ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom);
            
            let requested_type = match element_type.to_lowercase().as_str() {
                "button" => ControlType::Button,
                "edit" => ControlType::Edit,
                "text" => ControlType::Text,
                "checkbox" => ControlType::CheckBox,
                "combobox" => ControlType::ComboBox,
                "list" => ControlType::List,
                "listitem" => ControlType::ListItem,
                "menu" => ControlType::Menu,
                "menuitem" => ControlType::MenuItem,
                "pane" => ControlType::Pane,
                "radiobutton" => ControlType::RadioButton,
                "scrollbar" => ControlType::ScrollBar,
                "slider" => ControlType::Slider,
                "spinner" => ControlType::Spinner,
                "statusbar" => ControlType::StatusBar,
                "tab" => ControlType::Tab,
                "tabitem" => ControlType::TabItem,
                "toolbar" => ControlType::ToolBar,
                "tooltip" => ControlType::ToolTip,
                "tree" => ControlType::Tree,
                "treeitem" => ControlType::TreeItem,
                "window" => ControlType::Window,
                _ => return Err("Unsupported element type".into()),
            };
            
            if focused_type == requested_type {
                debug!("Using focused element as it matches requested type: {:?}", focused_type);
                return Ok(focused_element);
            }
        }
        
        // If focused element doesn't match, search the entire tree
        let root = self.automation.get_root_element()?;
        let condition = self.automation.create_property_condition(
            UIProperty::ControlType, 
            Variant::from(match element_type.to_lowercase().as_str() {
                "button" => ControlType::Button,
                "edit" => ControlType::Edit,
                "text" => ControlType::Text,
                "checkbox" => ControlType::CheckBox,
                "combobox" => ControlType::ComboBox,
                "list" => ControlType::List,
                "listitem" => ControlType::ListItem,
                "menu" => ControlType::Menu,
                "menuitem" => ControlType::MenuItem,
                "pane" => ControlType::Pane,
                "radiobutton" => ControlType::RadioButton,
                "scrollbar" => ControlType::ScrollBar,
                "slider" => ControlType::Slider,
                "spinner" => ControlType::Spinner,
                "statusbar" => ControlType::StatusBar,
                "tab" => ControlType::Tab,
                "tabitem" => ControlType::TabItem,
                "toolbar" => ControlType::ToolBar,
                "tooltip" => ControlType::ToolTip,
                "tree" => ControlType::Tree,
                "treeitem" => ControlType::TreeItem,
                "window" => ControlType::Window,
                _ => return Err("Unsupported element type".into()),
            } as i32), 
            None
        )?;
        
        root.find_first(TreeScope::Descendants, &condition)
            .map_err(|e| e.into())
    }
}

impl CoreUIAutomation for WindowsUIAutomation {
    fn get_focused_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>> {
        let element = self.get_focused_window()?;
        Ok(Box::new(WindowsWindow::new(element, Box::new(self.clone()))?))
    }

    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>> {
        let (element, _, _) = self.get_focused_element_details()?;
        Ok(Box::new(WindowsElement::new(element)))
    }

    fn find_element_by_name(&self, name: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>> {
        let element = self.find_element_by_name(name)?;
        Ok(Box::new(WindowsElement::new(element)))
    }

    fn find_element_by_type(&self, element_type: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>> {
        let element = self.find_element_by_type(element_type)?;
        Ok(Box::new(WindowsElement::new(element)))
    }
} 