use std::error::Error;
use windows::Win32::Foundation::RECT;

/// Represents a UI element in the application
pub trait UIElement {
    /// Get the name of the element
    fn get_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the type of the element
    fn get_type(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the text content of the element
    fn get_text(&self) -> Result<String, Box<dyn Error>>;
    
    /// Set the text content of the element
    fn set_text(&self, text: &str) -> Result<(), Box<dyn Error>>;
    
    /// Append text to the element's content
    fn append_text(&self, text: &str) -> Result<(), Box<dyn Error>>;
}

/// Represents a window in the application
pub trait Window {
    /// Get the window title
    fn get_title(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the window class name
    fn get_class_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the window's process ID
    fn get_process_id(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Get the window's thread ID
    fn get_thread_id(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Get the window's process name
    fn get_process_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the window's process path
    fn get_process_path(&self) -> Result<String, Box<dyn Error>>;
    
    /// Check if the window is visible
    fn is_visible(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Check if the window is minimized
    fn is_minimized(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Check if the window is maximized
    fn is_maximized(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Get the window's geometry (position and size)
    fn get_rect(&self) -> Result<RECT, Box<dyn Error>>;
    
    /// Get the window's DPI
    fn get_dpi(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Get the currently focused element in this window
    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
}

/// Main interface for UI automation
pub trait UIAutomation {
    /// Get the currently focused window
    fn get_focused_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>>;
    
    /// Get the currently focused element
    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
    
    /// Find an element by its name
    fn find_element_by_name(&self, name: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
    
    /// Find an element by its type
    fn find_element_by_type(&self, element_type: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
} 