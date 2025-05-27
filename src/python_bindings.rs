// Python bindings for the UI Automation library
// 
// This module provides Python bindings for interacting with UI elements,
// windows, and applications using UI Automation.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{debug, warn};
use ::uia_interaction::core::{UIAutomation, Window, UIElement, UITree, UITreeNode, UIQuery, ApplicationManager, ApplicationInfo, AppendPosition};
use ::uia_interaction::factory::{UIAutomationFactory, ApplicationManagerFactory};

// =============================================================================
// THREAD SAFETY UTILITIES
// =============================================================================

// Thread-safe wrapper for our non-thread-safe types
pub struct ThreadSafe<T>(pub(crate) Mutex<T>);

impl<T> ThreadSafe<T> {
    pub fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }
}

// Make ThreadSafe Send + Sync
unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}

// =============================================================================
// UI AUTOMATION MAIN CLASS
// =============================================================================

/// Main UI Automation interface for Python
/// 
/// This class provides the primary entry point for UI automation tasks.
/// Use it to get windows, elements, and interact with the desktop.
/// 
/// # Examples
/// 
/// ```python
/// from uia_interaction import PyAutomation
/// 
/// automation = PyAutomation()
/// window = automation.active_window()
/// element = automation.focused_element()
/// ```
#[pyclass]
pub struct PyAutomation {
    inner: Arc<ThreadSafe<Box<dyn UIAutomation>>>
}

#[pymethods]
impl PyAutomation {
    /// Create a new UI Automation instance
    /// 
    /// Returns:
    ///     PyAutomation: A new automation instance
    /// 
    /// Raises:
    ///     RuntimeError: If the automation system cannot be initialized
    #[new]
    pub fn new() -> PyResult<Self> {
        let factory = UIAutomationFactory::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { 
            inner: Arc::new(ThreadSafe::new(factory))
        })
    }

    /// Get the currently active (foreground) window
    /// 
    /// This returns the top-level application window that currently has focus.
    /// 
    /// Returns:
    ///     PyWindow: The currently active window
    /// 
    /// Raises:
    ///     RuntimeError: If no active window can be found
    /// 
    /// # Examples
    /// 
    /// ```python
    /// automation = PyAutomation()
    /// window = automation.active_window()
    /// print(f"Active window: {window.title}")
    /// ```
    fn active_window(&self) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_active_window()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyWindow { 
                inner: Arc::new(ThreadSafe::new(window))
            })?)
        })
    }

    /// Get the window that contains the currently focused element
    /// 
    /// This might be the same as active_window(), but could be a child window or dialog.
    /// 
    /// Returns:
    ///     PyWindow: The window containing the focused element
    /// 
    /// Raises:
    ///     RuntimeError: If no window with focus can be found
    fn window_containing_focus(&self) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_window_containing_focus()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyWindow { 
                inner: Arc::new(ThreadSafe::new(window))
            })?)
        })
    }

    /// Get the currently focused element
    /// 
    /// This returns the UI element that currently has keyboard focus.
    /// 
    /// Returns:
    ///     PyUIElement: The element with keyboard focus
    /// 
    /// Raises:
    ///     RuntimeError: If no focused element can be found
    /// 
    /// # Examples
    /// 
    /// ```python
    /// automation = PyAutomation()
    /// element = automation.focused_element()
    /// print(f"Focused element: {element.name} ({element.control_type})")
    /// ```
    fn focused_element(&self) -> PyResult<Py<PyUIElement>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let element = inner.get_focused_element()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyUIElement { 
                inner: Arc::new(ThreadSafe::new(element))
            })?)
        })
    }

    /// DEPRECATED: Use active_window() instead
    /// 
    /// This method is deprecated and will be removed in a future version.
    /// Use active_window() for the same functionality.
    fn focused_window(&self) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_active_window()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyWindow { 
                inner: Arc::new(ThreadSafe::new(window))
            })?)
        })
    }
}

// =============================================================================
// UI ELEMENT CLASS
// =============================================================================

/// Represents a UI element in an application
/// 
/// This class provides methods to interact with UI elements such as buttons,
/// text fields, menus, and other controls in desktop applications.
/// 
/// # Examples
/// 
/// ```python
/// # Get an element and interact with it
/// element = window.find_elements(PyUIQuery.by_name("OK"))[0]
/// print(f"Element: {element.name} ({element.control_type})")
/// element.click()
/// 
/// # Set text in a text field
/// text_field = window.find_elements(PyUIQuery.by_type("Edit"))[0]
/// text_field.set_text("Hello, World!")
/// ```
#[pyclass]
pub struct PyUIElement {
    inner: Arc<ThreadSafe<Box<dyn UIElement>>>
}

#[pymethods]
impl PyUIElement {
    /// Get the name/label of the element
    /// 
    /// Returns:
    ///     str: The element's name or label, empty string if none
    /// 
    /// # Examples
    /// 
    /// ```python
    /// button = window.find_elements(PyUIQuery.by_type("Button"))[0]
    /// print(f"Button name: {button.name}")
    /// ```
    #[getter]
    fn name(&self) -> PyResult<String> {
        debug!("Getting name for UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.get_name() {
            Ok(name) => {
                debug!("Successfully got name: {}", name);
                Ok(name)
            },
            Err(e) => {
                warn!("Failed to get name: {}", e);
                Ok(String::new())
            }
        }
    }

    /// Get the control type of the element
    /// 
    /// Returns:
    ///     str: The control type (e.g., "Button", "Edit", "Document")
    /// 
    /// # Examples
    /// 
    /// ```python
    /// element = window.find_elements(PyUIQuery.by_name("Submit"))[0]
    /// if element.control_type == "Button":
    ///     element.click()
    /// ```
    #[getter]
    fn control_type(&self) -> PyResult<String> {
        debug!("Getting control type for UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.get_type() {
            Ok(typ) => {
                debug!("Successfully got control type: {}", typ);
                Ok(typ)
            },
            Err(e) => {
                warn!("Failed to get control type: {}", e);
                Ok(String::new())
            }
        }
    }

    /// Check if the element is enabled for interaction
    /// 
    /// Returns:
    ///     bool: True if the element can be interacted with, False otherwise
    /// 
    /// # Examples
    /// 
    /// ```python
    /// button = window.find_elements(PyUIQuery.by_name("Save"))[0]
    /// if button.is_enabled:
    ///     button.click()
    /// else:
    ///     print("Save button is disabled")
    /// ```
    #[getter]
    fn is_enabled(&self) -> PyResult<bool> {
        debug!("Getting enabled state for UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.is_enabled() {
            Ok(val) => {
                debug!("Successfully got enabled state: {}", val);
                Ok(val)
            },
            Err(e) => {
                warn!("Failed to get enabled state: {}", e);
                Ok(false)
            }
        }
    }

    /// Get all properties of the element
    /// 
    /// Returns:
    ///     dict: Dictionary containing all element properties
    /// 
    /// # Examples
    /// 
    /// ```python
    /// element = window.find_elements(PyUIQuery.by_name("Login"))[0]
    /// props = element.get_properties()
    /// print(f"Properties: {props}")
    /// ```
    fn get_properties(&self) -> PyResult<HashMap<String, String>> {
        debug!("Getting properties for UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.get_properties() {
            Ok(props) => {
                debug!("Successfully got {} properties", props.len());
                Ok(props)
            },
            Err(e) => {
                warn!("Failed to get properties: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    /// Get child elements of this element
    /// 
    /// Returns:
    ///     list[PyUIElement]: List of child elements
    /// 
    /// # Examples
    /// 
    /// ```python
    /// menu = window.find_elements(PyUIQuery.by_type("MenuBar"))[0]
    /// menu_items = menu.get_children()
    /// for item in menu_items:
    ///     print(f"Menu item: {item.name}")
    /// ```
    fn get_children(&self) -> PyResult<Vec<Py<PyUIElement>>> {
        debug!("Getting children for UI element");
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            match inner.get_children() {
                Ok(children) => {
                    debug!("Successfully got {} children", children.len());
            Ok(children.into_iter()
                .map(|element| Py::new(py, PyUIElement { 
                    inner: Arc::new(ThreadSafe::new(element))
                }).unwrap())
                .collect())
                },
                Err(e) => {
                    warn!("Failed to get children: {}", e);
                    Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                }
            }
        })
    }

    /// Set the text content of the element
    /// 
    /// This method completely replaces the current text content.
    /// 
    /// Args:
    ///     text (str): The text to set
    /// 
    /// Raises:
    ///     RuntimeError: If the text cannot be set
    /// 
    /// # Examples
    /// 
    /// ```python
    /// text_field = window.find_elements(PyUIQuery.by_type("Edit"))[0]
    /// text_field.set_text("Hello, World!")
    /// ```
    fn set_text(&self, text: &str) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_text(text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Append text to the element's current content
    /// 
    /// Args:
    ///     text (str): The text to append
    ///     position (str): Where to append ("CurrentCursor", "EndOfLine", "EndOfText")
    /// 
    /// Raises:
    ///     RuntimeError: If the text cannot be appended
    /// 
    /// # Examples
    /// 
    /// ```python
    /// text_field = window.find_elements(PyUIQuery.by_type("Edit"))[0]
    /// text_field.append_text(" - Additional text", "EndOfText")
    /// ```
    fn append_text(&self, text: &str, position: &str) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        let append_pos = match position {
            "CurrentCursor" => AppendPosition::CurrentCursor,
            "EndOfLine" => AppendPosition::EndOfLine,
            "EndOfText" => AppendPosition::EndOfText,
            _ => AppendPosition::EndOfText, // Default
        };
        inner.append_text(text, append_pos)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get the text content of the element
    /// 
    /// Returns:
    ///     str: The current text content, empty string if none
    /// 
    /// # Examples
    /// 
    /// ```python
    /// text_field = window.find_elements(PyUIQuery.by_type("Edit"))[0]
    /// current_text = text_field.get_text()
    /// print(f"Current text: {current_text}")
    /// ```
    fn get_text(&self) -> PyResult<String> {
        debug!("Getting text from UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.get_text() {
            Ok(text) => {
                debug!("Successfully got text: {}", text);
                Ok(text)
            },
            Err(e) => {
                warn!("Failed to get text: {}", e);
                Ok(String::new())
            }
        }
    }

    /// Click the element
    /// 
    /// Performs a mouse click on the element.
    /// 
    /// Raises:
    ///     RuntimeError: If the element cannot be clicked
    /// 
    /// # Examples
    /// 
    /// ```python
    /// button = window.find_elements(PyUIQuery.by_name("OK"))[0]
    /// button.click()
    /// ```
    fn click(&self) -> PyResult<()> {
        debug!("Attempting to click UI element");
        let inner = self.inner.0.lock().unwrap();
        match inner.click() {
            Ok(()) => {
                debug!("Successfully clicked UI element");
                Ok(())
            },
            Err(e) => {
                warn!("Failed to click UI element: {}", e);
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
            }
        }
    }
}

// =============================================================================
// WINDOW CLASS
// =============================================================================

/// Represents a window in a desktop application
/// 
/// This class provides methods to interact with windows, find elements
/// within them, and control window state.
/// 
/// # Examples
/// 
/// ```python
/// # Get the active window and interact with it
/// automation = PyAutomation()
/// window = automation.active_window()
/// print(f"Window title: {window.title}")
/// 
/// # Find elements in the window
/// buttons = window.find_elements(PyUIQuery.by_type("Button"))
/// if buttons:
///     buttons[0].click()
/// ```
#[pyclass]
pub struct PyWindow {
    inner: Arc<ThreadSafe<Box<dyn Window>>>
}

#[pymethods]
impl PyWindow {
    /// Get the window title
    /// 
    /// Returns:
    ///     str: The window's title text
    /// 
    /// Raises:
    ///     RuntimeError: If the title cannot be retrieved
    /// 
    /// # Examples
    /// 
    /// ```python
    /// window = automation.active_window()
    /// print(f"Current window: {window.title}")
    /// ```
    #[getter]
    fn title(&self) -> PyResult<String> {
        let inner = self.inner.0.lock().unwrap();
        inner.get_title()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get the complete UI tree for this window
    /// 
    /// This returns a hierarchical representation of all UI elements
    /// in the window, useful for debugging and understanding the UI structure.
    /// 
    /// Returns:
    ///     PyUITree: Complete tree structure of the window's UI elements
    /// 
    /// Raises:
    ///     RuntimeError: If the UI tree cannot be retrieved
    /// 
    /// # Examples
    /// 
    /// ```python
    /// window = automation.active_window()
    /// tree = window.get_ui_tree()
    /// print(f"Window: {tree.window_title}")
    /// print(f"Root element: {tree.root.name}")
    /// ```
    fn get_ui_tree(&self) -> PyResult<Py<PyUITree>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let tree = inner.get_ui_tree()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyUITree { inner: tree })?)
        })
    }

    /// Find UI elements in the window matching a query
    /// 
    /// This is the primary method for locating specific UI elements
    /// within the window using various search criteria.
    /// 
    /// Args:
    ///     query (PyUIQuery): Query object specifying search criteria
    /// 
    /// Returns:
    ///     list[PyUIElement]: List of matching elements
    /// 
    /// Raises:
    ///     RuntimeError: If the search fails
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find all buttons
    /// buttons = window.find_elements(PyUIQuery.by_type("Button"))
    /// 
    /// # Find element by name
    /// save_btn = window.find_elements(PyUIQuery.by_name("Save"))
    /// 
    /// # Find enabled edit controls
    /// edits = window.find_elements(PyUIQuery.by_type("Edit"))
    /// enabled_edits = [e for e in edits if e.is_enabled]
    /// ```
    fn find_elements(&self, query: &PyUIQuery) -> PyResult<Vec<Py<PyUIElement>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let elements = inner.find_elements(&query.inner)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(elements.into_iter()
                .map(|element| Py::new(py, PyUIElement { 
                    inner: Arc::new(ThreadSafe::new(element))
                }).unwrap())
                .collect())
        })
    }

    /// Activate the window (bring it to the foreground)
    /// 
    /// This method makes the window the active window, bringing it
    /// to the front and giving it keyboard focus.
    /// 
    /// Raises:
    ///     RuntimeError: If the window cannot be activated
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find a specific application window and activate it
    /// app_manager = PyApplicationManager()
    /// apps = app_manager.find_applications_by_name("notepad.exe")
    /// if apps:
    ///     window = app_manager.get_window_by_process_id(apps[0].process_id)
    ///     window.activate()
    /// ```
    fn activate(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.activate()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Bring the window to the top of the Z-order
    /// 
    /// This method brings the window to the top without necessarily
    /// giving it focus (unlike activate()).
    /// 
    /// Raises:
    ///     RuntimeError: If the window cannot be brought to top
    /// 
    /// # Examples
    /// 
    /// ```python
    /// window.bring_to_top()
    /// ```
    fn bring_to_top(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.bring_to_top()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Set the window as the foreground window
    /// 
    /// This method attempts to make the window the foreground window,
    /// which may be restricted by the operating system.
    /// 
    /// Raises:
    ///     RuntimeError: If the window cannot be set as foreground
    /// 
    /// # Examples
    /// 
    /// ```python
    /// window.set_foreground()
    /// ```
    fn set_foreground(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_foreground()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

// =============================================================================
// UI TREE CLASSES
// =============================================================================

/// Represents a complete UI tree structure of a window
/// 
/// This class provides a snapshot of the entire UI hierarchy of a window,
/// useful for debugging, analysis, and understanding the structure of an application.
/// 
/// # Examples
/// 
/// ```python
/// window = automation.active_window()
/// tree = window.get_ui_tree()
/// print(f"Window: {tree.window_title}")
/// print(f"Root element: {tree.root.name} ({tree.root.control_type})")
/// 
/// # Explore the tree structure
/// def print_tree(node, indent=0):
///     print("  " * indent + f"{node.name} ({node.control_type})")
///     for child in node.children:
///         print_tree(child, indent + 1)
/// 
/// print_tree(tree.root)
/// ```
#[pyclass]
pub struct PyUITree {
    inner: UITree
}

#[pymethods]
impl PyUITree {
    /// Get the root node of the UI tree
    /// 
    /// Returns:
    ///     PyUITreeNode: The root element of the window's UI hierarchy
    /// 
    /// # Examples
    /// 
    /// ```python
    /// tree = window.get_ui_tree()
    /// root = tree.root
    /// print(f"Root element: {root.name}")
    /// ```
    #[getter]
    fn root(&self) -> Py<PyUITreeNode> {
        Python::with_gil(|py| {
            Py::new(py, PyUITreeNode { 
                inner: self.inner.root.clone() 
            }).unwrap()
        })
    }

    /// Get the window title when the tree was captured
    /// 
    /// Returns:
    ///     str: The title of the window at the time of tree capture
    #[getter]
    fn window_title(&self) -> String {
        self.inner.window_title.clone()
    }

    /// Get the window class name when the tree was captured
    /// 
    /// Returns:
    ///     str: The class name of the window at the time of tree capture
    #[getter]
    fn window_class(&self) -> String {
        self.inner.window_class.clone()
    }

    /// Get the timestamp when the tree was captured
    /// 
    /// Returns:
    ///     str: ISO 8601 formatted timestamp of when the tree was captured
    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
}

/// Represents a node in the UI tree hierarchy
/// 
/// Each node represents a UI element and contains information about
/// its properties, state, and children.
/// 
/// # Examples
/// 
/// ```python
/// def find_buttons(node):
///     buttons = []
///     if node.control_type == "Button":
///         buttons.append(node)
///     for child in node.children:
///         buttons.extend(find_buttons(child))
///     return buttons
/// 
/// tree = window.get_ui_tree()
/// buttons = find_buttons(tree.root)
/// print(f"Found {len(buttons)} buttons")
/// ```
#[pyclass]
pub struct PyUITreeNode {
    inner: UITreeNode
}

#[pymethods]
impl PyUITreeNode {
    /// Get the name/label of the element
    /// 
    /// Returns:
    ///     str: The element's name or label
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the control type of the element
    /// 
    /// Returns:
    ///     str: The control type (e.g., "Button", "Edit", "Document")
    #[getter]
    fn control_type(&self) -> String {
        self.inner.control_type.clone()
    }

    /// Get all properties of the element
    /// 
    /// Returns:
    ///     dict: Dictionary containing all element properties
    #[getter]
    fn properties(&self) -> HashMap<String, String> {
        self.inner.properties.clone()
    }

    /// Get child nodes of this element
    /// 
    /// Returns:
    ///     list[PyUITreeNode]: List of child nodes
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Recursively print all element names
    /// def print_names(node, indent=0):
    ///     print("  " * indent + node.name)
    ///     for child in node.children:
    ///         print_names(child, indent + 1)
    /// ```
    #[getter]
    fn children(&self) -> Vec<Py<PyUITreeNode>> {
        Python::with_gil(|py| {
            self.inner.children.iter()
                .map(|child| Py::new(py, PyUITreeNode { 
                    inner: child.clone() 
                }).unwrap())
                .collect()
        })
    }

    /// Check if the element is enabled for interaction
    /// 
    /// Returns:
    ///     bool: True if the element can be interacted with
    #[getter]
    fn is_enabled(&self) -> bool {
        self.inner.is_enabled
    }

    /// Check if the element is visible on screen
    /// 
    /// Returns:
    ///     bool: True if the element is visible
    #[getter]
    fn is_visible(&self) -> bool {
        self.inner.is_visible
    }
}

// =============================================================================
// UI QUERY CLASS
// =============================================================================

/// Query builder for finding UI elements
/// 
/// This class provides methods to create queries for finding specific UI elements
/// based on various criteria such as name, type, properties, or combinations thereof.
/// 
/// # Examples
/// 
/// ```python
/// from uia_interaction import PyUIQuery
/// 
/// # Find all buttons
/// buttons = window.find_elements(PyUIQuery.by_type("Button"))
/// 
/// # Find element by name
/// ok_button = window.find_elements(PyUIQuery.by_name("OK"))
/// 
/// # Find by custom property
/// special = window.find_elements(PyUIQuery.by_property("AutomationId", "SpecialButton"))
/// 
/// # Combine multiple criteria
/// enabled_buttons = window.find_elements(
///     PyUIQuery.and_([
///         PyUIQuery.by_type("Button"),
///         PyUIQuery.by_property("IsEnabled", "True")
///     ])
/// )
/// ```
#[pyclass]
pub struct PyUIQuery {
    inner: UIQuery
}

#[pymethods]
impl PyUIQuery {
    /// Create a query to find elements by name/label
    /// 
    /// Args:
    ///     name (str): The name or label to search for (exact match)
    /// 
    /// Returns:
    ///     PyUIQuery: Query object for finding elements by name
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find the "Save" button
    /// save_query = PyUIQuery.by_name("Save")
    /// save_buttons = window.find_elements(save_query)
    /// ```
    #[staticmethod]
    fn by_name(name: String) -> Self {
        Self { inner: UIQuery::ByName(name) }
    }

    /// Create a query to find elements by control type
    /// 
    /// Args:
    ///     control_type (str): The control type to search for (e.g., "Button", "Edit", "Document")
    /// 
    /// Returns:
    ///     PyUIQuery: Query object for finding elements by type
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find all text input fields
    /// edit_query = PyUIQuery.by_type("Edit")
    /// text_fields = window.find_elements(edit_query)
    /// 
    /// # Find all buttons
    /// button_query = PyUIQuery.by_type("Button")
    /// buttons = window.find_elements(button_query)
    /// ```
    #[staticmethod]
    fn by_type(control_type: String) -> Self {
        Self { inner: UIQuery::ByType(control_type) }
    }

    /// Create a query to find elements by a specific property
    /// 
    /// Args:
    ///     key (str): The property name to match
    ///     value (str): The property value to match
    /// 
    /// Returns:
    ///     PyUIQuery: Query object for finding elements by property
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find element with specific AutomationId
    /// id_query = PyUIQuery.by_property("AutomationId", "submitButton")
    /// element = window.find_elements(id_query)
    /// 
    /// # Find enabled elements
    /// enabled_query = PyUIQuery.by_property("IsEnabled", "True")
    /// enabled_elements = window.find_elements(enabled_query)
    /// ```
    #[staticmethod]
    fn by_property(key: String, value: String) -> Self {
        Self { inner: UIQuery::ByProperty(key, value) }
    }

    /// Create a query that matches elements satisfying ALL of the given queries
    /// 
    /// Args:
    ///     queries (list[PyUIQuery]): List of queries that must all match
    /// 
    /// Returns:
    ///     PyUIQuery: Query object that requires all conditions to be met
    /// 
    /// Raises:
    ///     RuntimeError: If query construction fails
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find enabled buttons with specific name
    /// complex_query = PyUIQuery.and_([
    ///     PyUIQuery.by_type("Button"),
    ///     PyUIQuery.by_name("Submit"),
    ///     PyUIQuery.by_property("IsEnabled", "True")
    /// ])
    /// matching_buttons = window.find_elements(complex_query)
    /// ```
    #[staticmethod]
    fn and_(queries: Vec<Py<PyUIQuery>>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let inner_queries: Vec<UIQuery> = queries.iter()
                .map(|q| q.borrow(py).inner.clone())
                .collect();
            Ok(Self { inner: UIQuery::And(inner_queries) })
        })
    }

    /// Create a query that matches elements satisfying ANY of the given queries
    /// 
    /// Args:
    ///     queries (list[PyUIQuery]): List of queries where at least one must match
    /// 
    /// Returns:
    ///     PyUIQuery: Query object that requires at least one condition to be met
    /// 
    /// Raises:
    ///     RuntimeError: If query construction fails
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find either "OK" or "Cancel" buttons
    /// choice_query = PyUIQuery.or_([
    ///     PyUIQuery.by_name("OK"),
    ///     PyUIQuery.by_name("Cancel")
    /// ])
    /// action_buttons = window.find_elements(choice_query)
    /// 
    /// # Find any text input controls (Edit or Document)
    /// text_query = PyUIQuery.or_([
    ///     PyUIQuery.by_type("Edit"),
    ///     PyUIQuery.by_type("Document")
    /// ])
    /// text_controls = window.find_elements(text_query)
    /// ```
    #[staticmethod]
    fn or_(queries: Vec<Py<PyUIQuery>>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let inner_queries: Vec<UIQuery> = queries.iter()
                .map(|q| q.borrow(py).inner.clone())
                .collect();
            Ok(Self { inner: UIQuery::Or(inner_queries) })
        })
    }
}

// =============================================================================
// APPLICATION MANAGER CLASSES
// =============================================================================

/// Information about a running application
/// 
/// This class contains details about a running application process,
/// including its window information and process details.
/// 
/// # Examples
/// 
/// ```python
/// app_manager = PyApplicationManager()
/// apps = app_manager.get_all_applications()
/// for app in apps:
///     print(f"App: {app.process_name} (PID: {app.process_id})")
///     print(f"Window: {app.main_window_title}")
///     print(f"Visible: {app.is_visible}")
/// ```
#[pyclass]
pub struct PyApplicationInfo {
    inner: ApplicationInfo
}

#[pymethods]
impl PyApplicationInfo {
    /// Get the process ID of the application
    /// 
    /// Returns:
    ///     int: The process identifier
    #[getter]
    fn process_id(&self) -> u32 {
        self.inner.process_id
    }

    /// Get the process name (executable name)
    /// 
    /// Returns:
    ///     str: The name of the executable file (e.g., "notepad.exe")
    #[getter]
    fn process_name(&self) -> String {
        self.inner.process_name.clone()
    }

    /// Get the full path to the process executable
    /// 
    /// Returns:
    ///     str: The complete file path to the executable
    #[getter]
    fn process_path(&self) -> String {
        self.inner.process_path.clone()
    }

    /// Get the title of the main window
    /// 
    /// Returns:
    ///     str: The title text of the application's main window
    #[getter]
    fn main_window_title(&self) -> String {
        self.inner.main_window_title.clone()
    }

    /// Get the class name of the main window
    /// 
    /// Returns:
    ///     str: The window class name
    #[getter]
    fn main_window_class(&self) -> String {
        self.inner.main_window_class.clone()
    }

    /// Check if the application window is visible
    /// 
    /// Returns:
    ///     bool: True if the main window is visible on screen
    #[getter]
    fn is_visible(&self) -> bool {
        self.inner.is_visible
    }

    /// Get a string representation of the application info
    /// 
    /// Returns:
    ///     str: Human-readable representation
    fn __repr__(&self) -> String {
        format!(
            "ApplicationInfo(process_id={}, process_name='{}', main_window_title='{}')",
            self.inner.process_id,
            self.inner.process_name,
            self.inner.main_window_title
        )
    }
}

/// Manager for discovering and interacting with running applications
/// 
/// This class provides methods to find running applications, get their windows,
/// and filter them by various criteria such as process name or window title.
/// 
/// # Examples
/// 
/// ```python
/// from uia_interaction import PyApplicationManager, PyUIQuery
/// 
/// # Create manager and find applications
/// app_manager = PyApplicationManager()
/// 
/// # Find all Notepad instances
/// notepad_apps = app_manager.find_applications_by_name("notepad.exe")
/// if notepad_apps:
///     # Get the first Notepad window
///     window = app_manager.get_window_by_process_id(notepad_apps[0].process_id)
///     print(f"Connected to: {window.title}")
///     
///     # Find text area and set content
///     text_areas = window.find_elements(PyUIQuery.by_type("Edit"))
///     if text_areas:
///         text_areas[0].set_text("Hello from Python!")
/// ```
#[pyclass]
pub struct PyApplicationManager {
    inner: Arc<ThreadSafe<Box<dyn ApplicationManager>>>
}

#[pymethods]
impl PyApplicationManager {
    /// Create a new Application Manager
    /// 
    /// Returns:
    ///     PyApplicationManager: A new application manager instance
    /// 
    /// Raises:
    ///     RuntimeError: If the application manager cannot be initialized
    #[new]
    pub fn new() -> PyResult<Self> {
        let manager = ApplicationManagerFactory::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { 
            inner: Arc::new(ThreadSafe::new(manager))
        })
    }

    /// Get information about all running applications
    /// 
    /// Returns:
    ///     list[PyApplicationInfo]: List of all running applications
    /// 
    /// Raises:
    ///     RuntimeError: If applications cannot be enumerated
    /// 
    /// # Examples
    /// 
    /// ```python
    /// app_manager = PyApplicationManager()
    /// all_apps = app_manager.get_all_applications()
    /// 
    /// print(f"Found {len(all_apps)} running applications:")
    /// for app in all_apps:
    ///     if app.is_visible:
    ///         print(f"  {app.process_name}: {app.main_window_title}")
    /// ```
    fn get_all_applications(&self) -> PyResult<Vec<Py<PyApplicationInfo>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let apps = inner.get_all_applications()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(apps.into_iter()
                .map(|app| Py::new(py, PyApplicationInfo { inner: app }).unwrap())
                .collect())
        })
    }

    /// Find applications by process name
    /// 
    /// Searches for applications whose process name matches the given name.
    /// The search is case-insensitive and supports partial matching.
    /// 
    /// Args:
    ///     name (str): Process name to search for (e.g., "notepad", "notepad.exe")
    /// 
    /// Returns:
    ///     list[PyApplicationInfo]: List of matching applications
    /// 
    /// Raises:
    ///     RuntimeError: If the search fails
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find all Chrome instances
    /// chrome_apps = app_manager.find_applications_by_name("chrome")
    /// 
    /// # Find specific executable
    /// calculator = app_manager.find_applications_by_name("calc.exe")
    /// ```
    fn find_applications_by_name(&self, name: &str) -> PyResult<Vec<Py<PyApplicationInfo>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let apps = inner.find_applications_by_name(name)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(apps.into_iter()
                .map(|app| Py::new(py, PyApplicationInfo { inner: app }).unwrap())
                .collect())
        })
    }

    /// Find applications by window title
    /// 
    /// Searches for applications whose main window title contains the given text.
    /// The search is case-insensitive and supports partial matching.
    /// 
    /// Args:
    ///     title (str): Window title text to search for
    /// 
    /// Returns:
    ///     list[PyApplicationInfo]: List of matching applications
    /// 
    /// Raises:
    ///     RuntimeError: If the search fails
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find applications with "Discord" in the title
    /// discord_apps = app_manager.find_applications_by_title("Discord")
    /// 
    /// # Find document editing applications
    /// word_docs = app_manager.find_applications_by_title("Microsoft Word")
    /// ```
    fn find_applications_by_title(&self, title: &str) -> PyResult<Vec<Py<PyApplicationInfo>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let apps = inner.find_applications_by_title(title)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(apps.into_iter()
                .map(|app| Py::new(py, PyApplicationInfo { inner: app }).unwrap())
                .collect())
        })
    }

    /// Get a window from an application by process ID
    /// 
    /// Creates a Window object for the main window of the application
    /// with the specified process ID.
    /// 
    /// Args:
    ///     process_id (int): The process ID of the application
    /// 
    /// Returns:
    ///     PyWindow: Window object for the application's main window
    /// 
    /// Raises:
    ///     RuntimeError: If the window cannot be found or accessed
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Find Notepad and get its window
    /// notepad_apps = app_manager.find_applications_by_name("notepad.exe")
    /// if notepad_apps:
    ///     window = app_manager.get_window_by_process_id(notepad_apps[0].process_id)
    ///     print(f"Notepad window: {window.title}")
    /// ```
    fn get_window_by_process_id(&self, process_id: u32) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_window_by_process_id(process_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(Py::new(py, PyWindow { 
                inner: Arc::new(ThreadSafe::new(window))
            })?)
        })
    }

    /// Get the main window of an application by process name
    /// 
    /// Finds the first application with the given process name and returns
    /// its main window. If multiple applications match, the first one found is used.
    /// 
    /// Args:
    ///     name (str): Process name to search for
    /// 
    /// Returns:
    ///     PyWindow: Window object for the application's main window
    /// 
    /// Raises:
    ///     RuntimeError: If no application is found or window cannot be accessed
    /// 
    /// # Examples
    /// 
    /// ```python
    /// # Get the first Calculator window
    /// try:
    ///     calc_window = app_manager.get_window_by_process_name("calc.exe")
    ///     print(f"Calculator: {calc_window.title}")
    /// except RuntimeError:
    ///     print("Calculator not found or not accessible")
    /// ```
    fn get_window_by_process_name(&self, name: &str) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_window_by_process_name(name)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(Py::new(py, PyWindow { 
                inner: Arc::new(ThreadSafe::new(window))
            })?)
        })
    }
}

// =============================================================================
// MODULE REGISTRATION
// =============================================================================

/// Register all Python classes and create the uia_interaction module
#[pymodule]
pub fn uia_interaction(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAutomation>()?;
    m.add_class::<PyWindow>()?;
    m.add_class::<PyUIElement>()?;
    m.add_class::<PyUITree>()?;
    m.add_class::<PyUITreeNode>()?;
    m.add_class::<PyUIQuery>()?;
    m.add_class::<PyApplicationInfo>()?;
    m.add_class::<PyApplicationManager>()?;
    Ok(())
} 