use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use ::uia_interaction::core::{UIAutomation, Window, UIElement, UITree, UITreeNode, UIQuery};
use ::uia_interaction::factory::UIAutomationFactory;
use std::collections::HashMap;

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
        let inner = self.inner.0.lock().unwrap();
        inner.get_name()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[getter]
    fn control_type(&self) -> PyResult<String> {
        let inner = self.inner.0.lock().unwrap();
        inner.get_type()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[getter]
    fn is_enabled(&self) -> PyResult<bool> {
        let inner = self.inner.0.lock().unwrap();
        inner.is_enabled()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_properties(&self) -> PyResult<HashMap<String, String>> {
        let inner = self.inner.0.lock().unwrap();
        inner.get_properties()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_children(&self) -> PyResult<Vec<Py<PyUIElement>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let children = inner.get_children()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            
            Ok(children.into_iter()
                .map(|element| Py::new(py, PyUIElement { 
                    inner: Arc::new(ThreadSafe::new(element))
                }).unwrap())
                .collect())
        })
    }

    fn set_text(&self, text: &str) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_text(text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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

    fn focused_window(&self) -> PyResult<Py<PyWindow>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            let window = inner.get_focused_window()
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
    Ok(())
} 