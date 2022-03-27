mod arith;

use ndarray::prelude::*;
use ndarray_rand::{
    rand::{prelude::SliceRandom, SeedableRng},
    rand_distr::Uniform,
    RandomExt,
};
use tensorflake::{functions::*, losses::SoftmaxCrossEntropy, ndarray_util::onehot, nn::*, *};

fn main() {
    let mut data = arith::make(10000, 42, 15);
    // let data = (0..10000)
    //     .map(|i| format!("=({}{}{})", i % 6 + 1, i % 6 + 1, i % 6 + 1))
    //     .collect::<Vec<_>>();
    let mut rng = rand_isaac::Isaac64Rng::seed_from_u64(42);

    let model = Model::new(arith::VOCAB_SIZE);

    let start = std::time::Instant::now();

    {
        let str = "1+2=3";
        let y = model.call(arith::encode(&str[..str.len() - 1]), true);
        let y = Concat::new(0).call(y).pop().unwrap();
        let loss = call!(SoftmaxCrossEntropy::new(arith::encode(&str[1..])), y);
        graph(&[loss], "rnn");
    }

    let mut gradients = GradientsAccumulator::default();
    for e in 0..100 {
        data.shuffle(&mut rng);
        for i in 0..data.len() {
            let str = &data[i];
            let eqp = str.chars().position(|c| c == '=').unwrap();
            let y = model.call(arith::encode(&str[..str.len() - 1]), true);
            let yy = Concat::new(0)
                .call(y.iter().skip(eqp).cloned().collect())
                .pop()
                .unwrap();
            let loss = call!(SoftmaxCrossEntropy::new(arith::encode(&str[eqp + 1..])), yy);
            gradients.compute(&loss);
            if i % 10 == 0 {
                gradients.optimize(0.01 - e as f32 * 0.0001);
            }
            if i % 5000 == 0 {
                // println!("{:?}", &*y[1]);
                let y = Concat::new(0).call(y).pop().unwrap();
                let v = y
                    .map_axis(Axis(1), |x| {
                        x.iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .unwrap()
                            .0
                    })
                    .into_raw_vec();
                // println!("{:?}", v);
                println!("{}", str);
                println!(" {}", arith::decode(&v));
                dbg!(loss[[]]);
            }
        }
        // for (p, t) in gradients.table.iter() {
        //     dbg!(&**t);
        // }
        println!("epoch: {}", e);
    }

    println!("time: {:?}", start.elapsed());
}

fn count_correction(y: &Tensor, t: &[usize]) -> usize {
    t.iter()
        .enumerate()
        .filter(|(i, t)| {
            let y = y
                .slice(s![*i, ..])
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;
            y == **t
        })
        .count()
}

pub struct Model {
    pub vocab_size: usize,
    pub initial: Param,
    pub enb: Linear,
    pub linear: Linear,
    pub output: Linear,
}

impl Model {
    pub fn new(vocab_size: usize) -> Self {
        let rng = rand_isaac::Isaac64Rng::seed_from_u64(42);
        let param_gen = {
            let rng = rng.clone();
            move || {
                let mut rng = rng.clone();
                move |shape: &[usize]| -> Param {
                    let t =
                        Array::random_using(shape, Uniform::new(0., 0.01), &mut rng).into_ndarray();
                    SGDOptimizee::new(t)
                    // AdamOptimizee::new(t)
                }
            }
        };
        let state_size = 200;
        Self {
            vocab_size,
            initial: param_gen()(&[1, state_size]),
            enb: Linear::new(vocab_size, state_size, &mut param_gen(), &mut param_gen()),
            linear: Linear::new(state_size, state_size, &mut param_gen(), &mut param_gen()),
            output: Linear::new(state_size, vocab_size, &mut param_gen(), &mut param_gen()),
        }
    }

    pub fn call(&self, x: Vec<usize>, train: bool) -> Vec<Tensor> {
        let mut state = self.initial.get_tensor();
        let mut outputs = vec![];
        for x in x {
            let enb = self
                .enb
                .call(onehot(&ndarray::arr1(&[x]), self.vocab_size).into(), train);
            // let concated = call!(Concat::new(1), enb, state);
            let concated = &enb + &state;
            state = self.linear.call(concated, train);
            state = call!(Tanh, state);
            outputs.push(self.output.call(state.clone(), train).named("output"));
        }
        outputs
    }

    pub fn all_params(&self) -> Vec<Param> {
        self.enb
            .all_params()
            .into_iter()
            .chain(self.linear.all_params())
            .collect()
    }
}

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