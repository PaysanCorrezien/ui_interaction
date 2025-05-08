use std::error::Error;
use crate::core::UIAutomation;

#[cfg(target_os = "windows")]
use crate::platform::windows::WindowsUIAutomation;

#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxUIAutomation;

/// Factory for creating UI automation instances
pub struct UIAutomationFactory;

impl UIAutomationFactory {
    /// Create a new UI automation instance for the current platform
    pub fn new() -> Result<Box<dyn UIAutomation>, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(WindowsUIAutomation::new()?))
        }
        
        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(LinuxUIAutomation::new()?))
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Unsupported platform".into())
        }
    }
} 