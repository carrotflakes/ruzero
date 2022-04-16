mod backward;
mod function_call;
pub mod graph;
mod optimize;
mod optimizer;
pub mod param;
mod tensor;

pub use backward::{chain, Backward, Function};
pub use function_call::FunctionCall;
pub use graph::gradients;
pub use optimize::{optimize, GradientsAccumulator};
pub use optimizer::Optimizer;
pub use param::Param;
pub use tensor::Tensor;

use crate::NDArray;

pub fn backprop(x: NDArray) -> Tensor {
    let y = Tensor::new(x);
    chain(&[], &[y.clone()], true, "backprop", |_, _, _| vec![]);
    y
}