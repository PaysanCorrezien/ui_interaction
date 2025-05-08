//! UI Automation library for Windows
//! 
//! This library provides a high-level interface to Windows UI Automation,
//! making it easier to interact with UI elements programmatically.

pub mod automation;
pub mod elements;
pub mod text;
pub mod windows;
pub mod app_context;

// Re-export commonly used types
pub use automation::UIAutomation;
pub use elements::ElementFinder;
pub use text::TextHandler;
pub use windows::WindowManager;
pub use app_context::AppContext;

// Re-export the main helper for backward compatibility
pub use automation::UiautomationHelper;
