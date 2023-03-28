struct WithCurrying {
    a: i32,
    b: i32,
    c: i32,
}
impl WithCurrying {
    #[auto_curry::curry]
    fn new(a: i32, b: i32, c: i32) -> Self {
        Self { a, b, c }
    }

    #[auto_curry::curry]
    fn add_with(self, d: i32, e: i32) -> i32 {
        self.a + self.b + self.c + d + e
    }
}

fn main() {
    let with_currying = WithCurrying::new(2)(4)(6);
    assert_eq!(with_currying.add_with(8)(10), 30);
}
