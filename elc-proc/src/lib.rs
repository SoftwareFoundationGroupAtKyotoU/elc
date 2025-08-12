use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Expr, ExprBlock, Ident, ItemFn, Pat, Result, ReturnType, Signature, Stmt, Token, parse_quote,
    parse2,
};

/// Macro argument of [`elc`].
enum ElcArg {
    /// Argument `pure`.
    Pure,
    /// Argument `ret = ...pattern...`.
    Ret(Pat),
}

impl Parse for ElcArg {
    /// Parse [`ParseStream`] to [`ElcArg`].
    fn parse(input: ParseStream) -> Result<ElcArg> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "pure" => Ok(ElcArg::Pure),
            "ret" => {
                input.parse::<Token![=]>()?;
                Ok(ElcArg::Ret(Pat::parse_single(input)?))
            }
            _ => Err(input.error(format!("Unknown identifier: {}", &ident))),
        }
    }
}

/// Macro arguments of [`elc`].
struct ElcArgs {
    args: Punctuated<ElcArg, Token![,]>,
}

impl Parse for ElcArgs {
    /// Parse [`ParseStream`] to [`ElcArgs`].
    fn parse(input: ParseStream) -> Result<ElcArgs> {
        input
            .parse_terminated(ElcArg::parse, Token![,])
            .map(|args| ElcArgs { args })
    }
}

/// Process an argument [`ElcArg`] for [`elc2`].
fn process_arg(pre_stmts: &mut Vec<Stmt>, ret: &mut Pat, arg: ElcArg) {
    match arg {
        ElcArg::Pure => pre_stmts.push(parse_quote! {
            fn __ELC_pure() {}
        }),
        ElcArg::Ret(ident) => *ret = ident,
    }
}

/// Update a statement [`Stmt`] for [`elc2`].
fn update_stmt(sig: &Signature, ret: &Pat, stmt: &mut Stmt) {
    match stmt {
        Stmt::Expr(
            Expr::Block(ExprBlock {
                attrs,
                label: Some(label),
                block,
            }),
            _,
        ) => match label.name.ident.to_string().as_str() {
            "requires" => {
                let inputs = sig.inputs.clone();
                *stmt = parse_quote! {
                    #(#attrs)*
                    #[allow(unused_variables)]
                    fn __ELC_requires(#inputs) -> bool #block
                };
            }
            "ensures" => {
                let mut inputs = sig.inputs.clone();
                match &sig.output {
                    ReturnType::Default => {}
                    ReturnType::Type(_, ty) => {
                        inputs.push_punct(parse_quote! { , });
                        let ty = ty.clone();
                        inputs.push_value(parse_quote! { #ret : #ty });
                    }
                }
                *stmt = parse_quote! {
                    #(#attrs)*
                    #[allow(unused_variables)]
                    fn __ELC_ensures(#inputs) -> bool #block
                };
            }
            _ => {}
        },
        _ => {}
    }
}

/// [`proc_macro2`] version of [`elc`].
fn elc2(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let ElcArgs { args } = parse2(attr)
        .unwrap_or_else(|err| panic!("Error parsing the attribute of the elc macro: {}", err));
    let ItemFn {
        attrs,
        vis,
        sig,
        mut block,
    } = parse2::<ItemFn>(item)
        .unwrap_or_else(|err| panic!("Error parsing the body of the elc macro: {}", err));
    let mut pre_stmts: Vec<Stmt> = vec![];
    let mut ret: Pat = parse_quote! { ret };
    args.into_iter()
        .for_each(|arg| process_arg(&mut pre_stmts, &mut ret, arg));
    let mut stmts = block.stmts;
    stmts
        .iter_mut()
        .for_each(|stmt| update_stmt(&sig, &ret, stmt));
    block.stmts = pre_stmts;
    block.stmts.append(&mut stmts);
    quote! {
        #(#attrs)* #vis #sig #block
    }
}

/// Procedural macro for Elc.
///
/// It is intended to be used as an attribute macro.
///
/// # Examples
///
/// A function under [`elc`]:
/// ```ignore
/// #[elc]
/// fn foo(n: i32) -> i32 {
///     'requires: {
///         n > 2
///     }
///     'ensures: {
///         ret > 8
///     }
///     n * n
/// }
/// ```
///
/// A function under [`elc`] with arguments:
/// ```ignore
/// #[elc(pure, ret = (ret0, ret1))]
/// fn foo(n: i32) -> (i32, i32) {
///     'ensures: {
///         ret0 == ret1
///     }
///     (n, n)
/// }
/// ```
///
/// # Syntax
///
/// In the following, we use the following meta-variables:
/// - `fun_params`: the parameters of the function
/// - `FunRetTy`: the return type of the function
///
/// ## Precondition (in a function)
///
/// A top-level block labeled `'requires` in a function
/// ```ignore
/// 'requires: {
///     ...body...
/// }
/// ```
/// is treated specially as the precondition of the function.
///
/// The body should return the type `bool`.
/// The block can refer to the function arguments.
///
/// Internally, it expands to an inner function:
/// ```ignore
/// fn __ELC_requires(fun_params...) -> bool { ...body... }
/// ```
///
/// ## Postcondition (in a function)
///
/// A top-level block labeled `'ensures` in a function
/// ```ignore
/// 'ensures: {
///     ...body...
/// }
/// ```
/// is treated specially as the postcondition of the function.
///
/// Internally, it expands to an inner function:
/// ```ignore
/// fn __ELC_ensures(fun_params..., ret: FunRetTy) -> bool { ...body... }
/// ```
///
/// The parameter `ret` binds the return value of the function.
/// This can be customized to an arbitrary pattern by passing
/// `ret = ...pattern...` as an argument of `elc`.
///
/// ## Purity (of a function)
///
/// A function can be marked pure by passing `pure` as an argument of `elc`.
#[proc_macro_attribute]
pub fn elc(attr: TokenStream, item: TokenStream) -> TokenStream {
    elc2(attr.into(), item.into()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elc() {
        assert_eq!(
            elc2(
                quote! {},
                quote! {
                    fn foo(n: i32) -> i32 {
                        'requires: {
                            n > 2
                        }
                        'ensures: {
                            ret > 8
                        }
                        n * n
                    }
                }
            )
            .to_string(),
            quote! {
                fn foo(n: i32) -> i32 {
                    #[allow(unused_variables)]
                    fn __ELC_requires(n: i32) -> bool {
                        n > 2
                    }
                    #[allow(unused_variables)]
                    fn __ELC_ensures(n: i32, ret: i32) -> bool {
                        ret > 8
                    }
                    n * n
                }
            }
            .to_string()
        );
    }

    #[test]
    fn test_elc_params() {
        assert_eq!(
            elc2(
                quote! {
                    // Purity
                    pure,
                    // Custom return pattern
                    ret = (ret0, ret1)
                },
                quote! {
                    fn foo(n: i32) -> (i32, i32) {
                        'ensures: {
                            ret0 == ret1
                        }
                        (n, n)
                    }
                }
            )
            .to_string(),
            quote! {
                fn foo(n: i32) -> (i32, i32) {
                    fn __ELC_pure() {}
                    #[allow(unused_variables)]
                    fn __ELC_ensures(n: i32, (ret0, ret1): (i32, i32)) -> bool {
                        ret0 == ret1
                    }
                    (n, n)
                }
            }
            .to_string()
        )
    }

    #[test]
    fn test_elc_no_ret() {
        assert_eq!(
            elc2(
                quote! {},
                quote! {
                    // No return type
                    fn foo(_n: i32) {
                        'requires: {
                            _n > 0
                        }
                        'ensures: {
                            _n >= 0
                        }
                    }
                }
            )
            .to_string(),
            quote! {
                fn foo(_n: i32) {
                    #[allow(unused_variables)]
                    fn __ELC_requires(_n: i32) -> bool {
                        _n > 0
                    }
                    #[allow(unused_variables)]
                    fn __ELC_ensures(_n: i32) -> bool {
                        _n >= 0
                    }
                }
            }
            .to_string()
        )
    }
}
