use ruzero::{
    functions::{Double, Exp, Square},
    Function, Variable,
};

fn main() {
    let x = Variable::new(2.0);
    let y = Double.call(vec![x]);
    println!("{}", *y[0]);

    // let d = numerical_diff(&Square, &x);
    // println!("{}", *d);

    {
        let x = Variable::new(0.5);
        let a = Square.call(vec![x.clone()]);
        let b = Exp.call(a);
        let y = Square.call(b);
        println!("{}", *y[0]);
        y[0].set_grad(Variable::new(1.0));
        y[0].backward();
        println!("{}", *x.get_grad().unwrap()); // 3.29
        // y.set_grad(1.0);
        // b.set_grad(*Square.backward(&b, &Variable::new(y.get_grad().unwrap())));
        // a.set_grad(*Exp.backward(&a, &Variable::new(b.get_grad().unwrap())));
        // x.set_grad(*Square.backward(&x, &Variable::new(a.get_grad().unwrap())));
        // println!("{:?}", x.get_grad());
    }
}
