#[auto_curry::curry]
fn add(r#mut: i32, b: i32) -> i32 {
    r#mut + b
}

fn main() {
    assert_eq!(add(1)(2), 3)
}
