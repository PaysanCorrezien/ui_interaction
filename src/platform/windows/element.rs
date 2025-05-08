use crate::core::UIElement;
use uiautomation::core::UIElement as UIAutomationElement;
use uiautomation::patterns::{UIValuePattern, UITextPattern};
use uiautomation::types::UIProperty;
use uiautomation::variants::Variant;
use uiautomation::controls::ControlType;
use std::error::Error;
use log::{debug, info, warn};
use std::{thread, time::Duration};

/// Windows-specific UI element implementation
pub struct WindowsElement {
    element: UIAutomationElement,
}

impl WindowsElement {
    pub fn new(element: UIAutomationElement) -> Self {
        WindowsElement { element }
    }

    fn get_control_type(&self) -> Result<ControlType, Box<dyn Error>> {
        let control_type: Variant = self.element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        Ok(ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom))
    }

    fn get_class_name(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.element.get_classname()?)
    }

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

impl UIElement for WindowsElement {
    fn get_name(&self) -> Result<String, Box<dyn Error>> {
        let name: Variant = self.element.get_property_value(UIProperty::Name)?;
        Ok(name.get_string()?)
    }

    fn get_type(&self) -> Result<String, Box<dyn Error>> {
        let control_type = self.get_control_type()?;
        Ok(format!("{:?}", control_type))
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

    fn append_text(&self, text: &str) -> Result<(), Box<dyn Error>> {
        debug!("Appending text '{}' to element", text);
        
        if !self.is_input_control()? {
            return Err("Element is not an input control".into());
        }

        // Try to set focus first
        if let Err(e) = self.element.set_focus() {
            warn!("Failed to set focus: {}", e);
        }
        
        // Move cursor to the end of the text
        if let Ok(text_pattern) = self.element.get_pattern::<UITextPattern>() {
            if let Ok(text_range) = text_pattern.get_document_range() {
                // Select the end of the text range
                if let Err(e) = text_range.select() {
                    warn!("Failed to select text range: {}", e);
                } else {
                    // Move cursor to the end by pressing End key
                    if let Err(e) = self.element.send_keys("{end}", 10) {
                        warn!("Failed to move cursor to end: {}", e);
                    } else {
                        info!("Successfully moved cursor to end of text");
                    }
                }
            }
        }
        
        // Type the text to append
        thread::sleep(Duration::from_millis(50));
        
        // For very long texts, we might need to adjust the interval
        let interval = if text.len() > 1000 {
            20 // Slower for very long texts
        } else {
            10 // Faster for shorter texts
        };
        
        if let Err(e) = self.element.send_text(text, interval) {
            return Err(format!("Failed to append text: {}", e).into());
        }
        
        info!("Successfully appended text to input field");
        Ok(())
    }
} 