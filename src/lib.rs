//! UI Automation library for Windows
//! 
//! This library provides a high-level interface to Windows UI Automation,
//! making it easier to interact with UI elements programmatically.

// Re-export commonly used types
pub mod core;
pub mod platform;
pub mod factory;

pub use core::{UIAutomation, Window, UIElement};
pub use factory::UIAutomationFactory;

// Re-export platform-specific types for advanced usage
#[cfg(target_os = "windows")]
pub use platform::windows::{WindowsUIAutomation, WindowsWindow, WindowsElement};

#[cfg(target_os = "linux")]
pub use platform::linux::{LinuxUIAutomation, LinuxWindow, LinuxUIElement};
