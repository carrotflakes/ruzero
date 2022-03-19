use crate::*;

pub struct CreateGraph {
    y: Tensor,
}

impl CreateGraph {
    pub fn new(y: Tensor) -> Self {
        Self { y }
    }
}

impl Function for CreateGraph {
    fn forward(&self, xs: &[Variable]) -> Vec<Variable> {
        #![allow(unused_variables)]

        vec![Variable::new(self.y.clone())]
    }

    fn backward(
        &self,
        xs: &Vec<Variable>,
        ys: &Vec<Variable>,
        gys: &Vec<Variable>,
    ) -> Vec<Variable> {
        #![allow(unused_variables)]

        vec![]
    }

    const IS_FORCE_CREATE_GRAPH: bool = true;
}