use std::error::Error;
use crate::core::{UIAutomation, ApplicationManager};

#[cfg(target_os = "windows")]
use crate::platform::windows::{WindowsUIAutomation, WindowsApplicationManager};

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

/// Factory for creating application manager instances
pub struct ApplicationManagerFactory;

impl ApplicationManagerFactory {
    /// Create a new application manager instance for the current platform
    pub fn new() -> Result<Box<dyn ApplicationManager>, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(WindowsApplicationManager::new()?))
        }
        
        #[cfg(target_os = "linux")]
        {
            Err("Linux application manager not implemented yet".into())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Unsupported platform".into())
        }
    }
} 