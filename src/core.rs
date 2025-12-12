//! Core traits and types for UI automation
//!
//! This module defines the fundamental abstractions for UI automation across different platforms.
//! It provides traits for interacting with UI elements, windows, and applications, as well as
//! query systems for finding specific elements.
//!
//! # Overview
//!
//! The core types include:
//! - [`UIElement`] - Represents a UI element that can be interacted with
//! - [`Window`] - Represents a window containing UI elements  
//! - [`UIAutomation`] - Main interface for UI automation operations
//! - [`ApplicationManager`] - Interface for discovering and managing applications
//! - [`UIQuery`] - Query builder for finding specific UI elements
//!
//! # Example
//!
//! ```rust
//! use uia_interaction::factory::UIAutomationFactory;
//! use uia_interaction::core::{UIQuery, AppendPosition};
//!
//! // Create automation instance
//! let automation = UIAutomationFactory::new()?;
//!
//! // Get the active window
//! let window = automation.get_active_window()?;
//!
//! // Find a text input field and set text
//! let query = UIQuery::ByType("Edit".to_string());
//! let elements = window.find_elements(&query)?;
//! if let Some(element) = elements.first() {
//!     element.set_text("Hello, World!")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use windows::Win32::Foundation::RECT;
use std::any::Any;
use serde::{Serialize, Deserialize};

/// Represents a rectangle in screen coordinates
/// 
/// This structure defines a rectangular area using left, top, right, and bottom coordinates
/// in screen pixel coordinates. It's commonly used for element bounds and window positioning.
/// 
/// # Fields
/// 
/// * `left` - The x-coordinate of the left edge
/// * `top` - The y-coordinate of the top edge  
/// * `right` - The x-coordinate of the right edge
/// * `bottom` - The y-coordinate of the bottom edge
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::core::Rect;
/// 
/// let rect = Rect {
///     left: 100,
///     top: 50,
///     right: 300,
///     bottom: 200,
/// };
/// 
/// let width = rect.right - rect.left;   // 200
/// let height = rect.bottom - rect.top;  // 150
/// ```
#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// Create a new Rect from coordinates
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Rect { left, top, right, bottom }
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// Get the center point of the rectangle
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }

    /// Check if a point is inside this rectangle
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left < other.right && self.right > other.left &&
        self.top < other.bottom && self.bottom > other.top
    }
}

/// Structured information about a UI element that contains text
///
/// This struct captures comprehensive information about text-containing UI elements,
/// designed to make it easy to identify, locate, and extract text from applications.
///
/// # Fields
///
/// * `text` - The actual text content of the element
/// * `name` - The accessible name/label of the element
/// * `control_type` - The type of UI control (e.g., "Text", "Edit", "Document")
/// * `automation_id` - The automation ID (if available) for reliable element identification
/// * `class_name` - The window class name of the element
/// * `bounds` - Screen position and size of the element
/// * `is_selected` - Whether this text is currently selected
/// * `is_editable` - Whether the text can be edited
/// * `is_visible` - Whether the element is visible on screen
/// * `is_enabled` - Whether the element is enabled for interaction
/// * `parent_name` - Name of the parent element (useful for context)
/// * `depth` - Depth in the UI tree (0 = root)
///
/// # Example
///
/// ```rust
/// use uia_interaction::core::TextElementInfo;
///
/// // Process text elements from a window
/// let text_elements: Vec<TextElementInfo> = get_text_elements(window)?;
///
/// for elem in text_elements {
///     if let Some(bounds) = &elem.bounds {
///         println!("Text: '{}' at ({}, {}) - {}x{}",
///             elem.text, bounds.left, bounds.top,
///             bounds.width(), bounds.height());
///     }
///     if elem.is_selected {
///         println!("  -> Currently selected!");
///     }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextElementInfo {
    /// The text content of the element
    pub text: String,
    /// The accessible name/label of the element
    pub name: String,
    /// The control type (e.g., "Text", "Edit", "Document", "Button")
    pub control_type: String,
    /// Automation ID for reliable element identification (if available)
    pub automation_id: Option<String>,
    /// Window class name of the element
    pub class_name: Option<String>,
    /// Screen bounds of the element
    pub bounds: Option<Rect>,
    /// Whether this text is currently selected
    pub is_selected: bool,
    /// Whether the text can be edited
    pub is_editable: bool,
    /// Whether the element is visible on screen
    pub is_visible: bool,
    /// Whether the element is enabled for interaction
    pub is_enabled: bool,
    /// Name of the parent element for context
    pub parent_name: Option<String>,
    /// Depth in the UI tree (0 = root)
    pub depth: u32,
}

impl TextElementInfo {
    /// Create a new TextElementInfo with default values
    pub fn new(text: String) -> Self {
        TextElementInfo {
            text,
            name: String::new(),
            control_type: String::new(),
            automation_id: None,
            class_name: None,
            bounds: None,
            is_selected: false,
            is_editable: false,
            is_visible: true,
            is_enabled: true,
            parent_name: None,
            depth: 0,
        }
    }

    /// Check if this element has meaningful text content
    pub fn has_text(&self) -> bool {
        !self.text.trim().is_empty()
    }

    /// Check if the element is on screen (visible and has bounds)
    pub fn is_on_screen(&self) -> bool {
        self.is_visible && self.bounds.is_some()
    }
}

/// Information about selected text in a UI element
///
/// This struct captures details about text that is currently selected
/// (highlighted) by the user in a UI element.
///
/// # Fields
///
/// * `text` - The selected text content
/// * `start_offset` - Character offset where selection starts
/// * `end_offset` - Character offset where selection ends
/// * `element_info` - Information about the element containing the selection
///
/// # Example
///
/// ```rust
/// use uia_interaction::core::SelectedTextInfo;
///
/// // Get selected text from the focused element
/// if let Some(selection) = get_selected_text(automation)? {
///     println!("Selected: '{}' (chars {}-{})",
///         selection.text,
///         selection.start_offset,
///         selection.end_offset);
///     if let Some(bounds) = &selection.bounds {
///         println!("Selection at ({}, {})", bounds.left, bounds.top);
///     }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectedTextInfo {
    /// The selected text content
    pub text: String,
    /// Character offset where selection starts
    pub start_offset: i32,
    /// Character offset where selection ends
    pub end_offset: i32,
    /// Bounding rectangle of the selected text (if available)
    pub bounds: Option<Rect>,
    /// Information about the element containing the selection
    pub element_info: Option<TextElementInfo>,
}

impl SelectedTextInfo {
    /// Create a new SelectedTextInfo
    pub fn new(text: String, start_offset: i32, end_offset: i32) -> Self {
        SelectedTextInfo {
            text,
            start_offset,
            end_offset,
            bounds: None,
            element_info: None,
        }
    }

    /// Get the length of the selection
    pub fn selection_length(&self) -> i32 {
        self.end_offset - self.start_offset
    }
}

/// Represents a node in the UI tree hierarchy
/// 
/// A UI tree node captures the state and properties of a UI element at a specific
/// point in time, forming a hierarchical tree structure that represents the complete
/// UI layout of a window or application.
/// 
/// # Fields
/// 
/// * `name` - The accessible name or label of the element
/// * `control_type` - The type of control (e.g., "Button", "Edit", "Document")
/// * `properties` - Key-value pairs of all element properties
/// * `children` - Child nodes in the UI hierarchy
/// * `bounds` - Optional rectangle defining the element's screen position and size
/// * `is_enabled` - Whether the element is enabled for interaction
/// * `is_visible` - Whether the element is visible on screen
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::core::UITreeNode;
/// use std::collections::HashMap;
/// 
/// // Accessing node information
/// let process_tree_node = |node: &UITreeNode| {
///     println!("Element: {} ({})", node.name, node.control_type);
///     if node.is_enabled && node.is_visible {
///         println!("  Interactive element found");
///     }
///     
///     // Process all children recursively
///     for child in &node.children {
///         process_tree_node(child);
///     }
/// };
/// ```
#[allow(dead_code)]
#[derive(Clone)]
pub struct UITreeNode {
    pub name: String,
    pub control_type: String,
    pub properties: HashMap<String, String>,
    pub children: Vec<UITreeNode>,
    pub bounds: Option<Rect>,
    pub is_enabled: bool,
    pub is_visible: bool,
}

/// Represents a complete UI tree snapshot
/// 
/// A UI tree provides a hierarchical snapshot of all UI elements within a window
/// at a specific point in time. This is useful for debugging, analysis, and 
/// understanding the structure of an application's user interface.
/// 
/// # Fields
/// 
/// * `root` - The root node of the UI hierarchy (typically the window itself)
/// * `timestamp` - When this snapshot was captured
/// * `window_title` - The title of the window when captured
/// * `window_class` - The window class name when captured
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::UIAutomationFactory;
/// 
/// let automation = UIAutomationFactory::new()?;
/// let window = automation.get_active_window()?;
/// let tree = window.get_ui_tree()?;
/// 
/// println!("Window: {} ({})", tree.window_title, tree.window_class);
/// println!("Captured at: {}", tree.timestamp);
/// println!("Root element: {} ({})", tree.root.name, tree.root.control_type);
/// println!("Child count: {}", tree.root.children.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
#[derive(Clone)]
pub struct UITree {
    pub root: UITreeNode,
    pub timestamp: DateTime<Utc>,
    pub window_title: String,
    pub window_class: String,
}

/// Specifies where to append text when using text append operations
/// 
/// This enum defines the different positions where text can be appended
/// to existing text content in UI elements.
/// 
/// # Variants
/// 
/// * `CurrentCursor` - Append at the current cursor/caret position
/// * `EndOfLine` - Append at the end of the current line
/// * `EndOfText` - Append at the very end of all text content
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::core::AppendPosition;
/// use uia_interaction::factory::UIAutomationFactory;
/// use uia_interaction::core::UIQuery;
/// 
/// let automation = UIAutomationFactory::new()?;
/// let window = automation.get_active_window()?;
/// 
/// // Find a text field and append text in different positions
/// let query = UIQuery::ByType("Edit".to_string());
/// if let Ok(elements) = window.find_elements(&query) {
///     if let Some(element) = elements.first() {
///         element.append_text(" (end of text)", AppendPosition::EndOfText)?;
///         element.append_text(" [EOL]", AppendPosition::EndOfLine)?;
///         element.append_text(" |cursor|", AppendPosition::CurrentCursor)?;
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub enum AppendPosition {
    CurrentCursor,
    EndOfLine,
    EndOfText,
}

/// Trait for interacting with UI elements
/// 
/// This trait defines the interface for interacting with UI elements such as buttons,
/// text fields, menus, and other controls in desktop applications. It provides methods
/// for getting element information, setting content, and performing actions.
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::UIAutomationFactory;
/// use uia_interaction::core::{UIQuery, AppendPosition};
/// 
/// let automation = UIAutomationFactory::new()?;
/// let window = automation.get_active_window()?;
/// 
/// // Find a button and click it
/// let button_query = UIQuery::ByType("Button".to_string());
/// if let Ok(elements) = window.find_elements(&button_query) {
///     if let Some(button) = elements.first() {
///         println!("Found button: {}", button.get_name()?);
///         if button.is_enabled()? {
///             button.click()?;
///         }
///     }
/// }
/// 
/// // Find a text field and set content
/// let edit_query = UIQuery::ByType("Edit".to_string());
/// if let Ok(elements) = window.find_elements(&edit_query) {
///     if let Some(text_field) = elements.first() {
///         text_field.set_text("Hello, World!")?;
///         text_field.append_text(" - Additional text", AppendPosition::EndOfText)?;
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
pub trait UIElement {
    /// Get the accessible name or label of the element
    /// 
    /// Returns the name/label text that identifies this element to users and
    /// screen readers. This is often the button text, field label, or similar.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The element's name/label
    /// * `Err(...)` - If the name cannot be retrieved
    fn get_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the control type of the element
    /// 
    /// Returns the type of UI control (e.g., "Button", "Edit", "Document", "Menu").
    /// This helps identify what kind of element you're working with.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The control type name
    /// * `Err(...)` - If the type cannot be determined
    fn get_type(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the text content of the element
    /// 
    /// Returns the current text content for elements that contain text,
    /// such as text fields, labels, or documents.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The current text content
    /// * `Err(...)` - If text cannot be retrieved or element doesn't support text
    fn get_text(&self) -> Result<String, Box<dyn Error>>;
    
    /// Set the text content of the element
    /// 
    /// Replaces the entire text content of the element with the specified text.
    /// This works with text fields, documents, and other text-containing elements.
    /// 
    /// # Arguments
    /// 
    /// * `text` - The new text content to set
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If text was set successfully
    /// * `Err(...)` - If text cannot be set or element doesn't support text input
    fn set_text(&self, text: &str) -> Result<(), Box<dyn Error>>;
    
    /// Append text to the element's existing content
    /// 
    /// Adds text to the element at the specified position without replacing
    /// existing content. Useful for adding to existing text in fields or documents.
    /// 
    /// # Arguments
    /// 
    /// * `text` - The text to append
    /// * `position` - Where to append the text (cursor, end of line, end of text)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If text was appended successfully
    /// * `Err(...)` - If text cannot be appended or element doesn't support text input
    fn append_text(&self, text: &str, position: AppendPosition) -> Result<(), Box<dyn Error>>;

    /// Click the element
    /// 
    /// Performs a mouse click on the element. This works with buttons, links,
    /// and other clickable elements.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the element was clicked successfully
    /// * `Err(...)` - If the element cannot be clicked or is not accessible
    fn click(&self) -> Result<(), Box<dyn Error>>;

    /// Check if the element is enabled for interaction
    /// 
    /// Returns whether the element is currently enabled and can be interacted with.
    /// Disabled elements typically appear grayed out and don't respond to input.
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Element is enabled and interactive
    /// * `Ok(false)` - Element is disabled
    /// * `Err(...)` - If the enabled state cannot be determined
    fn is_enabled(&self) -> Result<bool, Box<dyn Error>>;

    /// Get all properties of the element
    /// 
    /// Returns a map of all available properties for this element, including
    /// both standard properties and any custom properties specific to the element.
    /// 
    /// # Returns
    /// 
    /// * `Ok(HashMap)` - Map of property names to values
    /// * `Err(...)` - If properties cannot be retrieved
    fn get_properties(&self) -> Result<HashMap<String, String>, Box<dyn Error>>;
    
    /// Get the screen bounds of the element
    /// 
    /// Returns the rectangular area occupied by the element on screen,
    /// or None if the element is not visible or bounds cannot be determined.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(Rect))` - The element's screen rectangle
    /// * `Ok(None)` - Element is not visible or has no bounds
    /// * `Err(...)` - If bounds cannot be retrieved
    fn get_bounds(&self) -> Result<Option<Rect>, Box<dyn Error>>;
    
    /// Get direct child elements
    /// 
    /// Returns all direct child elements in the UI hierarchy. This allows
    /// traversing the UI tree structure.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<Box<dyn UIElement>>)` - List of child elements
    /// * `Err(...)` - If children cannot be retrieved
    fn get_children(&self) -> Result<Vec<Box<dyn UIElement>>, Box<dyn Error>>;
    
    /// Convert element to tree node representation
    /// 
    /// Creates a tree node representation of this element for use in UI tree structures.
    /// This is primarily used internally for tree building operations.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn UIElement>)` - Tree node representation
    /// * `Err(...)` - If conversion fails
    fn to_tree_node(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>>;

    /// Get a reference to the underlying type for downcasting
    ///
    /// Provides access to the concrete type implementing this trait,
    /// enabling downcasting to platform-specific implementations when needed.
    ///
    /// # Returns
    ///
    /// Reference to the underlying type as `&dyn Any`
    fn as_any(&self) -> &dyn Any;

    /// Get structured information about this text element
    ///
    /// Extracts comprehensive information about this element including its text content,
    /// position, identifiers, and state. This is useful for building structured
    /// representations of UI text content.
    ///
    /// # Returns
    ///
    /// * `Ok(TextElementInfo)` - Structured element information
    /// * `Err(...)` - If element information cannot be retrieved
    ///
    /// # Example
    ///
    /// ```rust
    /// let element = window.get_focused_element()?;
    /// let info = element.get_text_element_info()?;
    /// println!("Text: '{}' at {:?}", info.text, info.bounds);
    /// ```
    fn get_text_element_info(&self) -> Result<TextElementInfo, Box<dyn Error>> {
        let text = self.get_text().unwrap_or_default();
        let name = self.get_name().unwrap_or_default();
        let control_type = self.get_type().unwrap_or_default();
        let bounds = self.get_bounds().unwrap_or(None);
        let is_enabled = self.is_enabled().unwrap_or(true);

        let props = self.get_properties().unwrap_or_default();
        let automation_id = props.get("automation_id").cloned();
        let class_name = props.get("class_name").cloned();

        // Determine if element is editable based on control type
        let is_editable = matches!(control_type.as_str(), "Edit" | "Document" | "ComboBox");

        Ok(TextElementInfo {
            text,
            name,
            control_type,
            automation_id,
            class_name,
            bounds,
            is_selected: false,
            is_editable,
            is_visible: true,
            is_enabled,
            parent_name: None,
            depth: 0,
        })
    }

    /// Get the currently selected text within this element
    ///
    /// Retrieves text that is currently highlighted/selected by the user within
    /// this element. Returns None if there is no selection or the element
    /// doesn't support text selection.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SelectedTextInfo))` - Information about the selected text
    /// * `Ok(None)` - No text is selected
    /// * `Err(...)` - If selection cannot be retrieved
    ///
    /// # Example
    ///
    /// ```rust
    /// let element = automation.get_focused_element()?;
    /// if let Some(selection) = element.get_selected_text()? {
    ///     println!("Selected: '{}'", selection.text);
    /// }
    /// ```
    fn get_selected_text(&self) -> Result<Option<SelectedTextInfo>, Box<dyn Error>> {
        // Default implementation returns None - platform-specific implementations can override
        Ok(None)
    }
}

/// Trait for interacting with application windows
/// 
/// This trait provides methods to interact with windows, get window information,
/// control window state, and find UI elements within the window. It represents
/// a top-level window or dialog in a desktop application.
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::UIAutomationFactory;
/// use uia_interaction::core::UIQuery;
/// 
/// let automation = UIAutomationFactory::new()?;
/// let window = automation.get_active_window()?;
/// 
/// // Get window information
/// println!("Window: {}", window.get_title()?);
/// println!("Process: {} (PID: {})", window.get_process_name()?, window.get_process_id()?);
/// 
/// // Activate the window to ensure it has focus
/// window.activate()?;
/// 
/// // Find all buttons in the window
/// let button_query = UIQuery::ByType("Button".to_string());
/// let buttons = window.find_elements(&button_query)?;
/// println!("Found {} buttons", buttons.len());
/// 
/// // Get the UI tree for analysis
/// let tree = window.get_ui_tree()?;
/// println!("UI tree captured at: {}", tree.timestamp);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
pub trait Window {
    /// Get the window title text
    /// 
    /// Returns the title bar text of the window, which is typically displayed
    /// in the window's title bar and taskbar.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The window title
    /// * `Err(...)` - If the title cannot be retrieved
    fn get_title(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the window class name
    /// 
    /// Returns the window class name, which is a Windows-specific identifier
    /// that describes the type of window (e.g., "Notepad", "Chrome_WidgetWin_1").
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The window class name
    /// * `Err(...)` - If the class name cannot be retrieved
    fn get_class_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the process ID of the window's owning process
    /// 
    /// Returns the process identifier (PID) of the process that owns this window.
    /// This can be used to identify which application created the window.
    /// 
    /// # Returns
    /// 
    /// * `Ok(u32)` - The process ID
    /// * `Err(...)` - If the process ID cannot be retrieved
    fn get_process_id(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Get the thread ID of the window's owning thread
    /// 
    /// Returns the thread identifier of the thread that created this window.
    /// This is primarily useful for low-level Windows programming.
    /// 
    /// # Returns
    /// 
    /// * `Ok(u32)` - The thread ID
    /// * `Err(...)` - If the thread ID cannot be retrieved
    fn get_thread_id(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Get the executable name of the window's owning process
    /// 
    /// Returns the name of the executable file (e.g., "notepad.exe", "chrome.exe")
    /// that owns this window.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The process executable name
    /// * `Err(...)` - If the process name cannot be retrieved
    fn get_process_name(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the full path to the window's owning process executable
    /// 
    /// Returns the complete file system path to the executable that owns this window.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The full path to the executable
    /// * `Err(...)` - If the process path cannot be retrieved
    fn get_process_path(&self) -> Result<String, Box<dyn Error>>;
    
    /// Check if the window is visible on screen
    /// 
    /// Returns whether the window is currently visible (not hidden).
    /// Minimized windows are typically still considered visible.
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Window is visible
    /// * `Ok(false)` - Window is hidden
    /// * `Err(...)` - If visibility state cannot be determined
    fn is_visible(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Check if the window is minimized
    /// 
    /// Returns whether the window is currently minimized to the taskbar.
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Window is minimized
    /// * `Ok(false)` - Window is not minimized
    /// * `Err(...)` - If minimized state cannot be determined
    fn is_minimized(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Check if the window is maximized
    /// 
    /// Returns whether the window is currently maximized to fill the screen
    /// (or work area).
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Window is maximized
    /// * `Ok(false)` - Window is not maximized
    /// * `Err(...)` - If maximized state cannot be determined
    fn is_maximized(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Get the window's position and size
    /// 
    /// Returns a RECT structure containing the window's screen coordinates:
    /// left, top, right, and bottom edges in pixels.
    /// 
    /// # Returns
    /// 
    /// * `Ok(RECT)` - The window's screen rectangle
    /// * `Err(...)` - If window geometry cannot be retrieved
    fn get_rect(&self) -> Result<RECT, Box<dyn Error>>;
    
    /// Get the window's DPI (dots per inch) scaling
    /// 
    /// Returns the DPI scaling factor for this window, which is important
    /// for high-DPI displays and coordinate calculations.
    /// 
    /// # Returns
    /// 
    /// * `Ok(u32)` - The DPI value (typically 96, 120, 144, 192, etc.)
    /// * `Err(...)` - If DPI cannot be retrieved
    fn get_dpi(&self) -> Result<u32, Box<dyn Error>>;
    
    /// Activate the window (bring to foreground with focus)
    /// 
    /// Makes this window the active window, bringing it to the foreground
    /// and giving it keyboard focus. This is the recommended way to ensure
    /// a window is ready for interaction.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Window was activated successfully
    /// * `Err(...)` - If the window cannot be activated
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Ensure the window is active before interacting with it
    /// window.activate()?;
    /// 
    /// // Now it's safe to send keyboard input or interact with elements
    /// let element = window.get_focused_element()?;
    /// element.set_text("Hello, World!")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn activate(&self) -> Result<(), Box<dyn Error>>;
    
    /// Bring the window to the top of the Z-order
    /// 
    /// Moves the window to the top of the window stack without necessarily
    /// giving it focus. This is less intrusive than `activate()`.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Window was brought to top successfully
    /// * `Err(...)` - If the window cannot be brought to top
    fn bring_to_top(&self) -> Result<(), Box<dyn Error>>;
    
    /// Set the window as the foreground window
    /// 
    /// Attempts to make this window the foreground window. On some systems,
    /// this may be restricted by the operating system's focus policies.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Window was set as foreground successfully
    /// * `Err(...)` - If the window cannot be set as foreground
    fn set_foreground(&self) -> Result<(), Box<dyn Error>>;
    
    /// Get the currently focused element within this window
    /// 
    /// Returns the UI element that currently has keyboard focus within this window.
    /// This is useful for determining where text input will go.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn UIElement>)` - The focused element
    /// * `Err(...)` - If no element has focus or focus cannot be determined
    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>>;

    /// Get a complete UI tree snapshot of the window
    /// 
    /// Captures the entire UI element hierarchy of this window at the current
    /// moment, creating a tree structure that can be analyzed or traversed.
    /// This is useful for debugging UI structure or finding elements.
    /// 
    /// # Returns
    /// 
    /// * `Ok(UITree)` - Complete UI tree snapshot
    /// * `Err(...)` - If the UI tree cannot be captured
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let tree = window.get_ui_tree()?;
    /// println!("Window: {} captured at {}", tree.window_title, tree.timestamp);
    /// 
    /// // Recursively explore the tree
    /// fn explore_node(node: &UITreeNode, depth: usize) {
    ///     let indent = "  ".repeat(depth);
    ///     println!("{}|- {} ({})", indent, node.name, node.control_type);
    ///     for child in &node.children {
    ///         explore_node(child, depth + 1);
    ///     }
    /// }
    /// explore_node(&tree.root, 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_ui_tree(&self) -> Result<UITree, Box<dyn Error>>;
    
    /// Find UI elements matching a query
    /// 
    /// Searches for UI elements within this window that match the specified
    /// query criteria. This is the primary method for locating specific
    /// elements to interact with.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The search criteria specifying which elements to find
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<Box<dyn UIElement>>)` - List of matching elements
    /// * `Err(...)` - If the search fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use uia_interaction::core::UIQuery;
    /// 
    /// // Find all buttons
    /// let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
    /// 
    /// // Find element by name
    /// let save_button = window.find_elements(&UIQuery::ByName("Save".to_string()))?;
    /// 
    /// // Complex query - find enabled edit controls
    /// let enabled_edits = window.find_elements(&UIQuery::And(vec![
    ///     UIQuery::ByType("Edit".to_string()),
    ///     UIQuery::ByProperty("IsEnabled".to_string(), "True".to_string())
    /// ]))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn find_elements(&self, query: &UIQuery) -> Result<Vec<Box<dyn UIElement>>, Box<dyn Error>>;

    /// Get all text-containing elements in the window
    ///
    /// Scans the window's UI tree and returns structured information about all
    /// elements that contain text. This is useful for extracting readable content
    /// from applications.
    ///
    /// # Arguments
    ///
    /// * `options` - Options controlling what elements to include
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TextElementInfo>)` - List of text element information
    /// * `Err(...)` - If elements cannot be retrieved
    ///
    /// # Example
    ///
    /// ```rust
    /// let options = TextExtractionOptions::default();
    /// let text_elements = window.get_text_elements(&options)?;
    ///
    /// for elem in text_elements {
    ///     if elem.has_text() {
    ///         println!("[{}] {}: '{}'", elem.control_type, elem.name, elem.text);
    ///     }
    /// }
    /// ```
    fn get_text_elements(&self, options: &TextExtractionOptions) -> Result<Vec<TextElementInfo>, Box<dyn Error>>;

    /// Get the selected text from the currently focused element in this window
    ///
    /// Retrieves any text that is currently selected (highlighted) within
    /// the focused element of this window.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SelectedTextInfo))` - Selected text information
    /// * `Ok(None)` - No text is currently selected
    /// * `Err(...)` - If selection cannot be retrieved
    fn get_selected_text(&self) -> Result<Option<SelectedTextInfo>, Box<dyn Error>> {
        let focused = self.get_focused_element()?;
        focused.get_selected_text()
    }
}

/// Options for text extraction from UI elements
///
/// Controls what elements are included when extracting text from a window.
///
/// # Example
///
/// ```rust
/// let options = TextExtractionOptions {
///     include_hidden: false,
///     include_disabled: true,
///     min_text_length: 1,
///     control_types: Some(vec!["Text".to_string(), "Edit".to_string()]),
///     max_depth: Some(10),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct TextExtractionOptions {
    /// Include elements that are not visible on screen
    pub include_hidden: bool,
    /// Include elements that are disabled
    pub include_disabled: bool,
    /// Minimum text length to include (0 = include empty)
    pub min_text_length: usize,
    /// Only include specific control types (None = all)
    pub control_types: Option<Vec<String>>,
    /// Maximum depth in UI tree to traverse (None = unlimited)
    pub max_depth: Option<u32>,
    /// Include element names as text if actual text is empty
    pub include_names_as_text: bool,
}

impl Default for TextExtractionOptions {
    fn default() -> Self {
        TextExtractionOptions {
            include_hidden: false,
            include_disabled: true,
            min_text_length: 1,
            control_types: None,
            max_depth: Some(20),
            include_names_as_text: true,
        }
    }
}

impl TextExtractionOptions {
    /// Create options that extract all text elements
    pub fn all() -> Self {
        TextExtractionOptions {
            include_hidden: true,
            include_disabled: true,
            min_text_length: 0,
            control_types: None,
            max_depth: None,
            include_names_as_text: true,
        }
    }

    /// Create options for extracting only visible, non-empty text
    pub fn visible_text_only() -> Self {
        TextExtractionOptions {
            include_hidden: false,
            include_disabled: true,
            min_text_length: 1,
            control_types: Some(vec![
                "Text".to_string(),
                "Edit".to_string(),
                "Document".to_string(),
            ]),
            max_depth: Some(20),
            include_names_as_text: false,
        }
    }

    /// Create options for extracting editable text fields
    pub fn editable_only() -> Self {
        TextExtractionOptions {
            include_hidden: false,
            include_disabled: false,
            min_text_length: 0,
            control_types: Some(vec![
                "Edit".to_string(),
                "Document".to_string(),
                "ComboBox".to_string(),
            ]),
            max_depth: Some(20),
            include_names_as_text: false,
        }
    }
}

/// Query system for finding UI elements with various criteria
/// 
/// This enum provides a flexible query system for finding UI elements based on
/// different criteria such as name, type, properties, or complex combinations.
/// It supports logical operations and hierarchical relationships.
/// 
/// # Variants
/// 
/// * `ByName(String)` - Find elements with a specific accessible name
/// * `ByType(String)` - Find elements of a specific control type
/// * `ByProperty(String, String)` - Find elements with a specific property value
/// * `And(Vec<UIQuery>)` - Find elements matching ALL of the given queries
/// * `Or(Vec<UIQuery>)` - Find elements matching ANY of the given queries
/// * `Not(Box<UIQuery>)` - Find elements NOT matching the given query
/// * `Child(Box<UIQuery>)` - Find elements that have children matching the query
/// * `Descendant(Box<UIQuery>)` - Find elements that have descendants matching the query
/// * `Parent(Box<UIQuery>)` - Find elements whose parent matches the query
/// * `Ancestor(Box<UIQuery>)` - Find elements with an ancestor matching the query
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::core::UIQuery;
/// use uia_interaction::factory::UIAutomationFactory;
/// 
/// let automation = UIAutomationFactory::new()?;
/// let window = automation.get_active_window()?;
/// 
/// // Simple queries
/// let buttons = window.find_elements(&UIQuery::ByType("Button".to_string()))?;
/// let save_btn = window.find_elements(&UIQuery::ByName("Save".to_string()))?;
/// 
/// // Complex query - find enabled buttons that contain "OK"
/// let ok_buttons = window.find_elements(&UIQuery::And(vec![
///     UIQuery::ByType("Button".to_string()),
///     UIQuery::ByName("OK".to_string()),
///     UIQuery::ByProperty("IsEnabled".to_string(), "True".to_string())
/// ]))?;
/// 
/// // Find any text input control (Edit or Document)
/// let text_inputs = window.find_elements(&UIQuery::Or(vec![
///     UIQuery::ByType("Edit".to_string()),
///     UIQuery::ByType("Document".to_string())
/// ]))?;
/// 
/// // Find elements that are NOT buttons
/// let non_buttons = window.find_elements(&UIQuery::Not(
///     Box::new(UIQuery::ByType("Button".to_string()))
/// ))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum UIQuery {
    ByName(String),
    ByType(String),
    ByProperty(String, String),
    And(Vec<UIQuery>),
    Or(Vec<UIQuery>),
    Not(Box<UIQuery>),
    Child(Box<UIQuery>),
    Descendant(Box<UIQuery>),
    Parent(Box<UIQuery>),
    Ancestor(Box<UIQuery>),
}

impl UIQuery {
    /// Check if an element matches this query
    /// 
    /// Tests whether the given UI element satisfies the conditions of this query.
    /// This method is used internally by the query system to filter elements.
    /// 
    /// # Arguments
    /// 
    /// * `element` - The UI element to test against this query
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Element matches the query
    /// * `Ok(false)` - Element does not match
    /// * `Err(...)` - If the matching process fails
    #[allow(dead_code)]
    pub fn matches(&self, element: &dyn UIElement) -> Result<bool, Box<dyn Error>> {
        match self {
            UIQuery::ByName(name) => {
                let props = element.get_properties()?;
                Ok(props.get("name").map_or(false, |n| n == name))
            }
            UIQuery::ByType(control_type) => {
                let props = element.get_properties()?;
                Ok(props.get("control_type").map_or(false, |t| t == control_type))
            }
            UIQuery::ByProperty(key, value) => {
                let props = element.get_properties()?;
                Ok(props.get(key).map_or(false, |v| v == value))
            }
            UIQuery::And(queries) => {
                for query in queries {
                    if !query.matches(element)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            UIQuery::Or(queries) => {
                for query in queries {
                    if query.matches(element)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            UIQuery::Not(query) => {
                Ok(!query.matches(element)?)
            }
            UIQuery::Child(query) => {
                let children = element.get_children()?;
                for child in children {
                    if query.matches(child.as_ref())? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            UIQuery::Descendant(query) => {
                let children = element.get_children()?;
                for child in children {
                    if query.matches(child.as_ref())? || 
                       UIQuery::Descendant(query.clone()).matches(child.as_ref())? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            UIQuery::Parent(_query) => {
                // Implementation here
                Ok(false)
            },
            UIQuery::Ancestor(_query) => {
                // Implementation here
                Ok(false)
            }
        }
    }

    /// Find all elements matching this query in a tree rooted at the given element
    /// 
    /// Recursively searches through the UI element tree starting from the root element
    /// and returns all elements that match this query. This method performs a
    /// depth-first search through the entire subtree.
    /// 
    /// # Arguments
    /// 
    /// * `root` - The root element to start the search from
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<Box<dyn UIElement>>)` - List of all matching elements
    /// * `Err(...)` - If the search fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use uia_interaction::core::UIQuery;
    /// use uia_interaction::factory::UIAutomationFactory;
    /// 
    /// let automation = UIAutomationFactory::new()?;
    /// let window = automation.get_active_window()?;
    /// let root_element = window.get_focused_element()?;
    /// 
    /// // Find all buttons in the subtree
    /// let query = UIQuery::ByType("Button".to_string());
    /// let buttons = query.find_all(root_element.as_ref())?;
    /// println!("Found {} buttons", buttons.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[allow(dead_code)]
    pub fn find_all(&self, root: &dyn UIElement) -> Result<Vec<Box<dyn UIElement>>, Box<dyn Error>> {
        let mut results = Vec::new();
        if self.matches(root)? {
            results.push(root.to_tree_node()?);
        }
        
        let children = root.get_children()?;
        for child in children {
            let child_results = self.find_all(child.as_ref())?;
            results.extend(child_results);
        }
        
        Ok(results)
    }
}

/// Main UI Automation interface for desktop applications
/// 
/// This trait provides the primary entry point for UI automation operations.
/// It offers methods to access windows, elements, and perform high-level
/// automation tasks across the desktop environment.
/// 
/// Implementations of this trait are platform-specific and handle the
/// low-level details of interacting with the operating system's accessibility APIs.
/// 
/// # Thread Safety
/// 
/// This trait requires `Send + Sync`, making it safe to use across threads.
/// However, be aware that some UI operations may need to be performed on
/// specific threads (e.g., the main UI thread on some platforms).
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::UIAutomationFactory;
/// use uia_interaction::core::{UIQuery, AppendPosition};
/// 
/// // Create automation instance
/// let automation = UIAutomationFactory::new()?;
/// 
/// // Get the currently active window
/// let window = automation.get_active_window()?;
/// println!("Active window: {}", window.get_title()?);
/// 
/// // Get the focused element and interact with it
/// let element = automation.get_focused_element()?;
/// println!("Focused element: {} ({})", element.get_name()?, element.get_type()?);
/// 
/// // Set text if it's a text input element
/// if element.get_type()? == "Edit" {
///     element.set_text("Hello, World!")?;
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
pub trait UIAutomation: Send + Sync {
    /// Get the currently active (foreground) window
    /// 
    /// Returns the top-level application window that currently has focus.
    /// This is typically the window the user is actively working with.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn Window>)` - The active window
    /// * `Err(...)` - If no active window can be found or accessed
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let automation = UIAutomationFactory::new()?;
    /// let active_window = automation.get_active_window()?;
    /// println!("Current application: {}", active_window.get_title()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_active_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>>;
    
    /// Get the window that contains the currently focused element
    /// 
    /// This might return the same window as `get_active_window()`, but could
    /// be different if focus is in a child window, dialog, or popup.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn Window>)` - The window containing the focused element
    /// * `Err(...)` - If no window with focus can be found
    fn get_window_containing_focus(&self) -> Result<Box<dyn Window>, Box<dyn Error>>;
    
    /// Get the currently focused element
    /// 
    /// Returns the UI element that currently has keyboard focus across
    /// the entire desktop. This element will receive keyboard input.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn UIElement>)` - The element with keyboard focus
    /// * `Err(...)` - If no focused element can be found
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let automation = UIAutomationFactory::new()?;
    /// let focused = automation.get_focused_element()?;
    /// 
    /// // Check what type of element has focus
    /// match focused.get_type()?.as_str() {
    ///     "Edit" => {
    ///         println!("Text field has focus");
    ///         focused.set_text("New text content")?;
    ///     }
    ///     "Button" => {
    ///         println!("Button has focus: {}", focused.get_name()?);
    ///     }
    ///     _ => {
    ///         println!("Other element type: {}", focused.get_type()?);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_focused_element(&self) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
    
    /// Find an element by its accessible name
    /// 
    /// Searches the entire desktop for a UI element with the specified name.
    /// This is a global search that may be slower than window-specific searches.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The accessible name to search for
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn UIElement>)` - The first element found with this name
    /// * `Err(...)` - If no element with this name is found
    fn find_element_by_name(&self, name: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
    
    /// Find an element by its control type
    /// 
    /// Searches the entire desktop for a UI element of the specified type.
    /// This is a global search that may be slower than window-specific searches.
    /// 
    /// # Arguments
    /// 
    /// * `element_type` - The control type to search for (e.g., "Button", "Edit")
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn UIElement>)` - The first element found of this type
    /// * `Err(...)` - If no element of this type is found
    fn find_element_by_type(&self, element_type: &str) -> Result<Box<dyn UIElement>, Box<dyn Error>>;
    
    /// DEPRECATED: Use get_active_window() instead
    /// 
    /// This method is deprecated and will be removed in a future version.
    /// Use `get_active_window()` for the same functionality.
    #[deprecated(since = "0.1.0", note = "Use get_active_window() instead")]
    fn get_focused_window(&self) -> Result<Box<dyn Window>, Box<dyn Error>> {
        self.get_active_window()
    }
}

/// Information about a running application on the system
/// 
/// This structure contains comprehensive information about an application process,
/// including its main window details and process information. It's used for
/// application discovery and management operations.
/// 
/// # Fields
/// 
/// * `process_id` - The unique process identifier (PID)
/// * `process_name` - The executable filename (e.g., "notepad.exe")
/// * `process_path` - Full path to the executable file
/// * `main_window_title` - Title of the application's main window
/// * `main_window_class` - Window class name of the main window
/// * `is_visible` - Whether the main window is currently visible
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::ApplicationManagerFactory;
/// 
/// let app_manager = ApplicationManagerFactory::new()?;
/// let applications = app_manager.get_all_applications()?;
/// 
/// for app in applications {
///     if app.is_visible {
///         println!("App: {} (PID: {})", app.process_name, app.process_id);
///         println!("  Window: '{}'", app.main_window_title);
///         println!("  Path: {}", app.process_path);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ApplicationInfo {
    pub process_id: u32,
    pub process_name: String,
    pub process_path: String,
    pub main_window_title: String,
    pub main_window_class: String,
    pub is_visible: bool,
}

/// Trait for discovering and managing applications running on the system
/// 
/// This trait provides methods to enumerate running applications, search for
/// specific applications by various criteria, and obtain window handles for
/// UI automation. It's the primary interface for application discovery and
/// management in the automation system.
/// 
/// # Thread Safety
/// 
/// This trait requires `Send + Sync`, making it safe to use across threads.
/// 
/// # Example
/// 
/// ```rust
/// use uia_interaction::factory::ApplicationManagerFactory;
/// use uia_interaction::core::UIQuery;
/// 
/// let app_manager = ApplicationManagerFactory::new()?;
/// 
/// // Find all Notepad instances
/// let notepad_apps = app_manager.find_applications_by_name("notepad.exe")?;
/// 
/// if let Some(notepad) = notepad_apps.first() {
///     println!("Found Notepad: {} (PID: {})", notepad.main_window_title, notepad.process_id);
///     
///     // Get the window for interaction
///     let window = app_manager.get_window_by_process_id(notepad.process_id)?;
///     
///     // Activate the window and interact with it
///     window.activate()?;
///     
///     // Find text area and set content
///     let text_areas = window.find_elements(&UIQuery::ByType("Edit".to_string()))?;
///     if let Some(text_area) = text_areas.first() {
///         text_area.set_text("Hello from automation!")?;
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)]
pub trait ApplicationManager: Send + Sync {
    /// Get information about all running applications
    /// 
    /// Enumerates all currently running applications on the system and returns
    /// information about each one, including process details and main window information.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<ApplicationInfo>)` - List of all running applications
    /// * `Err(...)` - If application enumeration fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let app_manager = ApplicationManagerFactory::new()?;
    /// let all_apps = app_manager.get_all_applications()?;
    /// 
    /// println!("Found {} running applications:", all_apps.len());
    /// for app in all_apps {
    ///     if app.is_visible {
    ///         println!("  {}: '{}' (PID: {})", 
    ///                  app.process_name, 
    ///                  app.main_window_title, 
    ///                  app.process_id);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_all_applications(&self) -> Result<Vec<ApplicationInfo>, Box<dyn Error>>;
    
    /// Find applications by process name
    /// 
    /// Searches for applications whose process name matches the given name.
    /// The search is case-insensitive and supports partial matching.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Process name to search for (e.g., "notepad", "notepad.exe", "chrome")
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<ApplicationInfo>)` - List of matching applications
    /// * `Err(...)` - If the search fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Find all Chrome instances
    /// let chrome_apps = app_manager.find_applications_by_name("chrome")?;
    /// 
    /// // Find specific executable
    /// let calculator = app_manager.find_applications_by_name("calc.exe")?;
    /// 
    /// // Find by partial name
    /// let text_editors = app_manager.find_applications_by_name("notepad")?; // Matches notepad.exe
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn find_applications_by_name(&self, name: &str) -> Result<Vec<ApplicationInfo>, Box<dyn Error>>;
    
    /// Find applications by window title
    /// 
    /// Searches for applications whose main window title contains the given text.
    /// The search is case-insensitive and supports partial matching.
    /// 
    /// # Arguments
    /// 
    /// * `title` - Window title text to search for
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<ApplicationInfo>)` - List of matching applications
    /// * `Err(...)` - If the search fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Find applications with "Discord" in the title
    /// let discord_apps = app_manager.find_applications_by_title("Discord")?;
    /// 
    /// // Find document editing applications
    /// let word_docs = app_manager.find_applications_by_title("Microsoft Word")?;
    /// 
    /// // Find any browser windows
    /// let browsers = app_manager.find_applications_by_title("Mozilla Firefox")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn find_applications_by_title(&self, title: &str) -> Result<Vec<ApplicationInfo>, Box<dyn Error>>;
    
    /// Get a window object from an application by process ID
    /// 
    /// Creates a Window object for the main window of the application with
    /// the specified process ID. This window can then be used for UI automation.
    /// 
    /// # Arguments
    /// 
    /// * `process_id` - The process ID of the target application
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn Window>)` - Window object for the application's main window
    /// * `Err(...)` - If the window cannot be found or accessed
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Find Notepad and get its window
    /// let notepad_apps = app_manager.find_applications_by_name("notepad.exe")?;
    /// if let Some(notepad) = notepad_apps.first() {
    ///     let window = app_manager.get_window_by_process_id(notepad.process_id)?;
    ///     
    ///     // Now you can interact with the window
    ///     window.activate()?;
    ///     println!("Notepad window: {}", window.get_title()?);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_window_by_process_id(&self, process_id: u32) -> Result<Box<dyn Window>, Box<dyn Error>>;
    
    /// Get the main window of an application by process name
    /// 
    /// Finds the first application with the given process name and returns
    /// its main window. This is a convenience method that combines finding
    /// an application by name and getting its window.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Process name to search for
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn Window>)` - Window object for the application's main window
    /// * `Err(...)` - If no matching application is found or window cannot be accessed
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // Get the first Calculator window directly
    /// match app_manager.get_window_by_process_name("calc.exe") {
    ///     Ok(calc_window) => {
    ///         calc_window.activate()?;
    ///         println!("Calculator: {}", calc_window.get_title()?);
    ///     }
    ///     Err(_) => {
    ///         println!("Calculator not found or not accessible");
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_window_by_process_name(&self, name: &str) -> Result<Box<dyn Window>, Box<dyn Error>>;
} 