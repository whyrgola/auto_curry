#[auto_curry::curry]
fn add(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    a + b + c + d + e
}

#[auto_curry::curry]
fn mutable_add(a: i32, b: i32, mut c: i32, d: i32, e: i32) -> i32 {
    a + b + c + d + e
}

fn main() {
    assert_eq!(add(1)(1)(1)(1)(2), 6);
    assert_eq!(mutable_add(1)(1)(1)(1)(3), 7);
}
