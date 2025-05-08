use log::{debug, info, warn};
use uiautomation::controls::ControlType;
use uiautomation::core::UIElement;
use uiautomation::patterns::{UIValuePattern, UITextPattern};
use uiautomation::types::UIProperty;
use uiautomation::variants::Variant;
use std::error::Error;

use crate::automation::UIAutomation;

/// Element finding and manipulation functionality
pub struct ElementFinder<'a> {
    pub automation: &'a UIAutomation,
}

impl<'a> ElementFinder<'a> {
    /// Create a new ElementFinder instance
    pub fn new(automation: &'a UIAutomation) -> Self {
        ElementFinder { automation }
    }

    /// Get the currently focused element and its type
    pub fn get_focused_element(&self) -> Result<(UIElement, String, String), Box<dyn Error>> {
        debug!("Getting currently focused element");

        // Direct method to get focused element
        let element = self.automation.core().get_focused_element()?;

        let name: Variant = element.get_property_value(UIProperty::Name)?;
        let control_type: Variant = element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        let control_type_enum = ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom);

        info!(
            "Found focused element: Name: {}, Type: {:?}",
            name.get_string()?,
            control_type_enum
        );

        Ok((element, name.get_string()?, format!("{:?}", control_type_enum)))
    }

    /// Get the currently focused input element
    pub fn get_focused_input_element(&self) -> Result<UIElement, Box<dyn Error>> {
        debug!("Getting currently focused input element");
        
        // Get the currently focused element
        let focused_element = self.automation.core().get_focused_element()?;
        
        // Get the control type
        let control_type: Variant = focused_element.get_property_value(UIProperty::ControlType)?;
        let control_type_id: i32 = control_type.try_into()?;
        
        // Log the element's properties for debugging
        let name: Variant = focused_element.get_property_value(UIProperty::Name)?;
        let class_name = focused_element.get_classname()?;
        debug!(
            "Focused element details - Name: {}, Type: {:?}, ClassName: {}",
            name.get_string()?,
            ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom),
            class_name
        );
        
        // Check if it's an input control type using the ControlType enum directly
        let is_input = match ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom) {
            ControlType::Edit => true,
            ControlType::ComboBox => true,
            ControlType::CheckBox => true,
            ControlType::RadioButton => true,
            ControlType::Slider => true,
            ControlType::Text => true,  // For web text elements
            ControlType::Document => true,  // For web document elements
            ControlType::Pane => {
                // For Pane elements, check if they support text input patterns
                focused_element.get_pattern::<UIValuePattern>().is_ok() ||
                focused_element.get_pattern::<UITextPattern>().is_ok()
            },
            ControlType::Custom => {
                // For custom controls, check if it supports value pattern
                focused_element.get_pattern::<UIValuePattern>().is_ok()
            },
            _ => false,
        };
        
        if is_input {
            info!("Found focused input element with type: {:?}", ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom));
            Ok(focused_element)
        } else {
            warn!("Focused element is not an input control (Type: {:?})", ControlType::try_from(control_type_id).unwrap_or(ControlType::Custom));
            Err("Focused element is not an input control".into())
        }
    }
}
