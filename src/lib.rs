extern crate either;
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use either::Either;
use proc_macro as pm;
use proc_macro2 as pm2;
use syn::parse_macro_input;
use syn::spanned::Spanned;

mod kw {
    syn::custom_keyword!(model);
    syn::custom_keyword!(tested);
    syn::custom_keyword!(type_parameters);
    syn::custom_keyword!(methods);
    syn::custom_keyword!(equal);
    syn::custom_keyword!(equal_with);
}

enum PassingMode {
    ByValue,
    ByRef,
    ByRefMut
}

struct Argument {
    name: syn::Ident,
    ty: syn::Type,
    passing_mode: PassingMode
}

struct Method {
    name: syn::Ident,
    // self_mut: bool,
    inputs: Vec<Argument>,
    process_result: Option<syn::Path>,
    // output: syn::Type
}

impl syn::parse::Parse for Method {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let method_item: syn::TraitItemMethod = input.parse()?;
        
        if let Some(ref defaultness) = method_item.default {
            return Err(syn::Error::new(defaultness.span(), "unexpected `default`"));
        }
        if let Some(ref constness) = method_item.sig.constness {      
            return Err(syn::Error::new(constness.span(), "unexpected `const`"));
        }
        if let Some(ref asyncness) = method_item.sig.asyncness {      
            return Err(syn::Error::new(asyncness.span(), "unexpected `async`"));
        }
        if let Some(ref unsafety) = method_item.sig.unsafety {      
            return Err(syn::Error::new(unsafety.span(), "unexpected `unsafe`"));
        }

        let (receivers, args) = method_item.sig.inputs.iter().map(|input| match input {
            syn::FnArg::Receiver(receiver) =>
                Either::Left(receiver),
            syn::FnArg::Typed(syn::PatType { ty, pat, .. }) => {
                let ident = match **pat {
                    syn::Pat::Ident(syn::PatIdent { ref ident, .. }) => ident.clone(),
                    ref pat => {
                        //error_stream.extend(
                        //    syn::Error::new(pat.span(), "unexpected `unsafe`").to_compile_error());
                        syn::Ident::new("_", pat.span())
                    }
                };
                match **ty {
                    syn::Type::Reference(syn::TypeReference { ref mutability, ref elem, .. }) =>
                        Either::Right(Argument {
                            name: ident,
                            ty: (**elem).clone(),
                            passing_mode: if mutability.is_some() {
                                PassingMode::ByRefMut
                            } else {
                                PassingMode::ByRef
                            }
                        }),
                    ref ty =>
                        Either::Right(Argument {
                            name: ident,
                            ty: ty.clone(),
                            passing_mode: PassingMode::ByValue
                        })
                }       
            }
        }).partition::<Vec<_>, _>(Either::is_left);

        let receivers: Vec<_> = receivers.into_iter().filter_map(Either::left).collect();
        let args: Vec<_> = args.into_iter().filter_map(Either::right).collect();

        let receiver = receivers.first();
        match receiver {
            Some(receiver) => {
                if receiver.reference.is_none() {
                    return Err(syn::Error::new(receiver.span(), "unexpected by-value receiver"));
                }
            }
            None => {
                return Err(syn::Error::new(method_item.span(), "unexpected method with no receiver"));
            }
        }

        Ok(Self {
            name: method_item.sig.ident,
            // self_mut: receiver.map_or(false, |r| r.mutability.is_some()),
            process_result: None,
            inputs: args,
            /*output: match method_item.sig.output {
                syn::ReturnType::Default =>
                    syn::parse_str("()").unwrap(),
                syn::ReturnType::Type(_, typ) =>
                    (*typ).clone()
            }*/
        })
    }
}

struct Specification {
    model: syn::Path,
    tested: syn::Path,
    type_params: Vec<syn::TypeParam>,
    methods: Vec<Method> 
}

impl syn::parse::Parse for Specification {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{braced, parenthesized, Token};

        let _: kw::model = input.parse()?;
        let _: Token![=] = input.parse()?;
        let model: syn::Path = input.parse()?;
        let _: Token![,] = input.parse()?;

        let _: kw::tested = input.parse()?;
        let _: Token![=] = input.parse()?;
        let tested: syn::Path = input.parse()?;
        let _: Token![,] = input.parse()?;

        let _: kw::type_parameters = input.parse()?;
        let _: Token![=] = input.parse()?;
        let generics: syn::Generics = input.parse()?;
        let type_params = generics.type_params().cloned().collect();
        let _: Token![,] = input.parse()?;

        let mut methods: Vec<Method> = vec![];
        let outer;
        let mut inner;
        let path;
        
        let _: kw::methods = input.parse()?;
        braced!(outer in input);
       
        let _: kw::equal = outer.parse()?;
        braced!(inner in outer);
        while !inner.is_empty() {
            methods.push(inner.parse()?);
        }

        let _: kw::equal_with = outer.parse()?;
        parenthesized!(path in outer);
        let processor: syn::Path = path.parse()?;
        braced!(inner in outer);
        while !inner.is_empty() {
            let mut method: Method = inner.parse()?;
            method.process_result = Some(processor.clone());
            methods.push(method);
        }

        Ok(Self {
            model: model,
            tested: tested,
            type_params: type_params,
            methods: methods
        })
    }
}

impl quote::ToTokens for Method {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        use pm2::{Delimiter, Group, Punct, Spacing};
        use quote::{TokenStreamExt};

        tokens.append(self.name.clone());
        
        if !self.inputs.is_empty() {
            let mut fields = pm2::TokenStream::new();
            for input in self.inputs.iter() {
                fields.append(input.name.clone());
                fields.append(Punct::new(':', Spacing::Joint));
                input.ty.to_tokens(&mut fields);
                fields.append(Punct::new(',', Spacing::Joint));
            }
            tokens.append(Group::new(Delimiter::Brace, fields));
        }
    }
}

struct MethodTest<'s> {
    method: &'s Method
}

impl<'s> quote::ToTokens for MethodTest<'s> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let args: Vec<_> = self.method.inputs.iter().map(|input| {
            let input_name = &input.name;
            match input.passing_mode {
                PassingMode::ByValue =>
                    quote! { #input_name.clone() },
                PassingMode::ByRef =>
                    quote! { &#input_name },
                PassingMode::ByRefMut =>
                    quote! { &mut #input_name }
            }
        }).collect();
       
        let method_name = &self.method.name;
       
        let keys: Vec<_> = self.method.inputs.iter().map(|input| &input.name).collect();
        let pattern = if keys.is_empty() {
            quote! { Op::#method_name }
        } else {
            quote! { Op::#method_name { #(#keys),* } }
        };

        let process_model_res = self.method.process_result
            .as_ref()
            .map(|p| quote! { #p(model_res) })
            .unwrap_or(quote!{ model_res });
        let process_tested_res = self.method.process_result
            .as_ref()
            .map(|p| quote! { #p(tested_res) })
            .unwrap_or(quote!{ tested_res });

        tokens.extend(quote! {
            #pattern => {
                let model_res = model.#method_name(#(#args),*);
                let tested_res = tested.#method_name(#(#args),*);
                let model_res = #process_model_res;
                let tested_res = #process_tested_res;
                assert_eq!(model_res, tested_res);
            }
        });
    }
}

struct OperationEnum<'s> {
    spec: &'s Specification
}

impl<'s> quote::ToTokens for OperationEnum<'s> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let type_params_with_bounds = &self.spec.type_params;
        let type_params: Vec<_> = type_params_with_bounds
            .iter()
            .map(|tp| tp.ident.clone())
            .collect();

        let model = &self.spec.model;
        let tested = &self.spec.tested;
        let variants = &self.spec.methods;

        let method_tests: Vec<_> = self.spec.methods.iter().map(|method| {
            MethodTest { method: method }
        }).collect();

        tokens.extend(quote! {
            #[allow(non_camel_case_types)]
            #[derive(Arbitrary, Clone, Debug)]
            pub enum Op<#(#type_params_with_bounds),*> {
                #(#variants),*
            }

            impl<#(#type_params_with_bounds),*> Op<#(#type_params),*> {
                pub fn execute(self, model: &mut #model, tested: &mut #tested) {
                    match self {
                        #(#method_tests),* 
                    }
                }
            }
        })
    }
}

#[proc_macro]
pub fn arbitrary_stateful_operations(input: pm::TokenStream) -> pm::TokenStream {
    let parsed_spec = parse_macro_input!(input as Specification);

    let operation_enum = OperationEnum {
        spec: &parsed_spec
    };

    let output = quote! {
        mod op {
            use super::*;
            #operation_enum
        }
    };
    
    output.into()
}

