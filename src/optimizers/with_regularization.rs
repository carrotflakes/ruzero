use crate::{regularizers::Regularizer, *};

#[derive(Clone)]
pub struct WithRegularization<O: Optimizer, R: Regularizer> {
    pub optimizer: O,
    pub regularizer: R,
}

impl<O: Optimizer, R: Regularizer> WithRegularization<O, R> {
    pub fn new(optimizer: O, regularizer: R) -> Self {
        Self {
            optimizer,
            regularizer,
        }
    }
}

impl<O: Optimizer, R: Regularizer> Optimizer for WithRegularization<O, R> {
    type State = O::State;

    fn new_state(&self, shape: &[usize]) -> Self::State {
        self.optimizer.new_state(shape)
    }

    fn update(&mut self, tensor: &mut Tensor, state: &mut Self::State, grad: &NDArray) {
        let grad = (grad + &*self.regularizer.grad(tensor)).into_ndarray();
        self.optimizer.update(tensor, state, &grad);
    }
}

#[test]
fn test() {
    let optimizer = super::AdamOptimizer::new_with_params(0.01, 0.9, 0.999);
    let optimizer = WithRegularization::new(optimizer, regularizers::L1::new(0.01));
    super::test_optimizer(optimizer);
}
