use crate::core::UIElement as CoreUIElement;
use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::patterns::{UIValuePattern, UITextPattern, UILegacyIAccessiblePattern};
use uiautomation::types::{TreeScope, UIProperty};
use uiautomation::controls::ControlType;
use uiautomation::UITreeWalker;
use windows::Win32::Foundation::RECT;
use std::error::Error;
use std::collections::HashMap;
use crate::core::Rect;
use std::any::Any;
use std::convert::TryInto;
use crate::platform::windows::automation::AUTOMATION;
use std::thread;
use std::time::Duration;
use log::{debug, info, warn};
use crate::core::AppendPosition;

/// Windows-specific UI element implementation
pub struct WindowsElement {
    element: UIAutomationElement,
    automation: Option<UITreeWalker>,
}

impl WindowsElement {
    pub fn new(element: UIAutomationElement, automation: Option<UITreeWalker>) -> Self {
        WindowsElement { element, automation }
    }

    pub fn is_offscreen(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.element.is_offscreen()?)
    }

    pub fn get_control_type_variant(&self) -> Result<i32, Box<dyn Error>> {
        let variant = self.element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = variant.try_into()?;
        Ok(control_type_id)
    }

    #[allow(dead_code)]
    fn get_control_type(&self) -> Result<ControlType, Box<dyn Error>> {
        let control_type_id = self.get_control_type_variant()?;
        Ok(ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom))
    }

    #[allow(dead_code)]
    fn get_class_name(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.element.get_classname()?)
    }

    #[allow(dead_code)]
    fn is_input_control(&self) -> Result<bool, Box<dyn Error>> {
        let control_type = self.get_control_type()?;
        Ok(match control_type {
            ControlType::Edit => true,
            ControlType::ComboBox => true,
            ControlType::CheckBox => true,
            ControlType::RadioButton => true,
            ControlType::Slider => true,
            ControlType::Text => true,
            ControlType::Document => true,
            ControlType::Pane => {
                self.element.get_pattern::<UIValuePattern>().is_ok() ||
                self.element.get_pattern::<UITextPattern>().is_ok()
            },
            ControlType::Custom => {
                self.element.get_pattern::<UIValuePattern>().is_ok()
            },
            _ => false,
        })
    }
}

impl CoreUIElement for WindowsElement {
    fn get_name(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.element.get_name()?)
    }

    fn get_type(&self) -> Result<String, Box<dyn Error>> {
        let control_type = self.get_control_type()?;
        Ok(control_type.to_string())
    }

    fn get_text(&self) -> Result<String, Box<dyn Error>> {
        debug!("Getting text from element");
        
        let control_type = self.get_control_type()?;
        let name = self.get_name()?;
        let class_name = self.get_class_name()?;
        
        debug!(
            "Element details - Name: {}, Type: {:?}, ClassName: {}",
            name, control_type, class_name
        );

        // Try Value pattern first
        if let Ok(value_pattern) = self.element.get_pattern::<UIValuePattern>() {
            let value = value_pattern.get_value()?;
            if !value.is_empty() {
                info!("Got text using Value pattern: {}", value);
                return Ok(value);
            }
        }

        // Try Text pattern next
        if let Ok(text_pattern) = self.element.get_pattern::<UITextPattern>() {
            let text = text_pattern.get_document_range()?.get_text(-1)?;
            if !text.is_empty() {
                info!("Got text using Text pattern: {}", text);
                return Ok(text);
            }
        }

        // If no text found, return empty string
        warn!("No text found in element");
        Ok(String::new())
    }

    fn set_text(&self, text: &str) -> Result<(), Box<dyn Error>> {
        debug!("Setting text '{}' in element", text);
        
        if !self.is_input_control()? {
            return Err("Element is not an input control".into());
        }

        let control_type = self.get_control_type()?;
        let name = self.get_name()?;
        let class_name = self.get_class_name()?;
        
        debug!(
            "Element details - Name: {}, Type: {:?}, ClassName: {}",
            name, control_type, class_name
        );

        // First try to select all text using Text pattern
        if let Ok(text_pattern) = self.element.get_pattern::<UITextPattern>() {
            if let Ok(text_range) = text_pattern.get_document_range() {
                if let Err(e) = text_range.select() {
                    warn!("Failed to select text range: {}", e);
                } else {
                    info!("Successfully selected text range");
                }
            }
        }

        // Clear any existing text by sending Ctrl+A and Delete
        thread::sleep(Duration::from_millis(50));
        if let Err(e) = self.element.send_keys("{Ctrl}a{Delete}", 10) {
            warn!("Failed to clear existing text: {}", e);
        }
        thread::sleep(Duration::from_millis(50));

        // Split the text by newlines
        let lines: Vec<&str> = text.split('\n').collect();

        // Type each line separately
        for (i, line) in lines.iter().enumerate() {
            // Type the line
            thread::sleep(Duration::from_millis(50));
            if let Err(e) = self.element.send_text(line, 10) {
                warn!("Failed to type line {}: {}", i, e);
                return Err(format!("Failed to type line {}: {}", i, e).into());
            }

            // If not the last line, press Shift+Enter to create a new line
            if i < lines.len() - 1 {
                thread::sleep(Duration::from_millis(50));
                if let Err(e) = self.element.send_keys("{shift}{enter}", 20) {
                    warn!("Failed to create new line after line {}: {}", i, e);
                    return Err(format!("Failed to create new line after line {}: {}", i, e).into());
                }
            }
        }

        info!("Set new text in input field using keyboard input");

        // Verify the text was set correctly
        thread::sleep(Duration::from_millis(100)); // Give more time for text to be set
        match self.get_text() {
            Ok(current_text) => {
                if current_text == text {
                    info!("Verified text was set correctly: {}", current_text);
                } else {
                    warn!("Text was not set correctly. Expected: {}, Got: {}", text, current_text);
                    // Try one more time with a longer delay
                    thread::sleep(Duration::from_millis(200));
                    match self.get_text() {
                        Ok(retry_text) => {
                            if retry_text == text {
                                info!("Text verified on retry: {}", retry_text);
                            } else {
                                warn!("Text still not set correctly on retry. Expected: {}, Got: {}", text, retry_text);
                            }
                        }
                        Err(e) => warn!("Failed to verify text on retry: {}", e),
                    }
                }
            }
            Err(e) => warn!("Failed to verify text: {}", e),
        }

        Ok(())
    }

    fn append_text(&self, text: &str, position: AppendPosition) -> Result<(), Box<dyn Error>> {
        debug!("Appending text '{}' to element at position {:?}", text, position);
        
        if !self.is_input_control()? {
            return Err("Element is not an input control".into());
        }

        // Try to set focus first
        if let Err(e) = self.element.set_focus() {
            warn!("Failed to set focus: {}", e);
        }

        // Handle cursor positioning based on the requested position
        match position {
            AppendPosition::CurrentCursor => {
                // No need to move cursor, we're already at the right position
                debug!("Appending at current cursor position");
            },
            AppendPosition::EndOfLine => {
                // Move to end of current line
                if let Err(e) = self.element.send_keys("{END}", 10) {
                    warn!("Failed to move cursor to end of line: {}", e);
                } else {
                    debug!("Moved cursor to end of line");
                }
            },
            AppendPosition::EndOfText => {
                // Move to end of text block
                if let Err(e) = self.element.send_keys("{CTRL}{END}", 10) {
                    warn!("Failed to move cursor to end of text: {}", e);
                } else {
                    debug!("Moved cursor to end of text");
                }
            },
        }

        // Add a small delay after positioning
        thread::sleep(Duration::from_millis(50));

        // Handle multiline text properly
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Type the line
            if let Err(e) = self.element.send_text(line, 10) {
                warn!("Failed to type line {}: {}", i, e);
                return Err(format!("Failed to type line {}: {}", i, e).into());
            }

            // If not the last line, add a newline
            if i < lines.len() - 1 {
                thread::sleep(Duration::from_millis(50));
                if let Err(e) = self.element.send_keys("+{ENTER}", 20) {
                    warn!("Failed to add newline after line {}: {}", i, e);
                    return Err(format!("Failed to add newline after line {}: {}", i, e).into());
                }
            }
        }

        info!("Successfully appended text to input field");
        Ok(())
    }

    fn is_enabled(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.element.is_enabled()?)
    }

    fn get_properties(&self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut properties = HashMap::new();
        
        // Get basic properties
        if let Ok(name) = self.element.get_name() {
            properties.insert("name".to_string(), name);
        }
        
        // Get control type properly
        if let Ok(control_type_id) = self.get_control_type_variant() {
            if let Ok(control_type) = ControlType::try_from(control_type_id) {
                properties.insert("control_type".to_string(), control_type.to_string());
            }
        }
        
        // Get enabled state
        if let Ok(variant) = self.element.get_property_value(UIProperty::IsEnabled) {
            let enabled: bool = variant.try_into().unwrap_or(false);
            properties.insert("enabled".to_string(), enabled.to_string());
        }
        
        // Get keyboard focusable state
        if let Ok(variant) = self.element.get_property_value(UIProperty::IsKeyboardFocusable) {
            let focusable: bool = variant.try_into().unwrap_or(false);
            properties.insert("keyboard_focusable".to_string(), focusable.to_string());
        }
        
        // Add more properties as needed
        
        Ok(properties)
    }

    fn get_bounds(&self) -> Result<Option<Rect>, Box<dyn Error>> {
        match self.element.get_bounding_rectangle() {
            Ok(rect) => {
                let rect: RECT = rect.try_into()?;
                Ok(Some(Rect {
                    left: rect.left,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                }))
            },
            Err(_) => Ok(None),
        }
    }

    fn get_children(&self) -> Result<Vec<Box<dyn CoreUIElement>>, Box<dyn Error>> {
        let mut children = Vec::new();
        let automation = self.automation.clone();
        if let Some(walker) = &self.automation {
            if let Ok(first_child) = walker.get_first_child(&self.element) {
                children.push(Box::new(WindowsElement::new(first_child.clone(), automation.clone())) as Box<dyn CoreUIElement>);
                let mut next = first_child;
                while let Ok(sibling) = walker.get_next_sibling(&next) {
                    children.push(Box::new(WindowsElement::new(sibling.clone(), automation.clone())) as Box<dyn CoreUIElement>);
                    next = sibling;
                }
            }
        } else {
            // Use a match-all condition (TrueCondition) for children
            let condition = AUTOMATION.with(|cell| {
                let automation = cell.borrow();
                let automation = automation.as_ref().ok_or_else(|| Box::<dyn Error>::from("No global automation instance"))?;
                let automation = automation.lock().map_err(|e| Box::<dyn Error>::from(format!("Failed to lock automation: {}", e)))?;
                automation.create_true_condition().map_err(|e| Box::<dyn Error>::from(format!("Failed to create true condition: {}", e)))
            })?;
            let child_elements = self.element.find_all(TreeScope::Children, &condition)?;
            for child in child_elements {
                children.push(Box::new(WindowsElement::new(child.clone(), None)) as Box<dyn CoreUIElement>);
            }
        }
        if children.is_empty() {
            return Err("No children found".into());
        }
        Ok(children)
    }

    fn to_tree_node(&self) -> Result<Box<dyn CoreUIElement>, Box<dyn Error>> {
        Ok(Box::new(WindowsElement::new(self.element.clone(), self.automation.clone())) as Box<dyn CoreUIElement>)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// For backward compatibility
impl WindowsElement {
    pub fn append_text_compat(&self, text: &str) -> Result<(), Box<dyn Error>> {
        self.append_text(text, AppendPosition::EndOfText)
    }
} 