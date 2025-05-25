use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use ::uia_interaction::core::{UIAutomation, Window, UIElement, UITree, UITreeNode, UIQuery, ApplicationManager, ApplicationInfo};
use ::uia_interaction::factory::{UIAutomationFactory, ApplicationManagerFactory};
use std::collections::HashMap;
use log::{debug, warn};

// Thread-safe wrapper for our non-thread-safe types
struct ThreadSafe<T>(Mutex<T>);

impl<T> ThreadSafe<T> {
    fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }
}

// Make ThreadSafe Send + Sync
unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}

#[pyclass]
pub struct PyUIElement {
    inner: Arc<ThreadSafe<Box<dyn UIElement>>>
}

#[pymethods]
impl PyUIElement {
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

    fn set_text(&self, text: &str) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_text(text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn append_text(&self, text: &str, position: &str) -> PyResult<()> {
        use ::uia_interaction::core::AppendPosition;
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

#[pyclass]
pub struct PyUITreeNode {
    inner: UITreeNode
}

#[pymethods]
impl PyUITreeNode {
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn control_type(&self) -> String {
        self.inner.control_type.clone()
    }

    #[getter]
    fn properties(&self) -> HashMap<String, String> {
        self.inner.properties.clone()
    }

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

    #[getter]
    fn is_enabled(&self) -> bool {
        self.inner.is_enabled
    }

    #[getter]
    fn is_visible(&self) -> bool {
        self.inner.is_visible
    }
}

#[pyclass]
pub struct PyUITree {
    inner: UITree
}

#[pymethods]
impl PyUITree {
    #[getter]
    fn root(&self) -> Py<PyUITreeNode> {
        Python::with_gil(|py| {
            Py::new(py, PyUITreeNode { 
                inner: self.inner.root.clone() 
            }).unwrap()
        })
    }

    #[getter]
    fn window_title(&self) -> String {
        self.inner.window_title.clone()
    }

    #[getter]
    fn window_class(&self) -> String {
        self.inner.window_class.clone()
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
}

#[pyclass]
pub struct PyUIQuery {
    inner: UIQuery
}

#[pymethods]
impl PyUIQuery {
    #[staticmethod]
    fn by_name(name: String) -> Self {
        Self { inner: UIQuery::ByName(name) }
    }

    #[staticmethod]
    fn by_type(control_type: String) -> Self {
        Self { inner: UIQuery::ByType(control_type) }
    }

    #[staticmethod]
    fn by_property(key: String, value: String) -> Self {
        Self { inner: UIQuery::ByProperty(key, value) }
    }

    #[staticmethod]
    fn and_(queries: Vec<Py<PyUIQuery>>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let inner_queries: Vec<UIQuery> = queries.iter()
                .map(|q| q.borrow(py).inner.clone())
                .collect();
            Ok(Self { inner: UIQuery::And(inner_queries) })
        })
    }

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

#[pyclass]
pub struct PyWindow {
    inner: Arc<ThreadSafe<Box<dyn Window>>>
}

#[pymethods]
impl PyWindow {
    #[getter]
    fn title(&self) -> PyResult<String> {
        let inner = self.inner.0.lock().unwrap();
        inner.get_title()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_ui_tree(&self) -> PyResult<Py<PyUITree>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let tree = inner.get_ui_tree()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Py::new(py, PyUITree { inner: tree })?)
        })
    }

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

    fn activate(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.activate()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn bring_to_top(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.bring_to_top()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn set_foreground(&self) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_foreground()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

#[pyclass]
pub struct PyAutomation {
    inner: Arc<ThreadSafe<Box<dyn UIAutomation>>>
}

#[pymethods]
impl PyAutomation {
    #[new]
    pub fn new() -> PyResult<Self> {
        let factory = UIAutomationFactory::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { 
            inner: Arc::new(ThreadSafe::new(factory))
        })
    }

    /// Get the currently active (foreground) window - the top-level application window
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

    /// Get the currently focused element (the element with keyboard focus)
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

#[pyclass]
pub struct PyApplicationInfo {
    inner: ApplicationInfo
}

#[pymethods]
impl PyApplicationInfo {
    #[getter]
    fn process_id(&self) -> u32 {
        self.inner.process_id
    }

    #[getter]
    fn process_name(&self) -> String {
        self.inner.process_name.clone()
    }

    #[getter]
    fn process_path(&self) -> String {
        self.inner.process_path.clone()
    }

    #[getter]
    fn main_window_title(&self) -> String {
        self.inner.main_window_title.clone()
    }

    #[getter]
    fn main_window_class(&self) -> String {
        self.inner.main_window_class.clone()
    }

    #[getter]
    fn is_visible(&self) -> bool {
        self.inner.is_visible
    }

    fn __repr__(&self) -> String {
        format!(
            "ApplicationInfo(process_id={}, process_name='{}', main_window_title='{}')",
            self.inner.process_id,
            self.inner.process_name,
            self.inner.main_window_title
        )
    }
}

#[pyclass]
pub struct PyApplicationManager {
    inner: Arc<ThreadSafe<Box<dyn ApplicationManager>>>
}

#[pymethods]
impl PyApplicationManager {
    #[new]
    pub fn new() -> PyResult<Self> {
        let manager = ApplicationManagerFactory::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { 
            inner: Arc::new(ThreadSafe::new(manager))
        })
    }

    /// Get all running applications
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

    /// Find applications by process name (e.g., "notepad.exe")
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

    /// Find applications by window title (partial match)
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