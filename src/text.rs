use log::{debug, info, warn};
use std::{thread, time::Duration};
use uiautomation::core::UIElement;
use uiautomation::patterns::UITextPattern;
use uiautomation::types::UIProperty;
use std::error::Error;

use crate::automation::UIAutomation;

/// Text handling functionality
pub struct TextHandler<'a> {
    automation: &'a UIAutomation,
}

impl<'a> TextHandler<'a> {
    /// Create a new TextHandler instance
    pub fn new(automation: &'a UIAutomation) -> Self {
        TextHandler { automation }
    }

    /// Capture the text of an input field
    pub fn get_text(
        &self,
        element: &UIElement,
    ) -> Result<String, Box<dyn Error>> {
        debug!("Capturing input field text");
        
        // Try different properties to get text value
        let mut text = String::new();
        
        // Try ValueValue property first
        if let Ok(value) = element.get_property_value(UIProperty::ValueValue) {
            if let Ok(value_str) = value.get_string() {
                text = value_str;
            }
        }
        
        // If empty, try Name property
        if text.is_empty() {
            if let Ok(name) = element.get_property_value(UIProperty::Name) {
                if let Ok(name_str) = name.get_string() {
                    text = name_str;
                }
            }
        }
        
        info!("Captured text: {}", text);
        Ok(text)
    }

    /// Set text in an input field
    pub fn set_text(
        &self,
        element: &UIElement,
        new_text: &str,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Setting input text: {}", new_text);
        
        // Try to set focus first
        if let Err(e) = element.set_focus() {
            warn!("Failed to set focus: {}", e);
        }
        
        // Check if the text contains newlines
        if new_text.contains('\n') {
            info!("Detected multiline text, using Shift+Enter approach");
            return self.set_multiline_text(element, new_text);
        }
        
        // For single-line text, use keyboard input
        return self.set_single_line_text(element, new_text);
    }
    
    /// Append text to the existing content of an input field
    pub fn append_text(
        &self,
        element: &UIElement,
        text_to_append: &str,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Appending text: {}", text_to_append);
        
        // Try to set focus first
        if let Err(e) = element.set_focus() {
            warn!("Failed to set focus: {}", e);
        }
        
        // Move cursor to the end of the text
        if let Ok(text_pattern) = element.get_pattern::<UITextPattern>() {
            if let Ok(text_range) = text_pattern.get_document_range() {
                // Select the end of the text range
                if let Err(e) = text_range.select() {
                    warn!("Failed to select text range: {}", e);
                } else {
                    // Move cursor to the end by pressing End key
                    if let Err(e) = element.send_keys("{end}", 10) {
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
        let interval = if text_to_append.len() > 1000 {
            20 // Slower for very long texts
        } else {
            10 // Faster for shorter texts
        };
        
        if let Err(e) = element.send_text(text_to_append, interval) {
            return Err(format!("Failed to append text: {}", e).into());
        }
        
        info!("Successfully appended text to input field");
        Ok(())
    }
    
    /// Helper method for setting single-line text
    fn set_single_line_text(
        &self,
        element: &UIElement,
        new_text: &str,
    ) -> Result<(), Box<dyn Error>> {
        info!("Using keyboard input to set single-line text");
        
        // First try to select all text using Text pattern
        if let Ok(text_pattern) = element.get_pattern::<UITextPattern>() {
            if let Ok(text_range) = text_pattern.get_document_range() {
                if let Err(e) = text_range.select() {
                    warn!("Failed to select text range: {}", e);
                } else {
                    info!("Successfully selected text range");
                }
            }
        }
        
        // Finally, type the new text
        thread::sleep(Duration::from_millis(50));
        
        // For very long texts, we might need to adjust the interval
        // to prevent input loss
        let interval = if new_text.len() > 1000 {
            20 // Slower for very long texts
        } else {
            10 // Faster for shorter texts
        };
        
        if let Err(e) = element.send_text(new_text, interval) {
            return Err(format!("Failed to type new text: {}", e).into());
        }
        
        info!("Set new text in input field using keyboard input");
        
        // Verify the text was set correctly
        thread::sleep(Duration::from_millis(50));
        match self.get_text(element) {
            Ok(current_text) => {
                if current_text == new_text {
                    info!("Verified text was set correctly: {}", current_text);
                } else {
                    warn!("Text was not set correctly. Expected: {}, Got: {}", new_text, current_text);
                }
            }
            Err(e) => warn!("Failed to verify text: {}", e),
        }
        
        Ok(())
    }
    
    /// Helper method for setting multiline text
    fn set_multiline_text(
        &self,
        element: &UIElement,
        new_text: &str,
    ) -> Result<(), Box<dyn Error>> {
        // First try to select all text using Text pattern
        if let Ok(text_pattern) = element.get_pattern::<UITextPattern>() {
            if let Ok(text_range) = text_pattern.get_document_range() {
                if let Err(e) = text_range.select() {
                    warn!("Failed to select text range: {}", e);
                } else {
                    info!("Successfully selected text range");
                }
            }
        }
        
        // Split the text by newlines
        let lines: Vec<&str> = new_text.split('\n').collect();
        
        // Type each line separately
        for (i, line) in lines.iter().enumerate() {
            // Type the line
            thread::sleep(Duration::from_millis(50));
            if let Err(e) = element.send_text(line, 10) {
                warn!("Failed to type line {}: {}", i, e);
                return Err(format!("Failed to type line {}: {}", i, e).into());
            }
            
            // If not the last line, press Shift+Enter to create a new line
            if i < lines.len() - 1 {
                thread::sleep(Duration::from_millis(50));
                // Use the correct syntax for Shift+Enter
                if let Err(e) = element.send_keys("{shift}{enter}", 20) {
                    warn!("Failed to create new line after line {}: {}", i, e);
                    return Err(format!("Failed to create new line after line {}: {}", i, e).into());
                }
            }
        }
        
        info!("Successfully set multiline text using Shift+Enter approach");
        
        // Verify the text was set correctly
        thread::sleep(Duration::from_millis(50));
        match self.get_text(element) {
            Ok(current_text) => {
                // For verification, we need to normalize the text
                // Some applications might represent newlines differently
                let normalized_current = current_text.replace("\r\n", "\n").replace("\r", "\n");
                let normalized_expected = new_text.replace("\r\n", "\n").replace("\r", "\n");
                
                if normalized_current == normalized_expected {
                    info!("Verified multiline text was set correctly");
                    return Ok(());
                } else {
                    warn!("Multiline text was not set correctly. Expected: {}, Got: {}", new_text, current_text);
                }
            }
            Err(e) => warn!("Failed to verify multiline text: {}", e),
        }
        
        Ok(())
    }
}
