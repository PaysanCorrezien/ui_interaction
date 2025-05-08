//TODO: Implement AppContext
// this should be able to :
// - get the generaal app context ( window.rs provide it) : DONE
// - get the current focused element, its text: DONE
// - Get the surrounding elements of the focused element to get more context of what the user want
// This is the hard part , because the UIA context is not that easy to get because it realy varies from app to app
// - generate a structured marddown report from it with all the infos ( to be passed to LLM)
// - reuse the previous tree capture as cache, and treewalk back with custom filtering ? to get just `parent` and `siblings`
// - refine this filtering on lot of apps to test

use crate::elements::ElementFinder;
use crate::text::TextHandler;
use uiautomation::core::UIElement;
use uiautomation::types::UIProperty;
use std::error::Error;

/// Represents the application context around a focused element
pub struct AppContext {
    pub focused_element: UIElement,
    pub parent: Option<UIElement>,
    pub siblings: Vec<UIElement>,
    pub nearby_elements: Vec<UIElement>,
}

impl AppContext {
    /// Creates a new AppContext instance by analyzing the current focused element
    pub fn new(element_finder: &ElementFinder, text_handler: &TextHandler) -> Result<Self, Box<dyn Error>> {
        // Get the focused element
        let (focused_element, _, _) = element_finder.get_focused_element()?;
        
        // Get parent element using the control view walker
        let walker = element_finder.automation.core().get_control_view_walker()?;
        let parent = match walker.get_parent(&focused_element) {
            Ok(p) => Some(p),
            Err(_) => None,
        };

        // Get siblings
        let siblings = if let Some(p) = &parent {
            match Self::get_siblings(&focused_element, p, &walker) {
                Ok(s) => s,
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Get nearby elements (within a certain distance)
        let nearby_elements = Self::get_nearby_elements(&focused_element, element_finder)?;

        Ok(Self {
            focused_element,
            parent,
            siblings,
            nearby_elements,
        })
    }

    /// Gets all siblings of the focused element
    fn get_siblings(
        focused: &UIElement,
        parent: &UIElement,
        walker: &uiautomation::core::UITreeWalker,
    ) -> Result<Vec<UIElement>, Box<dyn Error>> {
        let mut siblings = Vec::new();
        
        // Get first child
        if let Ok(mut current) = walker.get_first_child(parent) {
            loop {
                // Skip the focused element itself
                if current.get_runtime_id()? != focused.get_runtime_id()? {
                    siblings.push(current.clone());
                }
                
                // Get next sibling
                match walker.get_next_sibling(&current) {
                    Ok(next) => current = next,
                    Err(_) => break,
                }
            }
        }
        
        Ok(siblings)
    }

    /// Gets elements that are nearby the focused element in the UI tree
    fn get_nearby_elements(
        focused: &UIElement,
        element_finder: &ElementFinder,
    ) -> Result<Vec<UIElement>, Box<dyn Error>> {
        let mut nearby = Vec::new();
        let walker = element_finder.automation.core().get_control_view_walker()?;
        
        // Get parent
        if let Ok(parent) = walker.get_parent(focused) {
            // Get grandparent
            if let Ok(grandparent) = walker.get_parent(&parent) {
                // Get great-grandparent if it exists
                if let Ok(great_grandparent) = walker.get_parent(&grandparent) {
                    nearby.push(great_grandparent);
                }
                nearby.push(grandparent.clone());
                
                // Get aunts/uncles (siblings of parent)
                if let Ok(mut uncle) = walker.get_first_child(&grandparent) {
                    loop {
                        if uncle.get_runtime_id()? != parent.get_runtime_id()? {
                            nearby.push(uncle.clone());
                            
                            // Get cousins (children of aunts/uncles)
                            if let Ok(mut cousin) = walker.get_first_child(&uncle) {
                                loop {
                                    nearby.push(cousin.clone());
                                    match walker.get_next_sibling(&cousin) {
                                        Ok(next) => cousin = next,
                                        Err(_) => break,
                                    }
                                }
                            }
                        }
                        match walker.get_next_sibling(&uncle) {
                            Ok(next) => uncle = next,
                            Err(_) => break,
                        }
                    }
                }
            }
            nearby.push(parent);
        }
        
        Ok(nearby)
    }

    /// Helper function to collect all elements in the tree
    fn collect_elements(
        element: &UIElement,
        walker: &uiautomation::core::UITreeWalker,
        elements: &mut Vec<UIElement>,
    ) -> Result<(), Box<dyn Error>> {
        elements.push(element.clone());
        
        // Get first child
        if let Ok(mut current) = walker.get_first_child(element) {
            loop {
                Self::collect_elements(&current, walker, elements)?;
                
                // Get next sibling
                match walker.get_next_sibling(&current) {
                    Ok(next) => current = next,
                    Err(_) => break,
                }
            }
        }
        
        Ok(())
    }

    /// Generates a structured report of the app context
    pub fn generate_report(&self, text_handler: &TextHandler) -> String {
        let mut report = String::new();
        
        // Add focused element text
        if let Ok(name) = self.focused_element.get_property_value(UIProperty::Name) {
            if let Ok(name_str) = name.get_string() {
                if !name_str.is_empty() {
                    report.push_str(&format!("## Focused Element\n{}\n", name_str));
                }
            }
        }
        
        // Add parent text
        if let Some(parent) = &self.parent {
            if let Ok(name) = parent.get_property_value(UIProperty::Name) {
                if let Ok(name_str) = name.get_string() {
                    if !name_str.is_empty() {
                        report.push_str(&format!("## Parent Context\n{}\n", name_str));
                    }
                }
            }
        }
        
        // Add siblings text
        if !self.siblings.is_empty() {
            report.push_str("## Sibling Context\n");
            for sibling in &self.siblings {
                if let Ok(name) = sibling.get_property_value(UIProperty::Name) {
                    if let Ok(name_str) = name.get_string() {
                        if !name_str.is_empty() {
                            report.push_str(&format!("- {}\n", name_str));
                        }
                    }
                }
            }
        }
        
        // Add nearby elements text
        if !self.nearby_elements.is_empty() {
            report.push_str("## Nearby Context\n");
            for element in &self.nearby_elements {
                if let Ok(name) = element.get_property_value(UIProperty::Name) {
                    if let Ok(name_str) = name.get_string() {
                        if !name_str.is_empty() {
                            report.push_str(&format!("- {}\n", name_str));
                        }
                    }
                }
            }
        }
        
        report
    }
}
