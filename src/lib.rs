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
pub use core::{UIAutomation, Window, UIElement, ApplicationManager, ApplicationInfo, UIQuery, UITree, UITreeNode, AppendPosition, Rect};
pub use factory::{UIAutomationFactory, ApplicationManagerFactory};

// Re-export platform-specific types for advanced usage
#[cfg(target_os = "windows")]
pub use platform::windows::{WindowsUIAutomation, WindowsWindow, WindowsElement, WindowsApplicationManager};

#[cfg(target_os = "linux")]
pub use platform::linux::{LinuxUIAutomation, LinuxWindow, LinuxUIElement};