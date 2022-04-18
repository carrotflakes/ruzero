use crate::*;

pub fn transpose(x: &Computed, axes: impl Into<Vec<usize>>) -> Computed {
    let axes = axes.into();

    let y = Computed::new(x.view().permuted_axes(&*axes).into_ndarray());

    chain(
        &[x.clone()],
        &[y.clone()],
        false,
        "transpose",
        move |_xs, _ys, gys| {
            let gx = gys[0].transpose(
                (0..axes.len())
                    .map(|i| axes.iter().position(|j| *j == i).unwrap())
                    .collect::<Vec<_>>(),
            );
            vec![gx]
        },
    );

    y
}

pub struct Transpose {
    axes: Vec<usize>,
}

impl Transpose {
    pub fn new(axes: Vec<usize>) -> Self {
        assert!((0..axes.len()).all(|i| axes.contains(&i)));

        Self { axes }
    }
}

impl Function for Transpose {
    fn forward(&self, xs: &[Computed]) -> Vec<Computed> {
        assert!(xs.len() == 1);

        vec![xs[0]
            .view()
            .permuted_axes(&*self.axes)
            .into_ndarray()
            .into()]
    }

    fn backward(&self, xs: &Vec<Computed>, ys: &Vec<Computed>, gys: &Vec<Computed>) -> Vec<Computed> {
        #![allow(unused_variables)]

        Transpose::new(
            (0..self.axes.len())
                .map(|i| self.axes.iter().position(|j| *j == i).unwrap())
                .collect::<Vec<_>>(),
        )
        .call(vec![gys[0].clone()])
    }
}

#[test]
fn test() {
    {
        let x = backprop(ndarray::Array::zeros([1, 2, 3]).into_ndarray());
        let y = call!(Transpose::new(vec![1, 2, 0]), x);
        assert_eq!(y.shape(), &[2, 3, 1]);

        let grads = gradients(&[y], &[x], false);
        assert_eq!(grads[0].shape(), &[1, 2, 3]);
    }
}
