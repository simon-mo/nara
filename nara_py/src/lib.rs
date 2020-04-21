use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::wrap_pyfunction;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[pyfunction]
fn call_closure(py: Python, o: PyObject) -> PyResult<bool> {
    o.call(py, (), None);
    Ok(true)
}

#[pyclass]
struct Bencher {
    map: HashMap<u32, Instant>,
}


#[pyfunction]
fn call_closure_par(py: Python, os: Vec<PyObject>) -> PyResult<bool> {
    let py_futures: Vec<Result<PyObject, bool>> = py.allow_threads(move || {
        os.par_iter() // <-- just change that!
            .map(|o| {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let res = o.call(py, (), None);
                match res {
                    Ok(obj) => Ok(obj),
                    Err(e) => {
                        e.print(py);
                        Err(false)
                    }
                }
            })
            .collect()
    });

    Ok(true)
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn profiler_py(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(call_closure))?;
    m.add_wrapped(wrap_pyfunction!(call_closure_par))?;

    Ok(())
}
