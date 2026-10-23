//! A tensor kept in memory as its shape and numbers, so it can be saved with a voice and fed back
//! to ONNX Runtime.

use std::fmt::{Debug, Display};

use ort::value::{DynValue, PrimitiveTensorElementType, Tensor};

use crate::{Error, Result};

pub(super) fn engine_error(e: impl Display) -> Error {
    Error::Engine(format!("chatterbox: {e}"))
}

/// The shape and the numbers of a tensor, in row-major order.
#[derive(Debug, Clone, PartialEq)]
pub struct Array<T> {
    pub shape: Vec<i64>,
    pub data: Vec<T>,
}

impl<T: PrimitiveTensorElementType + Debug + Clone + 'static> Array<T> {
    /// Copies the tensor out of a model output.
    pub(super) fn from_value(value: &DynValue) -> Result<Array<T>> {
        let (shape, data) = value.try_extract_tensor::<T>().map_err(engine_error)?;
        Ok(Array {
            shape: shape.to_vec(),
            data: data.to_vec(),
        })
    }

    /// A tensor to give to a model as input.
    pub(super) fn tensor(&self) -> Result<Tensor<T>> {
        Tensor::from_array((self.shape.clone(), self.data.clone())).map_err(engine_error)
    }
}
