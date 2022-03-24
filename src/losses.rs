use std::ops::Sub;

use ndarray::Axis;

use crate::functions::*;
use crate::nn::Softmax;
use crate::*;

pub fn naive_mean_squared_error(x0: Variable, x1: Variable) -> Variable {
    let x = call!(Pow::new(2.0), call!(Sub, x0, x1));
    call!(
        Div,
        call!(Sum::new((0..x.ndim()).collect(), false), x),
        Variable::new(scalar(x.shape().iter().product::<usize>() as f32))
    )
}

pub struct SoftmaxCrossEntropy {
    t: Vec<usize>,
}

impl SoftmaxCrossEntropy {
    pub fn new(t: Vec<usize>) -> Self {
        Self { t }
    }
}

impl Function for SoftmaxCrossEntropy {
    fn forward(&self, xs: &[Variable]) -> Vec<Variable> {
        assert_eq!(xs.len(), 1);
        let x = &*xs[0];

        let n = x.shape().iter().take(x.ndim() - 1).product();
        let log_z = log_sum_exp(&*x);
        let log_p = x.to_shape((n, x.shape()[x.ndim() - 1])).unwrap();
        let mut y = 0.0;
        for i in 0..n {
            y -= log_p[[i, self.t[i]]] - log_z[i];
        }
        vec![Variable::new(scalar(y / n as f32))]
    }

    fn backward(
        &self,
        xs: &Vec<Variable>,
        ys: &Vec<Variable>,
        gys: &Vec<Variable>,
    ) -> Vec<Variable> {
        #![allow(unused_variables)]

        let n: usize = xs[0].shape().iter().take(xs[0].ndim() - 1).product();
        let class_num = xs[0].shape()[xs[0].ndim() - 1];
        let gy = call!(Mul, gys[0], Variable::new(scalar(1.0 / n as f32)));
        let y = call!(Softmax, xs[0]);
        let t_onehot = Variable::new(onehot(&self.t, class_num));
        vec![call!(Mul, call!(Sub, y, t_onehot), gy)]
    }
}

#[test]
fn test_softmax_cross_entropy() {
    let x = backprop(ndarray::array![[0.1, 0.2, 0.3], [0.0, 0.0, 100.0]].into_tensor());
    let t = vec![1, 2];
    let loss = call!(SoftmaxCrossEntropy::new(t), x.clone());
    dbg!(&*loss);

    let grads = gradients(&[loss], &vec![x.clone()], false);
    dbg!(&*grads[0]);
}

// max(x) + log(sum(exp(x - max(x))))
pub fn log_sum_exp(x: &Tensor) -> Tensor {
    let ndim = x.ndim();
    let x_max = x.map_axis(Axis(ndim - 1), |x| {
        *x.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
    });

    (&x_max
        + (x.sub(&x_max.view().insert_axis(Axis(ndim - 1))))
            .map(|x| x.exp())
            .sum_axis(Axis(ndim - 1))
            .map(|x| x.ln()))
    .into_tensor()
}

#[test]
fn test_log_sum_exp() {
    let x = ndarray::array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0],].into_tensor();
    let y = log_sum_exp(&x);
    assert_eq!(y.shape(), &[2]);
    assert!(
        (y[0] - (3.0 + ((1.0f32 - 3.0).exp() + (2.0f32 - 3.0).exp() + (3.0f32 - 3.0).exp()).ln()))
            .abs()
            < 1e-6
    );
}

pub fn onehot(t: &[usize], size: usize) -> Tensor {
    let mut y = ndarray::Array::zeros([t.len(), size]);
    for i in 0..t.len() {
        y[[i, t[i]]] = 1.0;
    }
    y.into_tensor()
}
