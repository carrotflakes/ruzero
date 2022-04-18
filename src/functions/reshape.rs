use crate::*;

pub fn reshape(x: &Computed, shape: impl Into<Vec<usize>>) -> Computed {
    let shape = shape.into();
    let y = Computed::new((**x).reshape(shape.as_slice()));

    chain(
        &[x.clone()],
        &[y.clone()],
        false,
        "reshape",
        move |xs, _ys, gys| {
            let gx = gys[0].reshape(xs[0].shape());
            vec![gx]
        },
    );

    y
}

pub struct Reshape {
    pub shape: Vec<usize>,
    original_shape: Vec<usize>,
}

impl Reshape {
    pub fn new(shape: Vec<usize>) -> Self {
        Self {
            shape,
            original_shape: Vec::new(),
        }
    }
}

impl Function for Reshape {
    fn forward(&self, xs: &[Computed]) -> Vec<Computed> {
        assert!(xs.len() == 1);

        vec![(*xs[0]).reshape(self.shape.as_slice()).into()]
    }

    fn backward(
        &self,
        xs: &Vec<crate::Computed>,
        ys: &Vec<Computed>,
        gys: &Vec<crate::Computed>,
    ) -> Vec<crate::Computed> {
        #![allow(unused_variables)]

        vec![gys[0].reshape(self.original_shape.as_slice())]
    }

    fn into_backward(mut self, xs: &Vec<crate::Computed>) -> Box<dyn crate::Backward>
    where
        Self: Sized + 'static,
    {
        self.original_shape = xs[0].shape().to_vec();
        Box::new(self)
    }
}

#[test]
fn test() {
    {
        let x = backprop(ndarray::array![[1., 2., 3.], [4., 5., 6.]].into_ndarray());
        let ys = Reshape::new(vec![3, 2]).call(vec![x.clone()]);
        dbg!(&*ys[0]);
        assert_eq!(ys[0].shape(), &[3, 2]);

        let grads = gradients(&ys, &[x.clone()], false);
        assert_eq!(grads[0].shape(), &[2, 3]);
    }
}
