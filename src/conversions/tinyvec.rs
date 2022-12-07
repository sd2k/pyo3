#![cfg(feature = "tinyvec")]

//! Conversions to and from [tinyvec](https://docs.rs/tinyvec/)’s `TinyVec`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! pyo3 = { version = "*", features = ["tinyvec"] }
//! # Minimum version 1.6.1 required.
//! tinyvec = "1.6.1"
use tinyvec::{Array, TinyVec};

use crate::ffi;
use crate::types::list::new_from_iter;
use crate::{
    exceptions::PyTypeError,
    types::{PySequence, PyString},
    AsPyPointer, FromPyObject, IntoPy, PyAny, PyDowncastError, PyObject, PyResult, Python,
    ToPyObject,
};

impl<A> ToPyObject for TinyVec<A>
where
    A: Array,
    A::Item: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_slice().to_object(py)
    }
}

impl<A> IntoPy<PyObject> for TinyVec<A>
where
    A: Array,
    A::Item: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut iter = self.into_iter().map(|e| e.into_py(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }
}

impl<'a, A> FromPyObject<'a> for TinyVec<A>
where
    A: Array,
    A::Item: FromPyObject<'a>,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        if let Ok(true) = obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err("Can't extract `str` to `Vec`"));
        }
        extract_sequence(obj)
    }
}

fn extract_sequence<'s, A>(obj: &'s PyAny) -> PyResult<TinyVec<A>>
where
    A: Array,
    A::Item: FromPyObject<'s>,
{
    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    let seq: &PySequence = unsafe {
        if ffi::PySequence_Check(obj.as_ptr()) != 0 {
            obj.downcast_unchecked()
        } else {
            return Err(PyDowncastError::new(obj, "Sequence").into());
        }
    };

    let mut v = TinyVec::with_capacity(seq.len().unwrap_or(0));
    for item in seq.iter()? {
        v.push(item?.extract::<A::Item>()?);
    }
    Ok(v)
}
