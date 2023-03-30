// Waiting for this:
//#![feature(impl_trait_in_fn_trait_return)]
// or alternatively as a workaound:
//#![feature(type_alias_impl_trait)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
//use syn::visit::{self, Visit};
use syn::ReturnType;
use syn::Type;
use syn::{parse_macro_input, FnArg, ItemFn, Signature};

/// Automatically curries an `fn` function when used as an attribute,
/// supports generics and lifetimes
#[proc_macro_attribute]
pub fn curry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as ItemFn);
    match generate_curry(parsed) {
        Ok(gen) => gen,
        Err(err) => panic!("{err}"),
    }
    .into()
}

/// Generates the curried function for `curry`. Used internally for testing.
fn generate_curry(
    ItemFn {
        attrs,
        vis: visibility,
        sig:
            Signature {
                generics,
                output,
                ident: fn_name,
                inputs,
                ..
            },
        block,
    }: ItemFn,
) -> Result<proc_macro2::TokenStream, &'static str> {
    let mut arguments = inputs.into_iter();

    if let ReturnType::Type(_, ref return_type) = output {
        if matches!(**return_type, Type::ImplTrait(_)) {
            return Err(
                "Cannot have an `impl Trait` return type in a curried function \
                        try a `Box<dyn Trait>` as an alternative",
            );
        }
    }

    // TODO: Check if it's possible to do this without cloning
    let has_mutability = arguments.clone().any(|argument| {
        matches!(argument, FnArg::Typed(typed_argument) if {
            typed_argument.to_token_stream().into_iter().any(
                |token| matches!(token, TokenTree::Ident(ident) if &ident.to_string() == "mut"),
            )
        })
    });

    let fn_trait = match has_mutability {
        true => quote!(FnMut),
        false => quote!(Fn),
    };

    // Take care of the self receiver and the first argument
    let (receiver, first_argument) = match arguments.next().ok_or(MUST_HAVE_NON_SELF_ARGUMENT)? {
        FnArg::Receiver(receiver) => (
            Some(receiver),
            match arguments.next().ok_or(MUST_HAVE_NON_SELF_ARGUMENT)? {
                FnArg::Typed(typed_argument) => typed_argument,
                FnArg::Receiver(_) => unreachable!("{ONLY_ONE_SELF_RECEIVER}"),
            },
        ),
        FnArg::Typed(typed_argument) => (None, typed_argument),
    };

    let fn_arguments = match receiver {
        Some(receiver) => quote!((#receiver, #first_argument)),
        None => quote!((#first_argument)),
    };

    let mut arguments = arguments
        .map(|argument| match argument {
            FnArg::Typed(typed_argument) => typed_argument,
            FnArg::Receiver(_) => unreachable!("{ONLY_ONE_SELF_RECEIVER}"),
        })
        .map(|argument| {
            let (argument_name, argument_type) = (argument.pat, argument.ty);

            (
                quote!(move |#argument_name|),
                quote!(#fn_trait (#argument_type)),
            )
        });

    let (first_closure_args, first_type) = arguments
        .next()
        .ok_or("Cannot curry a function with only 1 argument")?;

    // Any arguments after the first and second arguments
    // if there are any, will be caught here and reduced down to
    // a single quote
    //
    // The problem here is reducing a bunch of:
    // `move |name|` and `Fn(SomeType)`
    //
    // into single:
    // `Box::new(move |name|)` and `Box<Fn(SomeType)>`
    //
    // A convenient yet perhaps dangerous way to do it is via
    // a recursive function:
    let final_arguments = {
        // TODO: Try replacing this with a `reduce` on the Iterator.
        fn recursively_box(
            mut iterator: impl Iterator<Item = (TokenStream2, TokenStream2)>,
            streams: Option<(TokenStream2, TokenStream2)>,
            // TODO: Convert into `FnOnce`
            get_final_additions: &impl Fn() -> (TokenStream2, TokenStream2),
        ) -> Option<(TokenStream2, TokenStream2)> {
            let (left_closure_args, left_closure_type) = streams?;

            let next = iterator.next();
            match recursively_box(iterator, next, get_final_additions) {
                // Still going
                Some((right_closure_args, right_closure_types)) => Some((
                    quote!(Box::new(#left_closure_args #right_closure_args)),
                    quote!(Box<#left_closure_type -> #right_closure_types>),
                )),
                // Finally over
                None => {
                    let (block, output) = get_final_additions();
                    Some((
                        quote!(Box::new(#left_closure_args #block)),
                        quote!(Box<#left_closure_type #output>),
                    ))
                }
            }
        }

        //Fn(SomeType), Fn(OtherType), Output
        //Fn(SomeType) -> Fn(OtherType) -> Output
        //
        // then iter: when you see an `Fn` (or `FnMut`),
        // put a `Box<` up front and an `>` on the back,
        // which might be done like this:
        //
        // | quote!(#left Box<#right>)
        //
        // repeatedly until all `Fn` or `FnMut` items are gon.
        //
        //Box<Fn(SomeType) -> Box<Fn(OtherType) -> Output>>

        let mut dyn_arguments = arguments
            .map(|(closure_args, argument_type)| (closure_args, quote!(dyn #argument_type)));

        //let boxed_arguments = dyn_arguments
        //    .clone()
        //    .fold("", |acc, (closure_args, argument_type)| format!("{acc} "));

        let first_elem = dyn_arguments.next();
        let get_final_additions = || (quote!(#block), quote!(#output));

        recursively_box(dyn_arguments, first_elem, &get_final_additions)
    };

    let (final_closure, final_type) = match final_arguments {
        Some((joined_closure_args, joined_type_of_arguments)) => (
            quote!(#joined_closure_args),
            quote!(-> #joined_type_of_arguments),
        ),
        // (-> is there for us in #output)
        None => (quote!(#block), quote!(#output)),
    };

    Ok(quote! {
        // TODO: Do we need to include the `attrs`? (assumption: yes)
        #(#attrs)*
        #visibility fn #fn_name #generics #fn_arguments -> impl #first_type #final_type {
            #first_closure_args #final_closure
        }
    })
}

const MUST_HAVE_NON_SELF_ARGUMENT: &str = "Must have atleast two non `self` argument to curry";
const ONLY_ONE_SELF_RECEIVER: &str = "Cannot have two or more `self` receivers";

/*
struct FnBoxingVisitor;

// TODO: Visit the type for Fn's and the code for fns
impl<'ast> Visit<'ast> for FnBoxingVisitor {
    fn typ
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn test_curry(input: &str, output: &str) {
        let parsed: ItemFn = syn::parse_str(input).unwrap();
        assert_eq!(generate_curry(parsed).unwrap().to_string(), output)
    }

    #[test]
    fn long_add() {
        test_curry(
            "
                pub fn add(self, a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
                    a + b + c + d + e
                }
            ",
            "pub fn add (self , a : i32) -> impl Fn (i32) -> Box < dyn Fn (i32) -> Box < dyn Fn (i32) -> Box < dyn Fn (i32) -> i32 > > > { move | b | Box :: new (move | c | Box :: new (move | d | Box :: new (move | e | { a + b + c + d + e }))) }"
        )
    }

    #[test]
    fn with_generics() {
        test_curry(
            r#"
                fn generic<T>(x: T, y: T, z: T) {
                    println!("{x}");
                    println!("{y}");
                    println!("{z}");
                }
            "#,
            "fn generic < T > (x : T) -> impl Fn (T) -> Box < dyn Fn (T) > { move | y | Box :: new (move | z | { println ! (\"{x}\") ; println ! (\"{y}\") ; println ! (\"{z}\") ; }) }"
        )
    }

    #[test]
    fn with_fake_mut() {
        test_curry(
            r#"
                fn fake_mut(r#mut: i8, b: i32) {
                    println!("{} {}", r#mut, b);
                }
            "#,
            "fn fake_mut (r#mut : i8) -> impl Fn (i32) { move | b | { println ! (\"{} {}\" , r#mut , b) ; } }",
        )
    }

    //#[test]
    //fn with_gats() {
    //    todo!()
    //}
}
