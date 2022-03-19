use crate::*;

pub struct Neg;

impl Function for Neg {
    fn forward(&self, xs: &[Variable]) -> Vec<Variable> {
        assert!(xs.len() == 1);
        vec![xs[0].map(|x| -x).into_tensor().into()]
    }

    fn backward(
        &self,
        xs: &Vec<Variable>,
        ys: &Vec<Variable>,
        gys: &Vec<Variable>,
    ) -> Vec<Variable> {
        #![allow(unused_variables)]

        Neg.call(gys.clone())
    }
}
