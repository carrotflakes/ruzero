use ndarray::Array;
use ndarray_rand::{rand::SeedableRng, rand_distr::Uniform, RandomExt};
use ruzero::{
    call,
    functions::{Add, BroadcastTo, Div, Matmul, Mul, Pow, Sub, SumTo},
    scalar, Function, Variable, ENABLE_BACKPROP,
};

fn main() {
    let n = 10;
    let mut rng = rand_isaac::Isaac64Rng::seed_from_u64(42);

    // make dataset
    let x = Variable::<ENABLE_BACKPROP>::new(
        Array::random_using((n, 1, 1), Uniform::new(0., 1.), &mut rng).into_dyn(),
    );
    let y = call!(
        Add,
        call!(Mul, x, Variable::new(scalar(2.0))),
        Variable::new(Array::zeros((n, 1, 1)).into_dyn())
    );

    // dbg!(&*x);
    // dbg!(&*y);

    let mut w = Variable::new(ndarray::array![[[0.0]]].into_dyn()).named("w");
    let mut b = Variable::new(ndarray::array![0.0].into_dyn()).named("b");

    let predict = |w: Variable<ENABLE_BACKPROP>,
                   b: Variable<ENABLE_BACKPROP>,
                   x: Variable<ENABLE_BACKPROP>| {
        call!(
            Add,
            call!(Matmul, x, call!(BroadcastTo::new(vec![n, 1, 1]), w)),
            b
        )
    };

    for i in 0..100 {
        let y_ = predict(w.clone(), b.clone(), x.clone());
        // dbg!(&*y_);

        let loss = mean_squared_error(y.clone(), y_.clone());
        println!("loss: {}", loss[[]]);

        loss.backward(false, false);

        if i == 0 {
            graph(&[loss]);
        }

        let gw = w.get_grad::<ENABLE_BACKPROP>().unwrap();
        let gb = b.get_grad::<ENABLE_BACKPROP>().unwrap();
        // dbg!(&*gw);
        // dbg!(&*gb);

        let lr = 0.01;
        w = Variable::new(&*w - &*gw * lr);
        b = Variable::new(&*b - &*gb * lr);
    }
}

fn mean_squared_error<const EB: bool>(x0: Variable<EB>, x1: Variable<EB>) -> Variable<EB> {
    let x = call!(Pow::new(2.0), call!(Sub, x0, x1));
    call!(
        Div,
        call!(SumTo::new((0..x.ndim()).collect()), x),
        Variable::new(scalar(x.shape().iter().product::<usize>() as f32))
    )
}

fn graph(vars: &[Variable<ENABLE_BACKPROP>]) {
    let f = std::fs::File::create("graph.dot").unwrap();
    let mut w = std::io::BufWriter::new(f);
    ruzero::export_dot::write_dot(&mut w, vars, &mut |v| {
        // format!("{} {}", v.get_name(), (*v).to_string())
        // v.get_name().to_string()
        format!("{} {:?}", v.get_name(), v.shape())
    })
    .unwrap();
}
