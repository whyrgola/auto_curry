#[auto_curry::curry]
fn add(mut a: i32, b: i32) {
    a = a + b;
}

fn main() {
    add(2)(2);
}
