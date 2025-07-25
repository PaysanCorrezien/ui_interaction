use crate::core::UIElement as CoreUIElement;
use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::patterns::{UIValuePattern, UITextPattern};
use uiautomation::types::{TreeScope, UIProperty};
use uiautomation::variants::Variant;
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
        debug!("WindowsElement::get_name - Getting name");
        match self.element.get_name() {
            Ok(name) => {
                debug!("WindowsElement::get_name - Success: {}", name);
                Ok(name)
            },
            Err(e) => {
                warn!("WindowsElement::get_name - Error: {}", e);
                Err(e.into())
            }
        }
    }

    fn get_type(&self) -> Result<String, Box<dyn Error>> {
        debug!("WindowsElement::get_type - Getting control type");
        let control_type = self.get_control_type()?;
        debug!("WindowsElement::get_type - Success: {}", control_type);
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

        // Step 1: Try to send the whole text at once
        info!("Attempting to set entire text: '{}'", text);
        
        // Try sending the whole text
        if let Err(e) = self.element.send_text(text, 30) {
            warn!("Failed to send entire text: {}", e);
            return Err(format!("Failed to send text: {}", e).into());
        }
        
        // Step 2: Verify what actually got input
        thread::sleep(Duration::from_millis(200)); // Give time for text to be processed
        let actual_text = self.get_text().unwrap_or_default();
        
        info!("Expected text: '{}'", text);
        info!("Actual text: '{}'", actual_text);
        
        // Step 3: Check if we got what we wanted
        if actual_text == text {
            info!("✓ Text set correctly on first try");
            return Ok(());
        }
        
        // Step 4: If not perfect, clear and try correction approach
        warn!("Text not set correctly, attempting corrections...");
        
        // Clear everything and try word-by-word
        if let Err(e) = self.element.send_keys("{Ctrl}a{Delete}", 10) {
            warn!("Failed to clear text for correction: {}", e);
        }
        thread::sleep(Duration::from_millis(50));
        
        // Try word-by-word correction
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            info!("Setting word {}: '{}'", i + 1, word);
            
            if let Err(e) = self.element.send_text(word, 20) {
                warn!("Failed to send word '{}': {}, trying character-by-character", word, e);
                
                // If word fails, try character by character for this word only
                for ch in word.chars() {
                    let char_str = ch.to_string();
                    
                    // Handle newlines specially
                    if ch == '\n' {
                        if let Err(e) = self.element.send_keys("{shift}{enter}", 20) {
                            warn!("Failed to add newline: {}", e);
                        }
                        continue;
                    }
                    
                    if let Err(e2) = self.element.send_text(&char_str, 15) {
                        warn!("Failed to send character '{}': {}", ch, e2);
                    }
                    
                    // Small delay for non-ASCII characters
                    if !ch.is_ascii() {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            
            // Add space between words (except for last word)
            if i < words.len() - 1 {
                if let Err(e) = self.element.send_text(" ", 10) {
                    warn!("Failed to send space: {}", e);
                }
            }
            
            // Small delay between words
            thread::sleep(Duration::from_millis(20));
        }

        info!("Completed text setting with corrections");

        // Final verification
        thread::sleep(Duration::from_millis(100));
        match self.get_text() {
            Ok(current_text) => {
                if current_text == text {
                    info!("✓ Text verified correctly after corrections");
                } else {
                    warn!("Text still not correct after corrections. Expected: '{}', Got: '{}'", text, current_text);
                }
            }
            Err(e) => warn!("Failed to verify final text: {}", e),
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

        // Step 1: Try to send the whole text at once
        info!("Attempting to send entire text: '{}'", text);
        
        // Get text before appending to know where we started
        let text_before = self.get_text().unwrap_or_default();
        
        // Try sending the whole text
        if let Err(e) = self.element.send_text(text, 30) {
            warn!("Failed to send entire text: {}", e);
            return Err(format!("Failed to send text: {}", e).into());
        }
        
        // Step 2: Verify what actually got input
        thread::sleep(Duration::from_millis(200)); // Give time for text to be processed
        let text_after = self.get_text().unwrap_or_default();
        
        // Extract what was actually appended
        let actually_appended = if text_after.len() > text_before.len() {
            &text_after[text_before.len()..]
        } else {
            ""
        };
        
        info!("Expected to append: '{}'", text);
        info!("Actually appended: '{}'", actually_appended);
        
        // Step 3: Check if we got what we wanted
        if actually_appended == text {
            info!("✓ Text appended correctly on first try");
            return Ok(());
        }
        
        // Step 4: If not perfect, identify and fix differences
        warn!("Text not appended correctly, attempting corrections...");
        
        // Clear what we just added and try correction approach
        let chars_to_remove = actually_appended.chars().count();
        if chars_to_remove > 0 {
            // Select and delete what was incorrectly added
            for _ in 0..chars_to_remove {
                if let Err(e) = self.element.send_keys("{BACKSPACE}", 10) {
                    warn!("Failed to backspace: {}", e);
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        // Try word-by-word correction
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            info!("Sending word {}: '{}'", i + 1, word);
            
            if let Err(e) = self.element.send_text(word, 20) {
                warn!("Failed to send word '{}': {}, trying character-by-character", word, e);
                
                // If word fails, try character by character for this word only
                for ch in word.chars() {
                    let char_str = ch.to_string();
                    if let Err(e2) = self.element.send_text(&char_str, 15) {
                        warn!("Failed to send character '{}': {}", ch, e2);
                    }
                    
                    // Small delay for non-ASCII characters
                    if !ch.is_ascii() {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            
            // Add space between words (except for last word)
            if i < words.len() - 1 {
                if let Err(e) = self.element.send_text(" ", 10) {
                    warn!("Failed to send space: {}", e);
                }
            }
            
            // Small delay between words
            thread::sleep(Duration::from_millis(20));
        }

        info!("Completed text append with corrections");
        Ok(())
    }

    fn click(&self) -> Result<(), Box<dyn Error>> {
        // Try to get the element's bounds for clicking
        if let Ok(Some(bounds)) = self.get_bounds() {
            // Calculate center point
            let _center_x = (bounds.left + bounds.right) / 2;
            let _center_y = (bounds.top + bounds.bottom) / 2;
            
            // Use the UIA click method
            if let Err(e) = self.element.click() {
                // Fallback: Try using invoke pattern if available
                use uiautomation::patterns::UIInvokePattern;
                if let Ok(invoke_pattern) = self.element.get_pattern::<UIInvokePattern>() {
                    if let Err(e) = invoke_pattern.invoke() {
                        return Err(format!("Failed to click element: {}", e).into());
                    } else {
                        return Ok(());
                    }
                } else {
                    return Err(format!("Failed to click element: {}", e).into());
                }
            } else {
                return Ok(());
            }
        } else {
            // Try invoke pattern as fallback
            use uiautomation::patterns::UIInvokePattern;
            if let Ok(invoke_pattern) = self.element.get_pattern::<UIInvokePattern>() {
                if let Err(e) = invoke_pattern.invoke() {
                    return Err(format!("Failed to click element: {}", e).into());
                } else {
                    return Ok(());
                }
            } else {
                return Err("Failed to click element: no bounds and no invoke pattern available".into());
            }
        }
    }

    fn is_enabled(&self) -> Result<bool, Box<dyn Error>> {
        match self.element.is_enabled() {
            Ok(enabled) => Ok(enabled),
            Err(e) => Err(e.into())
        }
    }

    fn get_properties(&self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut properties = HashMap::new();
        
        // Get only essential properties for performance
        if let Ok(name) = self.element.get_name() {
            properties.insert("name".to_string(), name);
        }
        if let Ok(class_name) = self.element.get_classname() {
            properties.insert("class_name".to_string(), class_name);
        }
        
        // Get control type efficiently
        if let Ok(variant) = self.element.get_property_value(UIProperty::ControlType) {
            if let Ok(control_type_id) = <Variant as TryInto<i32>>::try_into(variant) {
                if let Ok(control_type) = ControlType::try_from(control_type_id) {
                    properties.insert("control_type".to_string(), control_type.to_string());
                }
            }
        }
        
        // Get automation ID if available (useful for finding elements)
        if let Ok(variant) = self.element.get_property_value(UIProperty::AutomationId) {
            let automation_id = variant.to_string();
            if !automation_id.is_empty() && automation_id != "null" {
                properties.insert("automation_id".to_string(), automation_id);
            }
        }
        
        // Get enabled state efficiently
        if let Ok(enabled) = self.element.is_enabled() {
            properties.insert("enabled".to_string(), enabled.to_string());
        }
        
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