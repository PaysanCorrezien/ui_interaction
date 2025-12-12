//! # UI Automation Library
//! 
//! A cross-platform library for UI automation and desktop application testing.
//! This library provides a high-level interface to interact with UI elements,
//! windows, and applications programmatically.
//!
//! ## Features
//!
//! - **Cross-platform support**: Windows (full), Linux (planned)
//! - **Application discovery**: Find and manage running applications
//! - **Window automation**: Control window state and focus
//! - **Element interaction**: Click, type, and manipulate UI elements
//! - **UI tree inspection**: Analyze application UI structure
//! - **Python bindings**: Use from Python with full feature support
//!
//! ## Quick Start
//!
//! ```rust
//! use uia_interaction::factory::UIAutomationFactory;
//! use uia_interaction::core::UIQuery;
//!
//! // Create automation instance
//! let automation = UIAutomationFactory::new()?;
//!
//! // Get the active window
//! let window = automation.get_active_window()?;
//! println!("Active window: {}", window.get_title()?);
//!
//! // Find and click a button
//! let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
//! if let Some(button) = buttons.first() {
//!     button.click()?;
//! }
//!
//! // Find a text field and set its content
//! let text_fields = window.find_elements(&UIQuery::ByType("Edit".to_string()))?;
//! if let Some(text_field) = text_fields.first() {
//!     text_field.set_text("Hello, World!")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Application Management
//!
//! ```rust
//! use uia_interaction::factory::ApplicationManagerFactory;
//!
//! // Create application manager
//! let app_manager = ApplicationManagerFactory::new()?;
//!
//! // Find all running applications
//! let apps = app_manager.get_all_applications()?;
//! println!("Found {} applications", apps.len());
//!
//! // Find specific applications
//! let notepad_apps = app_manager.find_applications_by_name("notepad.exe")?;
//! if let Some(notepad) = notepad_apps.first() {
//!     // Get the window for interaction
//!     let window = app_manager.get_window_by_process_id(notepad.process_id)?;
//!     window.activate()?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Python Integration
//!
//! This library includes Python bindings that provide the same functionality:
//!
//! ```python
//! from uia_interaction import PyAutomation, PyApplicationManager, PyUIQuery
//!
//! # Create automation instances
//! automation = PyAutomation()
//! app_manager = PyApplicationManager()
//!
//! # Find and interact with applications
//! apps = app_manager.find_applications_by_name("notepad.exe")
//! if apps:
//!     window = app_manager.get_window_by_process_id(apps[0].process_id)
//!     window.activate()
//!     
//!     # Find text area and set content
//!     text_areas = window.find_elements(PyUIQuery.by_type("Edit"))
//!     if text_areas:
//!         text_areas[0].set_text("Hello from Python!")
//! ```
//!
//! ## Platform Support
//!
//! ### Windows
//! - Full support via Windows UI Automation API
//! - All features available
//! - Tested on Windows 10/11
//!
//! ### Linux
//! - Planned support via AT-SPI
//! - Implementation in progress
//!
//! ### macOS
//! - Not yet supported
//! - May be added in future versions
//!
//! ## Modules
//!
//! - [`core`] - Core traits and types for UI automation
//! - [`factory`] - Platform-specific factory functions
//! - [`platform`] - Platform-specific implementations

// Re-export commonly used types for easy access
pub mod core;
pub mod platform;
pub mod factory;

// Re-export the main public API
pub use core::{UIAutomation, Window, UIElement, ApplicationManager, ApplicationInfo, UIQuery, UITree, UITreeNode, AppendPosition, Rect, TextElementInfo, SelectedTextInfo, TextExtractionOptions};
pub use factory::{UIAutomationFactory, ApplicationManagerFactory};

// Re-export platform-specific types for advanced usage
#[cfg(target_os = "windows")]
pub use platform::windows::{WindowsUIAutomation, WindowsWindow, WindowsElement, WindowsApplicationManager};

#[cfg(target_os = "linux")]
pub use platform::linux::{LinuxUIAutomation, LinuxWindow, LinuxUIElement};

use std::error::Error;

/// Create a UI automation instance for the current platform
pub fn create_automation() -> Result<Box<dyn UIAutomation>, Box<dyn Error>> {
    UIAutomationFactory::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::AppendPosition;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_character_encoding_detection() {
        // Test our character detection logic
        let test_cases = vec![
            ("Hello World", false), // ASCII only
            ("Demain, réunion de famille", true), // Contains é
            ("ça va être chiant", true), // Contains ç, à, ê
            ("Test 123", false), // ASCII only
            ("Côte d'Azur", true), // Contains ô
            ("naïve café", true), // Contains ï, é
        ];

        for (text, expected_has_special) in test_cases {
            let has_special = text.chars().any(|c| !c.is_ascii());
            assert_eq!(has_special, expected_has_special, 
                "Text '{}' special character detection failed", text);
        }
    }

    #[test] 
    #[ignore] // This test requires manual setup and interaction
    fn test_special_character_input() {
        // This test needs to be run manually with a text field focused
        let automation = create_automation().expect("Failed to create automation");
        
        let test_text = "Demain, réunion de famille. Ambiance pourrie, ça va être chiant.";
        println!("Testing special character input: '{}'", test_text);
        
        // Wait for user to focus on a text field
        println!("Focus on a text input field and press Enter in the console...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        
        let focused_element = automation.get_focused_element()
            .expect("Failed to get focused element");
        
        // Clear the field
        focused_element.set_text("")
            .expect("Failed to clear text");
        
        // Set text with special characters
        focused_element.set_text(test_text)
            .expect("Failed to set text");
        
        // Wait a bit for the text to be processed
        thread::sleep(Duration::from_millis(500));
        
        // Get the text back
        let result_text = focused_element.get_text()
            .expect("Failed to get text");
        
        println!("Expected: '{}'", test_text);
        println!("Got:      '{}'", result_text);
        
        // Check character by character
        let expected_chars: Vec<char> = test_text.chars().collect();
        let result_chars: Vec<char> = result_text.chars().collect();
        
        for (i, (expected, actual)) in expected_chars.iter().zip(result_chars.iter()).enumerate() {
            if expected != actual {
                println!("Character {} differs: expected '{}' (U+{:04X}), got '{}' (U+{:04X})", 
                    i, expected, *expected as u32, actual, *actual as u32);
            }
        }
        
        assert_eq!(test_text, result_text, "Character encoding test failed");
    }

    #[test]
    #[ignore] // This test requires manual setup and interaction  
    fn test_append_special_characters() {
        let automation = create_automation().expect("Failed to create automation");
        
        let initial_text = "Initial text: ";
        let append_text = "réunion été";
        let expected_final = format!("{}{}", initial_text, append_text);
        
        println!("Testing append with special characters");
        println!("Focus on a text input field and press Enter in the console...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        
        let focused_element = automation.get_focused_element()
            .expect("Failed to get focused element");
        
        // Set initial text
        focused_element.set_text(initial_text)
            .expect("Failed to set initial text");
        
        // Append text with special characters
        focused_element.append_text(append_text, AppendPosition::EndOfText)
            .expect("Failed to append text");
        
        thread::sleep(Duration::from_millis(500));
        
        let result_text = focused_element.get_text()
            .expect("Failed to get final text");
        
        println!("Expected: '{}'", expected_final);
        println!("Got:      '{}'", result_text);
        
        assert_eq!(expected_final, result_text, "Append special characters test failed");
    }
}