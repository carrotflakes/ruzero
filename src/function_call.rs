use std::sync::{Arc, Weak};

use crate::{tensor::TensorInner, Backward, Tensor};

pub struct FunctionCall {
    pub(crate) backward: Box<dyn Backward>,
    pub(crate) xs: Vec<Tensor>,
    pub(crate) ys: Vec<Weak<TensorInner>>,
}

impl FunctionCall {
    pub fn new(backward: Box<dyn Backward>, xs: Vec<Tensor>, ys: &[Tensor]) -> Self {
        Self {
            backward,
            xs,
            ys: ys.iter().map(|y| Arc::downgrade(&y.inner)).collect(),
        }
    }

    pub fn get_ys(&self) -> Vec<Tensor> {
        self.ys
            .iter()
            .map(|y| Tensor {
                inner: y.upgrade().unwrap(),
            })
            .collect()
    }
}