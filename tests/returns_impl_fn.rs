//#[auto_curry::curry]
//fn add_curried(a: i32, b: i32) -> impl Fn(i32) -> i32 {
//    move || a + b
//}
//#[auto_curry::curry]
//fn add_curried<X: Iterator<Item = i32>>(a: i32, b: i32) -> X {
//    std::iter::from_fn(|| Some(a))
//}
