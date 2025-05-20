use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use chrono::Utc;
use log::debug;
use std::convert::TryInto;

use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::types::{TreeScope, UIProperty};
use uiautomation::variants::Variant;
use uiautomation::controls::ControlType;

use crate::core::{Window, UIElement, UITree, UIQuery, UITreeNode};
use super::automation::WindowsUIAutomation;
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

/// Windows-specific window implementation
#[derive(Clone)]
pub struct WindowsWindow {
    element: UIAutomationElement,
    automation: Arc<WindowsUIAutomation>,
    window_info: Option<WindowInfo>,
}

impl WindowsWindow {
    pub fn new(element: UIAutomationElement, automation: Arc<WindowsUIAutomation>) -> Result<Self, Box<dyn Error>> {
        let window_info = WindowInfo::get_current();
        if window_info.is_none() {
            debug!("Failed to get window info using Windows API, falling back to UIAutomation");
        }
        Ok(WindowsWindow { 
            element, 
            automation,
            window_info,
        })
    }
}

impl Window for WindowsWindow {
    fn get_title(&self) -> Result<String, Box<dyn Error>> {
        if let Some(info) = &self.window_info {
            Ok(info.title.clone())
        } else {
            Ok(self.element.get_name()?)
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
        let element = self.automation.automation.lock()?.get_focused_element()?;
        Ok(self.automation.element_to_ui_element(element))
    }

    fn get_ui_tree(&self) -> Result<UITree, Box<dyn Error>> {
        let root_element = self.element.clone();
        let root_name = root_element.get_name()?;
        let root_class = root_element.get_classname()?;
        
        // Create properties map
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), root_name.clone());
        properties.insert("class_name".to_string(), root_class.clone());
        
        // Get control type properly
        let control_type = if let Ok(variant) = root_element.get_property_value(UIProperty::ControlType) {
            let control_type_id: i32 = variant.try_into()?;
            if let Ok(control_type) = ControlType::try_from(control_type_id) {
                control_type.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };
        
        properties.insert("control_type".to_string(), control_type.clone());
        
        // Get enabled state
        let is_enabled = if let Ok(variant) = root_element.get_property_value(UIProperty::IsEnabled) {
            let enabled: bool = variant.try_into().unwrap_or(false);
            enabled
        } else {
            false
        };
        properties.insert("enabled".to_string(), is_enabled.to_string());
        
        // Get keyboard focusable state
        let is_keyboard_focusable = if let Ok(variant) = root_element.get_property_value(UIProperty::IsKeyboardFocusable) {
            let focusable: bool = variant.try_into().unwrap_or(false);
            focusable
        } else {
            false
        };
        properties.insert("keyboard_focusable".to_string(), is_keyboard_focusable.to_string());
        
        // Create a tree walker for traversing the UI tree
        let automation = self.automation.automation.lock()?;
        let walker = automation.create_tree_walker()?;
        
        // Create a WindowsElement wrapper for the root with the tree walker
        let root_windows_element = super::element::WindowsElement::new(root_element.clone(), Some(walker));
        
        // Recursively build the tree
        fn build_tree_node(element: &super::element::WindowsElement) -> Result<UITreeNode, Box<dyn Error>> {
            let name = element.get_name()?;
            let control_type = if let Ok(variant) = element.get_control_type_variant() {
                if let Ok(control_type) = ControlType::try_from(variant) {
                    control_type.to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };
            
            let properties = element.get_properties()?;
            let bounds = element.get_bounds()?;
            let is_enabled = element.is_enabled()?;
            let is_visible = !element.is_offscreen()?;
            
            // Get children
            let mut children = Vec::new();
            if let Ok(child_elements) = element.get_children() {
                for child in child_elements {
                    if let Some(child_windows_element) = child.as_any().downcast_ref::<super::element::WindowsElement>() {
                        if let Ok(child_node) = build_tree_node(child_windows_element) {
                            children.push(child_node);
                        }
                    }
                }
            }
            
            Ok(UITreeNode {
                name,
                control_type,
                properties,
                children,
                bounds,
                is_enabled,
                is_visible,
            })
        }
        
        // Build the root node
        let root_node = build_tree_node(&root_windows_element)?;
        
        Ok(UITree {
            root: root_node,
            timestamp: Utc::now(),
            window_title: root_name,
            window_class: root_class,
        })
    }

    fn find_elements(&self, query: &UIQuery) -> Result<Vec<Box<dyn UIElement>>, Box<dyn Error>> {
        debug!("Finding elements with query: {:?}", query);
        
        match query {
            UIQuery::ByName(name) => {
                let element = self.automation.find_element_by_name(name)?;
                Ok(vec![self.automation.element_to_ui_element(element)])
            },
            UIQuery::ByType(control_type) => {
                let element = self.automation.find_element_by_type(control_type)?;
                Ok(vec![self.automation.element_to_ui_element(element)])
            },
            UIQuery::ByProperty(key, value) => {
                let automation = self.automation.automation.lock()?;
                // For property queries, we need to get all children and filter
                let all_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Custom as i32), None)?;
                let children = self.element.find_all(TreeScope::Descendants, &all_condition)?;
                let mut result = Vec::new();
                for child in children {
                    let windows_element = super::element::WindowsElement::new(child.clone(), None);
                    if let Ok(properties) = windows_element.get_properties() {
                        if properties.get(key) == Some(value) {
                            result.push(Box::new(windows_element) as Box<dyn UIElement>);
                        }
                    }
                }
                Ok(result)
            },
            UIQuery::And(queries) => {
                let mut results = Vec::new();
                for query in queries {
                    let elements = self.find_elements(query)?;
                    if results.is_empty() {
                        results = elements;
                    } else {
                        // Keep only elements that exist in both results
                        results.retain(|e1| {
                            elements.iter().any(|e2| {
                                e1.get_name().ok() == e2.get_name().ok() &&
                                e1.get_type().ok() == e2.get_type().ok()
                            })
                        });
                    }
                }
                Ok(results)
            },
            UIQuery::Or(queries) => {
                let mut results = Vec::new();
                for query in queries {
                    let elements = self.find_elements(query)?;
                    results.extend(elements);
                }
                Ok(results)
            },
            UIQuery::Not(query) => {
                let automation = self.automation.automation.lock()?;
                let all_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Custom as i32), None)?;
                let children = self.element.find_all(TreeScope::Descendants, &all_condition)?;
                let mut result = Vec::new();
                for child in children {
                    let windows_element = super::element::WindowsElement::new(child.clone(), None);
                    if !query.matches(&windows_element)? {
                        result.push(Box::new(windows_element) as Box<dyn UIElement>);
                    }
                }
                Ok(result)
            },
            UIQuery::Child(query) => {
                let automation = self.automation.automation.lock()?;
                let all_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Custom as i32), None)?;
                let children = self.element.find_all(TreeScope::Children, &all_condition)?;
                let mut result = Vec::new();
                for child in children {
                    let windows_element = super::element::WindowsElement::new(child.clone(), None);
                    if query.matches(&windows_element)? {
                        result.push(Box::new(windows_element) as Box<dyn UIElement>);
                    }
                }
                Ok(result)
            },
            UIQuery::Descendant(query) => {
                let automation = self.automation.automation.lock()?;
                let all_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Custom as i32), None)?;
                let descendants = self.element.find_all(TreeScope::Descendants, &all_condition)?;
                let mut result = Vec::new();
                for descendant in descendants {
                    let windows_element = super::element::WindowsElement::new(descendant.clone(), None);
                    if query.matches(&windows_element)? {
                        result.push(Box::new(windows_element) as Box<dyn UIElement>);
                    }
                }
                Ok(result)
            },
            UIQuery::Parent(query) => {
                let automation = self.automation.automation.lock()?;
                let parent_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Window as i32), None)?;
                if let Ok(parent) = self.element.find_first(TreeScope::Parent, &parent_condition) {
                    let windows_element = super::element::WindowsElement::new(parent.clone(), None);
                    if query.matches(&windows_element)? {
                        return Ok(vec![Box::new(windows_element) as Box<dyn UIElement>]);
                    }
                }
                Ok(Vec::new())
            },
            UIQuery::Ancestor(query) => {
                let automation = self.automation.automation.lock()?;
                let ancestor_condition = automation.create_property_condition(UIProperty::ControlType, Variant::from(ControlType::Window as i32), None)?;
                let mut current = self.element.clone();
                let mut result = Vec::new();
                while let Ok(parent) = current.find_first(TreeScope::Parent, &ancestor_condition) {
                    let windows_element = super::element::WindowsElement::new(parent.clone(), None);
                    if query.matches(&windows_element)? {
                        result.push(Box::new(windows_element) as Box<dyn UIElement>);
                    }
                    current = parent;
                }
                Ok(result)
            },
        }
    }
} 