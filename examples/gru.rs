mod data;
mod training;

use ndarray::prelude::*;
use ndarray_rand::{
    rand::{prelude::*, SeedableRng},
    rand_distr::Uniform,
    RandomExt,
};
use tensorflake::{
    functions::*,
    losses::SoftmaxCrossEntropy,
    ndarray_util::argmax,
    nn::{activations::Sigmoid, *},
    *,
};

use crate::training::TrainingConfig;

fn main() {
    // let mut data = data::arith::make(10000, 42, 15);
    // let vocab = plane_corpus::Vocab::new(arith::CHARS);
    let data = data::plane_corpus::load("data/corpus_en.txt").unwrap();
    let vocab = data::plane_corpus::Vocab::new(&data);
    let data = data::plane_corpus::windows(&data, 50, 25);
    let data = data
        .into_iter()
        .filter(|str| str.len() == 50)
        .collect::<Vec<_>>();
    let vocab_size = vocab.size();
    println!("data size: {}", data.len());
    println!("vocab size: {}", vocab_size);

    // let optimizer = optimizers::SGDOptimizer::new();
    // let lr = 0.1;
    let optimizer = optimizers::AdamOptimizer::new();
    // let optimizer = optimizers::WithRegularization::new(optimizer, regularizers::L2::new(0.001));
    let lr = 0.0001;

    let norm =
        normalization::Normalization::new(vec![0, 1], 0.001, optimizers::AdamOptimizer::new());

    let mut rng = rand_isaac::Isaac64Rng::seed_from_u64(42);
    let mut param_gen = {
        let mut rng = rng.clone();
        move || {
            rng.gen::<u32>();
            let mut rng = rng.clone();
            let optimizer = optimizer.clone();
            move |shape: &[usize]| -> Param {
                let t = Array::random_using(shape, Uniform::new(0., 0.01), &mut rng).into_ndarray();
                Param::new(t, optimizer.clone())
            }
        }
    };

    let embedding_size = 64;
    let state_size = 128;
    let embedding = Embedding::new(embedding_size, vocab_size, &mut param_gen());
    let model = Gru::new(embedding_size, state_size, &mut param_gen());
    let linear = Linear::new(state_size, vocab_size, &mut param_gen(), &mut param_gen());
    // let output_fn = |x: Tensor| linear.call(x, true);
    let output_fn = |x: Tensor| linear.call(norm.call(x, true), true);

    let start = std::time::Instant::now();

    let mut training = TrainingConfig {
        epoch: 30,
        train_data: data,
        batch_size: 100,
        parallel: true,
        ..Default::default()
    }
    .build();
    while !training.is_end() {
        training.fit_one_epoch(|strs, ctx| {
            let initial_state = Tensor::new(NDArray::zeros(&[strs.len(), state_size][..]));
            let eqp = 10;
            let mut x = vec![vec![]; 50 - 1];
            let mut t = vec![vec![]; 50 - eqp];
            for str in strs.iter() {
                let y = vocab.encode(str);
                for (j, c) in y.iter().take(str.len() - 1).enumerate() {
                    x[j].push(*c);
                }
                for (j, c) in y.iter().skip(eqp).enumerate() {
                    t[j].push(*c);
                }
            }
            if x[0].len() != x[50 - 2].len() {
                for s in strs {
                    println!("{}", s.len());
                }
                panic!("length unmatch")
            }
            let x = x
                .into_iter()
                .map(|x| {
                    // onehot(&Array::from_shape_vec([strs.len()], x).unwrap(), vocab_size).into()
                    embedding.call(x, ctx.train)
                })
                .collect::<Vec<_>>();
            let t = t.into_iter().flatten().collect();
            let y = model.encode(initial_state, &x);
            let yy = Concat::new(0)
                .call(y.iter().skip(eqp - 1).cloned().collect())
                .pop()
                .unwrap();
            let yy = output_fn(yy);
            let loss = call!(SoftmaxCrossEntropy::new(t), yy);
            if ctx.train {
                optimize(&loss, lr * 0.95f32.powi(ctx.epoch as i32));
            }
            ctx.count(strs.len());
            ctx.add_metric(metrics::Loss::new(loss[[]], strs.len()));
        });

        let y = model.decode(
            Tensor::new(NDArray::random_using(
                &[1, state_size][..],
                Uniform::new(0.0, 1.0),
                &mut rng,
            )),
            output_fn,
            |x|
            //onehot(&argmax(&*x), vocab_size).into()
            embedding.call(argmax(&*x).into_raw_vec(), false),
            50,
        );
        let str: String = y
            .iter()
            .map(|x| vocab.decode(&argmax(&*x).into_raw_vec()))
            .collect();
        println!("{}", str);
    }

    println!("time: {:?}", start.elapsed());
}

pub struct Gru {
    pub input_size: usize,
    pub state_size: usize,
    pub ws: [Param; 3],
    pub us: [Param; 3],
    pub bs: [Param; 3],
}

impl Gru {
    pub fn new(
        input_size: usize,
        state_size: usize,
        param_gen: &mut impl FnMut(&[usize]) -> Param,
    ) -> Self {
        Self {
            input_size,
            state_size,
            ws: [
                param_gen(&[input_size, state_size]),
                param_gen(&[input_size, state_size]),
                param_gen(&[input_size, state_size]),
            ],
            us: [
                param_gen(&[state_size, state_size]),
                param_gen(&[state_size, state_size]),
                param_gen(&[state_size, state_size]),
            ],
            bs: [
                param_gen(&[state_size]),
                param_gen(&[state_size]),
                param_gen(&[state_size]),
            ],
        }
    }

    pub fn step(&self, x: Tensor, state: Tensor) -> Tensor {
        let z = call!(
            Sigmoid,
            x.matmul(&self.ws[0].get_tensor())
                + state.matmul(&self.us[0].get_tensor())
                + self.bs[0].get_tensor()
        );
        let r = call!(
            Sigmoid,
            x.matmul(&self.ws[1].get_tensor())
                + state.matmul(&self.us[1].get_tensor())
                + self.bs[1].get_tensor()
        );
        (Tensor::new(NDArray::ones(z.shape())) - z.clone()) * state.clone()
            + z * call!(
                Tanh,
                x.matmul(&self.ws[2].get_tensor())
                    + (r * state).matmul(&self.us[2].get_tensor())
                    + self.bs[2].get_tensor()
            )
    }

    pub fn encode(&self, initial_state: Tensor, x: &Vec<Tensor>) -> Vec<Tensor> {
        let batch_size = x[0].shape()[0];
        assert_eq!(initial_state.shape(), &[batch_size, self.state_size]);
        for x in x {
            assert_eq!(x.shape(), &[batch_size, self.input_size]);
        }
        let mut state = initial_state.clone();
        let mut outputs = vec![];
        for x in x {
            state = self.step(x.clone(), state);
            outputs.push(state.clone());
        }
        outputs
    }

    pub fn decode(
        &self,
        mut state: Tensor,
        output_fn: impl Fn(Tensor) -> Tensor,
        output_to_input_fn: impl Fn(Tensor) -> Tensor,
        len: usize,
    ) -> Vec<Tensor> {
        let mut outputs = vec![];
        for _ in 0..len {
            let output = output_fn(state.clone());
            outputs.push(output.clone());
            let input = output_to_input_fn(output);
            state = self.step(input, state);
        }
        outputs
    }

    pub fn all_params(&self) -> Vec<Param> {
        self.ws
            .iter()
            .chain(self.us.iter())
            .chain(self.bs.iter())
            .cloned()
            .collect()
    }
}

#[allow(dead_code)]
fn graph(vars: &[Tensor], name: impl ToString) {
    let f = std::fs::File::create(name.to_string() + ".dot").unwrap();
    let mut w = std::io::BufWriter::new(f);
    tensorflake::export_dot::write_dot(&mut w, vars, &mut |v| {
        // format!("{} {}", v.get_name(), (*v).to_string())
        // v.get_name().to_string()
        format!("{} {:?}", v.get_name(), v.shape())
    })
    .unwrap();
}
