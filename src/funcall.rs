use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use crate::{backprop, call, functions::Add, Backward, Function, Tensor, Variable, VariableInner};

pub struct Funcall {
    pub(crate) backward: Box<dyn Backward>,
    pub(crate) xs: Vec<Variable>,
    pub(crate) ys: Vec<Weak<VariableInner>>,
}

impl Funcall {
    pub fn new(backward: Box<dyn Backward>, xs: Vec<Variable>, ys: &Vec<Variable>) -> Self {
        Self {
            backward,
            xs,
            ys: ys.iter().map(|y| Arc::downgrade(&y.inner)).collect(),
        }
    }

    pub fn get_ys(&self) -> Vec<Variable> {
        self.ys
            .iter()
            .map(|y| Variable {
                inner: y.upgrade().unwrap(),
            })
            .collect()
    }
}

pub fn sort_for_backward(mut fcs: Vec<Arc<Funcall>>) -> Vec<Arc<Funcall>> {
    let mut sorted = Vec::with_capacity(fcs.len());
    let ys = fcs.iter().flat_map(|fc| fc.get_ys()).collect::<Vec<_>>();
    let mut visited: Vec<_> = fcs
        .iter()
        .flat_map(|fc| &fc.xs)
        .filter(|v| !ys.contains(v))
        .cloned()
        .collect();
    while !fcs.is_empty() {
        let (a, b): (Vec<_>, _) = fcs
            .into_iter()
            .partition(|fc| fc.xs.iter().all(|x| visited.contains(&x)));
        if a.is_empty() {
            panic!("cycle detected");
        }
        visited.extend(a.iter().flat_map(|fc| fc.get_ys()));
        sorted.extend(a);
        fcs = b;
    }
    sorted.reverse();
    sorted
}

pub fn gradients(ys: &[Variable], xs: &[Variable], create_graph: bool) -> Vec<Variable> {
    let mut grads = HashMap::new();

    for y in ys.iter() {
        grads.insert(
            Arc::as_ptr(&y.inner),
            if create_graph {
                backprop(Tensor::ones(y.shape()))
            } else {
                Variable::new(Tensor::ones(y.shape()))
            },
        );
    }

    let funcalls = collect_funcalls(ys.to_vec());
    for fc in sort_for_backward(funcalls) {
        let ys = fc.get_ys();
        let gys = ys
            .iter()
            .map(|y| grads[&Arc::as_ptr(&y.inner)].clone())
            .collect();

        let gxs = fc.backward.backward(&fc.xs, &ys, &gys);

        if !create_graph {
            for gx in &gxs {
                gx.cut_chain();
            }
        }

        for (x, gx) in fc.xs.iter().zip(gxs.iter()) {
            match grads.entry(Arc::as_ptr(&x.inner)) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    *entry.get_mut() = call!(Add, entry.get(), gx);
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(gx.clone());
                }
            }
        }

        for y in &ys {
            if !xs.contains(&y) {
                grads.remove(&Arc::as_ptr(&y.inner));
            }
        }
    }

    xs.iter()
        .map(|x| {
            grads
                .get(&Arc::as_ptr(&x.inner))
                .unwrap_or_else(|| panic!("grad not found {}", x.get_name()))
                .clone()
        })
        .collect()
}

pub(crate) fn collect_funcalls(mut vars: Vec<Variable>) -> Vec<Arc<Funcall>> {
    let mut funcall_vec = Vec::new();
    let mut closed_vars = Vec::new();
    while let Some(var) = vars.pop() {
        if closed_vars.contains(&var) {
            continue;
        }
        closed_vars.push(var.clone());

        if let Some(creator) = var.inner.attrs.lock().unwrap().creator.clone() {
            vars.extend(creator.xs.iter().cloned());
            vars.extend(creator.get_ys());
            funcall_vec.push(creator);
        }
    }
    funcall_vec
}

pub fn collect_variables(vars: Vec<Variable>) -> Vec<Variable> {
    let fcs = collect_funcalls(vars);
    let mut vars: Vec<_> = fcs.iter().flat_map(|fc| fc.xs.iter()).cloned().collect();
    vars.dedup();
    vars
}

#[test]
fn test_collect_funcalls() {
    use crate::{backprop, functions, scalar, Function};
    let x = backprop(scalar(1.0));
    let y = Variable::new(scalar(2.0));
    let z = Variable::new(scalar(3.0));
    let f = functions::Add.call(vec![x.clone(), y.clone()]);
    let g = functions::Add.call([f.clone(), vec![z.clone()]].concat());
    let f = functions::Add.call([g.clone(), vec![x.clone()]].concat());
    let funcall_vec = collect_funcalls(vec![f[0].clone()]);
    assert_eq!(funcall_vec.len(), 4);
}
