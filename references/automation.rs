use log::debug;
use uiautomation::core::UIAutomation as UIAutomationCore;
use std::error::Error;

/// Core UIAutomation functionality
pub struct UIAutomation {
    automation: UIAutomationCore,
}

impl UIAutomation {
    /// Create a new instance of UIAutomation
    pub fn new() -> Result<Self, Box<dyn Error>> {
        debug!("Initializing UIAutomation instance");
        let automation = UIAutomationCore::new()?;
        Ok(UIAutomation { automation })
    }

    /// Get the underlying UIAutomationCore instance
    pub fn core(&self) -> &UIAutomationCore {
        &self.automation
    }
}

/// Legacy helper struct for backward compatibility
/// This will be deprecated in future versions
pub struct UiautomationHelper {
    automation: UIAutomationCore,
}

impl UiautomationHelper {
    /// Create a new instance of UiautomationHelper
    pub fn new() -> Result<Self, Box<dyn Error>> {
        debug!("Initializing UiautomationHelper instance");
        let automation = UIAutomationCore::new()?;
        Ok(UiautomationHelper { automation })
    }

    /// Get the underlying UIAutomationCore instance
    pub fn core(&self) -> &UIAutomationCore {
        &self.automation
    }
}
