use crate::{Function, Tensor, Variable};

type Data = Tensor;

pub struct Square;

impl Function for Square {
    fn forward(&self, xs: &Vec<Variable>) -> Vec<Data> {
        assert!(xs.len() == 1);
        vec![xs[0].multiply(&xs[0])]
    }

    fn backward(&self, xs: &Vec<Variable>, gys: &Vec<Variable>) -> Vec<Variable> {
        vec![Variable::new(
            gys[0].multiply(&xs[0]).multiply_with_scalar(2.0),
        )]
    }
}

pub struct Exp;

impl Function for Exp {
    fn forward(&self, xs: &Vec<Variable>) -> Vec<Data> {
        assert!(xs.len() == 1);
        vec![xs[0].map(|x| x.exp())]
    }

    fn backward(&self, xs: &Vec<Variable>, gys: &Vec<Variable>) -> Vec<Variable> {
        vec![Variable::new(gys[0].multiply(&xs[0].map(|x| x.exp())))]
    }
}

pub struct Sum;

impl Function for Sum {
    fn forward(&self, xs: &Vec<Variable>) -> Vec<Data> {
        assert!(xs.len() >= 1);
        let mut y = xs[0].inner.data.clone();
        for x in xs.iter().skip(1) {
            y = &y + &x.inner.data;
        }
        vec![y]
    }

    fn backward(&self, xs: &Vec<Variable>, gys: &Vec<Variable>) -> Vec<Variable> {
        (0..xs.len()).map(|_| gys[0].clone()).collect()
    }
}

#[test]
fn test_sum() {
    {
        let x = Variable::new(1.0.into());
        let y = Variable::new(2.0.into());
        let z = Variable::new(3.0.into());
        let xs = vec![x.clone(), y.clone(), z.clone()];
        let ys = Sum.call(xs);
        assert_eq!(*ys[0], 6.0.into());

        ys[0].set_grad(Variable::new(1.0.into()));
        ys[0].backward();
        assert_eq!(*x.get_grad().unwrap(), 1.0.into());
        assert_eq!(*y.get_grad().unwrap(), 1.0.into());
        assert_eq!(*z.get_grad().unwrap(), 1.0.into());
    }
    {
        let x = Variable::new(3.0.into());
        Sum.call(vec![x.clone(), x.clone()]);
        let ys = Sum.call(vec![x.clone(), x.clone()]);
        assert_eq!(*ys[0], 6.0.into());

        ys[0].set_grad(Variable::new(1.0.into()));
        ys[0].backward();
        assert_eq!(*x.get_grad().unwrap(), 2.0.into());
    }
}
