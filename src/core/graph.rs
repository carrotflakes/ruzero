use std::{collections::HashMap, sync::Arc};

use super::{backprop, FunctionCall, NDArray, Tensor};

pub fn gradients(ys: &[Tensor], xs: &[Tensor], create_graph: bool) -> Vec<Tensor> {
    let mut grads = HashMap::new();

    for y in ys.iter() {
        grads.insert(
            Arc::as_ptr(&y.inner),
            if create_graph {
                backprop(NDArray::ones(y.shape()))
            } else {
                Tensor::new(NDArray::ones(y.shape()))
            },
        );
    }

    let function_calls = collect_function_calls(ys.to_vec());
    for fc in sort_for_backward(function_calls) {
        let ys = fc.get_ys();
        let gys = ys
            .iter()
            .map(|y| grads[&Arc::as_ptr(&y.inner)].clone())
            .collect();

        let gxs = fc.backward.backward(&fc.xs, &ys, &gys);

        if !create_graph {
            for gx in &gxs {
                gx.unchain();
            }
        }

        if fc.xs.len() != gxs.len() {
            panic!(
                "backward of {} has {} inputs, but {} gradients returned",
                fc.backward.get_function_name(),
                fc.xs.len(),
                gxs.len()
            );
        }

        for (x, gx) in fc.xs.iter().zip(gxs.iter()) {
            match grads.entry(Arc::as_ptr(&x.inner)) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry.get() + gx;
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

pub fn collect_variables(vars: Vec<Tensor>) -> Vec<Tensor> {
    let fcs = collect_function_calls(vars);
    let mut vars: Vec<_> = fcs.iter().flat_map(|fc| fc.xs.iter()).cloned().collect();
    vars.dedup();
    vars
}

pub(crate) fn sort_for_backward(mut fcs: Vec<Arc<FunctionCall>>) -> Vec<Arc<FunctionCall>> {
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

pub(crate) fn collect_function_calls(mut vars: Vec<Tensor>) -> Vec<Arc<FunctionCall>> {
    let mut function_call_vec = Vec::new();
    let mut closed_vars = Vec::new();
    while let Some(var) = vars.pop() {
        if closed_vars.contains(&var) {
            continue;
        }
        closed_vars.push(var.clone());

        if let Some(creator) = var.inner.attrs.lock().unwrap().creator.clone() {
            vars.extend(creator.xs.iter().cloned());
            vars.extend(creator.get_ys());
            function_call_vec.push(creator);
        }
    }
    function_call_vec
}

#[test]
fn test_collect_function_calls() {
    use crate::{backprop, functions, scalar, Function};
    let x = backprop(scalar(1.0));
    let y = Tensor::new(scalar(2.0));
    let z = Tensor::new(scalar(3.0));
    let f = functions::Add.call(vec![x.clone(), y.clone()]);
    let g = functions::Add.call([f.clone(), vec![z.clone()]].concat());
    let f = functions::Add.call([g.clone(), vec![x.clone()]].concat());
    let function_call_vec = collect_function_calls(vec![f[0].clone()]);
    assert_eq!(function_call_vec.len(), 4);
}