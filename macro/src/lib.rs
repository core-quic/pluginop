use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, AttributeArgs, FnArg, GenericArgument, Ident,
    ItemFn, Pat, PatType, Path, ReturnType, Type,
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

// First boolean returns whether the type is `Octets` or `OctetsMut`. Second returns whether the
// type is exactly `OctetsMut`, The third indicates whether the reference is mutable or not.
fn has_octets(pt: &PatType) -> (bool, bool, bool) {
    match &*pt.ty {
        Type::Reference(tref) => match &*tref.elem {
            Type::Path(p) => {
                if p.path
                    .segments
                    .iter()
                    .any(|ps| &ps.ident.to_string() == "Octets")
                {
                    (true, false, tref.mutability.is_some())
                } else if p
                    .path
                    .segments
                    .iter()
                    .any(|ps| &ps.ident.to_string() == "OctetsMut")
                {
                    (true, true, tref.mutability.is_some())
                } else {
                    (false, false, false)
                }
            }
            _ => (false, false, false),
        },
        _ => (false, false, false),
    }
}

fn is_result_unit(ty: &syn::Type) -> bool {
    match ty {
        Type::Path(tp) => {
            if let Some(ps) = tp
                .path
                .segments
                .iter()
                .find(|s| &s.ident.to_string() == "Result")
            {
                if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                    if ab.args.len() != 1 {
                        return false;
                    }
                    if let Some(GenericArgument::Type(syn::Type::Tuple(tu))) = ab.args.first() {
                        return tu.elems.is_empty();
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn get_param_block(
    args: &Punctuated<FnArg, syn::token::Comma>,
    ignore: Option<Ident>,
) -> proc_macro2::TokenStream {
    let args_code: Vec<proc_macro2::TokenStream> = args
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(pt) => {
                let pat = &pt.pat;
                match has_octets(pt) {
                    (true, false, true) => Some(quote!( OctetsPtr::from(#pat).into_with_ph(ph) )),
                    (true, true, true) => Some(quote!( OctetsMutPtr::from(#pat).into_with_ph(ph) )),
                    (true, _, false) => panic!("Octets argument must be mutable"),
                    _ => {
                        if let Some(ign) = &ignore {
                            if let Pat::Ident(pi) = &*pt.pat {
                                if pi.ident == *ign {
                                    return None;
                                }
                            }
                        }

                        Some(quote!( #pat.clone().into_with_ph(ph) ))
                    }
                }
            }
            _ => None,
        })
        .collect();
    quote!(
        &[
            #(#args_code ,)*
        ]
    )
}

fn get_ret_block(fn_output_type: &ReturnType) -> proc_macro2::TokenStream {
    match fn_output_type {
        syn::ReturnType::Default => quote!({
            if let Err(err) = res {
                panic!("plugin execution error: {:?}", err);
            }
        }),
        syn::ReturnType::Type(_, t) => {
            if let Type::Tuple(tu) = *t.clone() {
                let elems = tu.elems.into_iter();
                quote! {
                    let mut it = match res {
                        Ok(r) => r.into_iter(),
                        Err(pluginop::Error::OperationError(e)) => todo!("operation error {:?}; should you use pluginop_result?", e),
                        Err(err) => panic!("plugin execution error: {:?}", err),
                    };
                    (
                        #(
                            #elems :: try_from(it.next().unwrap()).unwrap(),
                        )*
                    )
                }
            } else {
                quote!(
                    let mut it = match res {
                        Ok(r) => r.into_iter(),
                        Err(pluginop::Error::OperationError(e)) => todo!("operation error {:?}; should you use pluginop_result?", e),
                        Err(err) => panic!("plugin execution error: {:?}", err),
                    };
                    { it.next().unwrap().try_into().unwrap() }
                )
            }
        }
    }
}

fn get_ret_result_block(fn_output_type: &ReturnType) -> proc_macro2::TokenStream {
    match fn_output_type {
        syn::ReturnType::Default => quote!({
            if let Err(err) = res {
                panic!("plugin execution error: {:?}", err);
            }
        }),
        syn::ReturnType::Type(_, t) => {
            if let Type::Tuple(tu) = *t.clone() {
                let elems = tu.elems.into_iter();
                quote! {
                    let mut it = match res {
                        Ok(r) => r.into_iter(),
                        Err(pluginop::Error::OperationError(e)) => return Err(e.into()),
                        Err(err) => panic!("plugin execution error: {:?}", err),
                    };
                    Ok((
                        #(
                            #elems :: try_from_with_ph(it.next().unwrap(), ph).unwrap(),
                        )*
                    ))
                }
            } else {
                // We need to check if this is the unit type.
                if is_result_unit(t) {
                    quote!(match res {
                        Ok(r) => Ok(()),
                        Err(pluginop::Error::OperationError(e)) => Err(e.into()),
                        Err(err) => panic!("plugin execution error: {:?}", err),
                    })
                } else {
                    quote!(
                        let mut it = match res {
                            Ok(r) => r.into_iter(),
                            Err(pluginop::Error::OperationError(e)) => return Err(e.into()),
                            Err(err) => panic!("plugin execution error: {:?}", err),
                        };
                        match it.next() {
                            Some(r) => Ok(r.try_into_with_ph(ph).unwrap()),
                            None => panic!("Missing output from the plugin"),
                        }
                    )
                }
            }
        }
    }
}

fn get_out_block(
    base_fn: &ItemFn,
    po: &Path,
    ret_block: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fn_args = extract_arg_idents(base_fn.sig.inputs.clone());
    let fn_inputs = &base_fn.sig.inputs;
    let mut fn_inputs_no_self = fn_inputs.clone();
    fn_inputs_no_self.pop();
    let fn_vis = &base_fn.vis;
    let fn_name = &base_fn.sig.ident;
    let fn_block = &base_fn.block;
    let fn_output = &base_fn.sig.output;
    let fn_name_internal = format_ident!("__{}__", fn_name);
    let param_code = get_param_block(fn_inputs, None);

    quote! {
        fn #fn_name_internal(#fn_inputs) #fn_output {
            #fn_block
        }

        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            use pluginop::api::ToPluginizableConnection;
            use pluginop::Error;
            use pluginop::IntoWithPH;
            use pluginop::TryIntoWithPH;
            use pluginop::octets::OctetsMutPtr;
            use pluginop::octets::OctetsPtr;
            let ph = self.get_pluginizable_connection().map(|pc| pc.get_ph_mut());
            if let Some(ph) = ph.filter(|ph| ph.provides(& #po, pluginop::common::Anchor::Replace)) {
                let params = #param_code;
                let res = ph.call(
                    & #po,
                    params,
                );
                ph.clear_bytes_content();

                #ret_block
            } else {
                self.#fn_name_internal(#(#fn_args,)*)
            }
        }
    }
}

fn get_out_param_block(
    param: Ident,
    base_fn: &ItemFn,
    po: &Path,
    ret_block: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fn_output = &base_fn.sig.output;
    let fn_inputs = &base_fn.sig.inputs;
    // let fn_inputs_no_param: Vec<FnArg> = fn_inputs
    //     .clone()
    //     .into_iter()
    //     .filter(|e| {
    //         if let FnArg::Typed(pt) = e {
    //             if let Pat::Ident(pi) = &*pt.pat {
    //                 pi.ident != param
    //             } else {
    //                 true
    //             }
    //         } else {
    //             true
    //         }
    //     })
    //     .collect();
    let fn_inputs_iter: Vec<FnArg> = fn_inputs.clone().into_iter().collect();
    let fn_inputs_no_self = fn_inputs_iter.clone();
    let fn_args = extract_arg_idents_vec(fn_inputs_no_self);
    let fn_vis = &base_fn.vis;
    let fn_name = &base_fn.sig.ident;
    let fn_block = &base_fn.block;
    let fn_name_internal = format_ident!("__{}__", fn_name);
    let param_code = get_param_block(fn_inputs, Some(param.clone()));

    quote! {
        #[allow(unused_variables)]
        fn #fn_name_internal(#(#fn_inputs_iter,)*) #fn_output {
            #fn_block
        }

        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            use pluginop::api::ToPluginizableConnection;
            use pluginop::IntoWithPH;
            use pluginop::TryIntoWithPH;
            use pluginop::octets::OctetsMutPtr;
            use pluginop::octets::OctetsPtr;
            let ph = self.get_pluginizable_connection().map(|pc| pc.get_ph_mut());
            if let Some(ph) = ph.filter(|ph| ph.provides(& #po(#param), pluginop::common::Anchor::Replace)) {
                let params = #param_code;
                let res = ph.call(
                    & #po(#param),
                    params,
                );
                ph.clear_bytes_content();

                #ret_block
            } else {
                self.#fn_name_internal(#(#fn_args,)*)
            }
        }
    }
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

    let ret_block = get_ret_block(&base_fn.sig.output);
    let out = get_out_block(&base_fn, po, &ret_block);

    // println!("output is\n{}", out);

    out.into()
}

#[proc_macro_attribute]
pub fn pluginop_result(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let ret_block = get_ret_result_block(&base_fn.sig.output);
    let out = get_out_block(&base_fn, po, &ret_block);

    // println!("output is\n{}", out);

    out.into()
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

    let ret_block = get_ret_block(&base_fn.sig.output);
    get_out_param_block(param, &base_fn, &po, &ret_block).into()
}

#[proc_macro_attribute]
pub fn pluginop_result_param(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let attrs_args = match MacroArgs::from_list(&attrs) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let po = attrs_args.po;
    let param = attrs_args.param;

    let base_fn = parse_macro_input!(item as ItemFn);

    let ret_block = get_ret_result_block(&base_fn.sig.output);
    let out = get_out_param_block(param, &base_fn, &po, &ret_block);

    // println!("output is\n{}", out);

    out.into()
}
