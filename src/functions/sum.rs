use ndarray::Axis;

use crate::*;

use super::Broadcast;

pub struct Sum {
    // NOTE: axes are in order
    pub axes: Vec<usize>,
    pub keep_dim: bool,
    original_shape: Vec<usize>,
}

impl Sum {
    pub fn new(axes: Vec<usize>, keep_dim: bool) -> Self {
        assert!(axes.windows(2).all(|w| w[0] < w[1]));
        Self {
            axes,
            keep_dim,
            original_shape: Vec::new(),
        }
    }
}

impl Function for Sum {
    fn forward(&self, xs: &[Variable]) -> Vec<Variable> {
        assert!(xs.len() == 1);

        let mut x = (*xs[0]).to_owned();
        for axis in self.axes.iter().rev() {
            x = x.sum_axis(Axis(*axis));
            if self.keep_dim {
                x.insert_axis_inplace(Axis(*axis));
            }
        }

        vec![x.into_tensor().into()]
    }

    fn backward(
        &self,
        xs: &Vec<Variable>,
        ys: &Vec<Variable>,
        gys: &Vec<Variable>,
    ) -> Vec<Variable> {
        #![allow(unused_variables)]

        Broadcast::new(self.original_shape.clone()).call(vec![gys[0].clone()])
    }

    fn into_backward(mut self, xs: &Vec<Variable>) -> Box<dyn Backward>
    where
        Self: Sized + 'static,
    {
        self.original_shape = xs[0].shape().to_vec();
        Box::new(self)
    }
}

pub fn sum_axes_to_desire(src_shape: &[usize], dst_shape: &[usize]) -> Vec<usize> {
    let mut axes = Vec::new();
    let mut target = dst_shape.to_vec();
    for (axis, size) in src_shape.iter().enumerate() {
        if let Some(s) = target.first() {
            if s == size {
                target.remove(0);
                continue;
            }
        }
        axes.push(axis);
    }
    axes
}

#[test]
fn test() {
    {
        let x = Variable::new(ndarray::array![[1., 2., 3.], [4., 5., 6.]].into_tensor());
        let ys = Sum::new(vec![0], false).call(vec![x.clone()]);
        assert_eq!(ys[0].shape(), &[3]);
        assert_eq!(&*ys[0], &ndarray::array![5., 7., 9.].into_tensor());
    }

    {
        let x = Variable::new(ndarray::array![[1., 2., 3.], [4., 5., 6.]].into_tensor());
        let ys = Sum::new(vec![1], false).call(vec![x.clone()]);
        assert_eq!(ys[0].shape(), &[2]);
        assert_eq!(&*ys[0], &ndarray::array![6., 15.].into_tensor());
    }
}