use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use uia_interaction::core::{UIAutomation, Window, UIElement};
use uia_interaction::factory::UIAutomationFactory;

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

    fn set_text(&self, text: &str) -> PyResult<()> {
        let inner = self.inner.0.lock().unwrap();
        inner.set_text(text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn click(&self) -> PyResult<()> {
        // TODO: Implement click method in core trait
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>("Click not implemented yet"))
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

    fn focused_element(&self) -> PyResult<Option<Py<PyUIElement>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            match inner.get_focused_element() {
                Ok(element) => Ok(Some(Py::new(py, PyUIElement { 
                    inner: Arc::new(ThreadSafe::new(element))
                })?)),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
            }
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

    fn find_by_name(&self, name: &str) -> PyResult<Option<Py<PyUIElement>>> {
        Python::with_gil(|py| {
            let inner = self.inner.0.lock().unwrap();
            match inner.find_element_by_name(name) {
                Ok(element) => Ok(Some(Py::new(py, PyUIElement { 
                    inner: Arc::new(ThreadSafe::new(element))
                })?)),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
            }
        })
    }
} 