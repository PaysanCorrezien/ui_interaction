//! Factory module for creating platform-specific UI automation instances
//!
//! This module provides factory functions to create UI automation and application
//! manager instances appropriate for the current platform. It handles the
//! platform-specific implementations transparently.
//!
//! # Supported Platforms
//!
//! - **Windows**: Full support via Windows UI Automation API
//! - **Linux**: Partial support (planned)
//! - **macOS**: Not yet supported
//!
//! # Example
//!
//! ```rust
//! use uia_interaction::factory::{UIAutomationFactory, ApplicationManagerFactory};
//! use uia_interaction::core::UIQuery;
//!
//! // Create automation instances
//! let automation = UIAutomationFactory::new()?;
//! let app_manager = ApplicationManagerFactory::new()?;
//!
//! // Use the instances for UI automation
//! let window = automation.get_active_window()?;
//! let apps = app_manager.get_all_applications()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use crate::core::{UIAutomation, ApplicationManager};

#[cfg(target_os = "windows")]
use crate::platform::windows::{WindowsUIAutomation, WindowsApplicationManager};

#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxUIAutomation;

/// Factory for creating platform-specific UI automation instances
///
/// This factory provides a cross-platform way to create UI automation instances.
/// It automatically selects the appropriate implementation based on the current
/// operating system.
///
/// # Platform Support
///
/// - **Windows**: Uses Windows UI Automation API for full functionality
/// - **Linux**: Uses AT-SPI (Assistive Technology Service Provider Interface) - planned
/// - **Other platforms**: Currently unsupported
///
/// # Example
///
/// ```rust
/// use uia_interaction::factory::UIAutomationFactory;
/// use uia_interaction::core::UIQuery;
///
/// // Create automation instance (works on any supported platform)
/// let automation = UIAutomationFactory::new()?;
///
/// // Get the active window and interact with it
/// let window = automation.get_active_window()?;
/// println!("Active window: {}", window.get_title()?);
///
/// // Find and click a button
/// let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
/// if let Some(button) = buttons.first() {
///     println!("Clicking button: {}", button.get_name()?);
///     button.click()?;
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct UIAutomationFactory;

impl UIAutomationFactory {
    /// Create a new UI automation instance for the current platform
    ///
    /// This method detects the current operating system and returns the
    /// appropriate UI automation implementation.
    ///
    /// # Returns
    ///
    /// * `Ok(Box<dyn UIAutomation>)` - Platform-specific UI automation instance
    /// * `Err(...)` - If the platform is unsupported or initialization fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The current platform is not supported
    /// - The platform-specific initialization fails
    /// - Required system services are not available
    ///
    /// # Example
    ///
    /// ```rust
    /// use uia_interaction::factory::UIAutomationFactory;
    ///
    /// match UIAutomationFactory::new() {
    ///     Ok(automation) => {
    ///         // Use automation for UI operations
    ///         let window = automation.get_active_window()?;
    ///         println!("Success! Active window: {}", window.get_title()?);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to create automation: {}", e);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
            Err("Unsupported platform: UI automation is currently only supported on Windows and Linux".into())
        }
    }
}

/// Factory for creating platform-specific application manager instances
///
/// This factory provides a cross-platform way to create application manager instances
/// for discovering and managing running applications. It automatically selects the
/// appropriate implementation based on the current operating system.
///
/// # Platform Support
///
/// - **Windows**: Full support using Windows APIs
/// - **Linux**: Planned support using standard Linux process management
/// - **Other platforms**: Currently unsupported
///
/// # Example
///
/// ```rust
/// use uia_interaction::factory::ApplicationManagerFactory;
///
/// // Create application manager (works on any supported platform)
/// let app_manager = ApplicationManagerFactory::new()?;
///
/// // Find all running applications
/// let apps = app_manager.get_all_applications()?;
/// println!("Found {} applications", apps.len());
///
/// // Find specific applications
/// let text_editors = app_manager.find_applications_by_name("notepad")?;
/// for app in text_editors {
///     println!("Text editor: {} (PID: {})", app.main_window_title, app.process_id);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ApplicationManagerFactory;

impl ApplicationManagerFactory {
    /// Create a new application manager instance for the current platform
    ///
    /// This method detects the current operating system and returns the
    /// appropriate application manager implementation.
    ///
    /// # Returns
    ///
    /// * `Ok(Box<dyn ApplicationManager>)` - Platform-specific application manager
    /// * `Err(...)` - If the platform is unsupported or initialization fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The current platform is not supported
    /// - The platform-specific initialization fails
    /// - Required system permissions are not available
    ///
    /// # Example
    ///
    /// ```rust
    /// use uia_interaction::factory::ApplicationManagerFactory;
    ///
    /// match ApplicationManagerFactory::new() {
    ///     Ok(app_manager) => {
    ///         // Use app_manager for application discovery
    ///         let apps = app_manager.get_all_applications()?;
    ///         println!("Success! Found {} applications", apps.len());
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to create application manager: {}", e);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Box<dyn ApplicationManager>, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(WindowsApplicationManager::new()?))
        }
        
        #[cfg(target_os = "linux")]
        {
            Err("Linux application manager not implemented yet. This feature is planned for a future release.".into())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Unsupported platform: Application management is currently only supported on Windows".into())
        }
    }
} 