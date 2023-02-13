use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, AttributeArgs, FnArg, Ident, ItemFn, Pat, Path, Type,
};

extern crate proc_macro;

/// Extracts the `PatType` of a `FnArg`.
fn extract_arg_pat(a: FnArg) -> Option<Pat> {
    match a {
        FnArg::Typed(p) => Some(*p.pat),
        _ => None,
    }
}

/// Retrieves the argument identifiers of a function.
fn extract_arg_idents(fn_args: Punctuated<FnArg, syn::token::Comma>) -> Vec<Pat> {
    fn_args
        .into_iter()
        .filter_map(extract_arg_pat)
        .collect::<Vec<_>>()
}

fn extract_arg_idents_vec(fn_args: Vec<FnArg>) -> Vec<Pat> {
    fn_args
        .into_iter()
        .filter_map(extract_arg_pat)
        .collect::<Vec<_>>()
}

#[proc_macro_attribute]
pub fn pluginop(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let po = match &attrs[0] {
        syn::NestedMeta::Meta(m) => match m {
            syn::Meta::Path(po) => po,
            syn::Meta::List(_) => todo!("meta list"),
            syn::Meta::NameValue(_) => todo!("meta nv"),
        },
        syn::NestedMeta::Lit(_) => todo!("lit"),
    };
    let base_fn = parse_macro_input!(item as ItemFn);
    let fn_args = extract_arg_idents(base_fn.sig.inputs.clone());

    let fn_output = &base_fn.sig.output;
    let fn_inputs = &base_fn.sig.inputs;
    let mut fn_inputs_no_self = fn_inputs.clone();
    fn_inputs_no_self.pop();
    let fn_vis = &base_fn.vis;
    let fn_name = &base_fn.sig.ident;
    let fn_block = &base_fn.block;
    let fn_name_internal = format_ident!("__{}__", fn_name);

    let ret_block = match &base_fn.sig.output {
        syn::ReturnType::Default => quote!({}),
        syn::ReturnType::Type(_, t) => {
            if let Type::Tuple(tu) = *t.clone() {
                let elems = tu.elems.into_iter();
                quote! {
                    let mut it = _res.iter_mut();
                    (
                        #(
                            #elems :: try_from(it.next().unwrap()).unwrap(),
                        )*
                    )
                }
            } else {
                quote!({ _res.iter_mut().unwrap().try_into().unwrap() })
            }
        }
    };

    quote! {
        fn #fn_name_internal(#fn_inputs) #fn_output {
            #fn_block
        }

        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            use pluginop::api::ConnectionToPlugin;
            let ph = self.get_pluginizable_connection().get_ph();
            if ph.provides(& #po, pluginop::common::Anchor::Replace) {
                let _res = ph.call(
                    & #po,
                    &[
                        #(#fn_args.into() ,)*
                    ],
                ).expect("call failed");

                #ret_block
            } else {
                self.#fn_name_internal(#(#fn_args,)*)
            }
        }
    }
    .into()
}

/// Arguments that can be passed through the `protoop` macro. See the
/// documentation of the macro `protoop` for more details.
#[derive(Debug, FromMeta)]
struct MacroArgs {
    po: Path,
    param: Ident,
}

#[proc_macro_attribute]
pub fn pluginop_param(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let attrs_args = match MacroArgs::from_list(&attrs) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let po = attrs_args.po;
    let param = attrs_args.param;

    let base_fn = parse_macro_input!(item as ItemFn);

    let fn_output = &base_fn.sig.output;
    let fn_inputs = &base_fn.sig.inputs;
    let fn_inputs_no_param: Vec<FnArg> = fn_inputs
        .clone()
        .into_iter()
        .filter(|e| {
            if let FnArg::Typed(pt) = e {
                if let Pat::Ident(pi) = &*pt.pat {
                    pi.ident != param
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect();
    let fn_inputs_no_self = fn_inputs_no_param.clone();
    let fn_args = extract_arg_idents_vec(fn_inputs_no_self);
    let fn_vis = &base_fn.vis;
    let fn_name = &base_fn.sig.ident;
    let fn_block = &base_fn.block;
    let fn_name_internal = format_ident!("__{}__", fn_name);

    let ret_block = match &base_fn.sig.output {
        syn::ReturnType::Default => quote!({}),
        syn::ReturnType::Type(_, t) => {
            if let Type::Tuple(tu) = *t.clone() {
                let elems = tu.elems.into_iter();
                quote! {
                    let mut it = _res.iter_mut();
                    (
                        #(
                            #elems :: try_from(it.next().unwrap()).unwrap(),
                        )*
                    )
                }
            } else {
                quote!({ _res.iter_mut().unwrap().try_into().unwrap() })
            }
        }
    };

    quote! {
        fn #fn_name_internal(#(#fn_inputs_no_param,)*) #fn_output {
            #fn_block
        }

        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            use pluginop::api::ConnectionToPlugin;
            let ph = self.get_pluginizable_connection().get_ph();
            if ph.provides(& #po(#param), pluginop::common::Anchor::Replace) {
                let _res = ph.call(
                    & #po(#param),
                    &[
                        #(#fn_args.into() ,)*
                    ],
                ).expect("call failed");

                #ret_block
            } else {
                self.#fn_name_internal(#(#fn_args,)*)
            }
        }
    }
    .into()
}
