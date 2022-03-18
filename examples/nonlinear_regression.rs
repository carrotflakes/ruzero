use std::collections::HashMap;

use ndarray::{Array, array};
use ndarray_rand::{rand::SeedableRng, rand_distr::Uniform, RandomExt};
use ruzero::{functions::*, nn::*, *};

fn main() {
    let mut rng = rand_isaac::Isaac64Rng::seed_from_u64(42);
    let n = 100;

    let x = Variable::new(
        Array::random_using((n, 1), Uniform::new(0.0, 1.0), &mut rng).into_dyn(),
    )
    .named("x");
    let y = call!(
        Add,
        call!(Sin, call!(Mul, x, Variable::new(scalar(2.0 * 3.14)))),
        Variable::new(Array::random_using((n, 1), Uniform::new(0.0, 1.0), &mut rng).into_dyn())
    )
    .named("y");

    let mut l1 = Layer::new(1, 10);
    let mut l2 = Layer::new(10, 1);

    let start = std::time::Instant::now();

    // let mut ll = 1000.0;

    for i in 0..10000 {
        let h = l1.forward(x.clone());
        let h = call!(Sigmoid, h).named("hidden");
        let y_ = l2.forward(h);
        // dbg!(&*y_);
        if i == 0 {
            graph(&[y_.clone()], "graph");
        }

        let loss = mean_squared_error(y.clone(), y_.clone());
        if i % 1000 == 0 {
            println!("loss: {}", loss[[]]);
        }
        // if loss[[]] > ll {
        //     dbg!("loss is not decreasing");
        //     return;
        // }
        // ll = loss[[]];
        
        // graph(&[loss.clone()], format!("loss{}", i));

        let trainables = l1.all_params()
        .into_iter()
        .chain(l2.all_params())
        .collect::<Vec<_>>();
        let gs = gradients(&[loss.clone()], &trainables, false);
        let gs = HashMap::from_iter(trainables.iter().cloned().zip(gs.into_iter()));

        // dbg!(&*l1.w.get_grad().unwrap());
        // dbg!(&*l1.b.get_grad().unwrap());
        let lr = 0.1;
        l1.update(lr, &gs);
        l2.update(lr, &gs);
    }
    for i in 0..20 {
        let x = Variable::new(
            array![[i as f32 / 20.0]].into_dyn()
        );
        let h = l1.forward(x);
        let h = sigmoid_simple(h).named("hidden");
        let y_ = l2.forward(h);
        println!("{}", &*y_);
    }
    println!("elapsed: {:?}", start.elapsed());
}

fn mean_squared_error(x0: Variable, x1: Variable) -> Variable {
    let x = call!(Pow::new(2.0), call!(Sub, x0, x1));
    call!(
        Div,
        call!(SumTo::new((0..x.ndim()).collect()), x),
        Variable::new(scalar(x.shape().iter().product::<usize>() as f32))
    )
}

pub struct Layer {
    pub w: Variable,
    pub b: Variable,
}

impl Layer {
    pub fn new(input: usize, output: usize) -> Self {
        Self {
            w: backprop(Array::random((input, output), Uniform::new(0., 0.01)).into_dyn())
                .named("param w"),
            b: backprop(Array::zeros(output).into_dyn()).named("param b"),
        }
    }

    pub fn forward(&self, x: Variable) -> Variable {
        call!(Add, call!(Matmul, x, self.w), self.b).named("return")
    }

    pub fn update(&mut self, lr: f32, gs: &HashMap<Variable, Variable>) {
        self.w = backprop(&*self.w - &*gs[&self.w] * lr);
        self.b = backprop(&*self.b - &*gs[&self.b] * lr);
    }

    pub fn all_params(&self) -> Vec<Variable> {
        vec![self.w.clone(), self.b.clone()]
    }
}

fn graph(vars: &[Variable], name: impl ToString) {
    let f = std::fs::File::create(name.to_string() + ".dot").unwrap();
    let mut w = std::io::BufWriter::new(f);
    ruzero::export_dot::write_dot(&mut w, vars, &mut |v| {
        // format!("{} {}", v.get_name(), (*v).to_string())
        // v.get_name().to_string()
        format!("{} {:?}", v.get_name(), v.shape())
    })
    .unwrap();
}
