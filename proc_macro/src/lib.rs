use either::Either;
use proc_macro as pm;
use proc_macro2 as pm2;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

mod kw {
    syn::custom_keyword!(equal);
    syn::custom_keyword!(equal_with);
    syn::custom_keyword!(methods);
    syn::custom_keyword!(model);
    syn::custom_keyword!(post);
    syn::custom_keyword!(pre);
    syn::custom_keyword!(tested);
    syn::custom_keyword!(type_parameters);
}

#[allow(clippy::enum_variant_names)]
enum PassingMode {
    ByValue,
    ByRef,
    ByRefMut,
}

struct Argument {
    name: syn::Ident,
    ty: syn::Type,
    passing_mode: PassingMode,
}

struct Method {
    name: syn::Ident,
    // self_mut: bool,
    inputs: Vec<Argument>,
    process_result: Option<syn::Path>,
    // output: syn::Type
}

impl syn::parse::Parse for Method {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let method_item: syn::TraitItemFn = input.parse()?;

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

        let (receivers, args) = method_item
            .sig
            .inputs
            .iter()
            .map(|input| match input {
                syn::FnArg::Receiver(receiver) => Either::Left(receiver),
                syn::FnArg::Typed(syn::PatType { ty, pat, .. }) => {
                    let ident = match **pat {
                        syn::Pat::Ident(syn::PatIdent { ref ident, .. }) => ident.clone(),
                        ref pat => syn::Ident::new("_", pat.span()),
                    };
                    match **ty {
                        syn::Type::Reference(syn::TypeReference {
                            ref mutability,
                            ref elem,
                            ..
                        }) => Either::Right(Argument {
                            name: ident,
                            ty: (**elem).clone(),
                            passing_mode: if mutability.is_some() {
                                PassingMode::ByRefMut
                            } else {
                                PassingMode::ByRef
                            },
                        }),
                        ref ty => Either::Right(Argument {
                            name: ident,
                            ty: ty.clone(),
                            passing_mode: PassingMode::ByValue,
                        }),
                    }
                }
            })
            .partition::<Vec<_>, _>(Either::is_left);

        let receivers: Vec<_> = receivers.into_iter().filter_map(Either::left).collect();
        let args: Vec<_> = args.into_iter().filter_map(Either::right).collect();

        let receiver = receivers.first();
        if let Some(receiver) = receiver {
            if receiver.reference.is_none() {
                return Err(syn::Error::new(
                    receiver.span(),
                    "unexpected by-value receiver",
                ));
            }
        } else {
            return Err(syn::Error::new(
                method_item.span(),
                "unexpected method with no receiver",
            ));
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
    lifetimes: Vec<syn::LifetimeParam>,
    type_params: Vec<syn::TypeParam>,
    methods: Vec<Method>,
    post: Vec<syn::Stmt>,
    pre: Vec<syn::Stmt>,
}

impl syn::parse::Parse for Specification {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        use syn::{braced, parenthesized, Token};

        let mut model: Option<syn::Path> = None;
        let mut tested: Option<syn::Path> = None;
        let mut lifetimes: Vec<syn::LifetimeParam> = vec![];
        let mut type_params: Vec<syn::TypeParam> = vec![];
        let mut methods: Vec<Method> = vec![];
        let mut post: Vec<syn::Stmt> = vec![];
        let mut pre: Vec<syn::Stmt> = vec![];

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::model) {
                let _: kw::model = input.parse()?;
                let _: Token![=] = input.parse()?;
                model = Some(input.parse()?);
            } else if lookahead.peek(kw::tested) {
                let _: kw::tested = input.parse()?;
                let _: Token![=] = input.parse()?;
                tested = Some(input.parse()?);
            } else if lookahead.peek(kw::type_parameters) {
                let _: kw::type_parameters = input.parse()?;
                let _: Token![=] = input.parse()?;
                let generics: syn::Generics = input.parse()?;
                lifetimes = generics.lifetimes().cloned().collect();
                type_params = generics.type_params().cloned().collect();
            } else if lookahead.peek(kw::methods) {
                let outer;
                let mut inner;
                let _: kw::methods = input.parse()?;
                braced!(outer in input);

                while !outer.is_empty() {
                    let lookahead = outer.lookahead1();
                    let process = if lookahead.peek(kw::equal) {
                        let _: kw::equal = outer.parse()?;
                        None
                    } else if lookahead.peek(kw::equal_with) {
                        let _: kw::equal_with = outer.parse()?;
                        let path;
                        parenthesized!(path in outer);
                        Some(path.parse()?)
                    } else {
                        return Err(lookahead.error());
                    };

                    braced!(inner in outer);
                    while !inner.is_empty() {
                        let mut method: Method = inner.parse()?;
                        method.process_result = process.clone();
                        methods.push(method);
                    }
                }
            } else if lookahead.peek(kw::post) {
                let inner;
                let _: kw::post = input.parse()?;
                braced!(inner in input);
                while !inner.is_empty() {
                    post.push(inner.parse()?);
                }
            } else if lookahead.peek(kw::pre) {
                let inner;
                let _: kw::pre = input.parse()?;
                braced!(inner in input);
                while !inner.is_empty() {
                    pre.push(inner.parse()?);
                }
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        let model = model.ok_or_else(|| input.error("missing `model`"))?;
        let tested = tested.ok_or_else(|| input.error("missing `tested`"))?;

        Ok(Self {
            model,
            tested,
            lifetimes,
            type_params,
            methods,
            post,
            pre,
        })
    }
}

impl quote::ToTokens for Method {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        use pm2::{Delimiter, Group, Ident, Punct, Spacing, Span};
        use quote::TokenStreamExt;

        tokens.append(self.name.clone());

        if !self.inputs.is_empty() {
            let mut fields = pm2::TokenStream::new();
            for input in &self.inputs {
                fields.append(input.name.clone());
                fields.append(Punct::new(':', Spacing::Joint));
                if let syn::Type::Slice(_) = input.ty {
                    fields.append(Ident::new("Box", Span::call_site()));
                    fields.append(Punct::new('<', Spacing::Joint));
                    input.ty.to_tokens(&mut fields);
                    fields.append(Punct::new('>', Spacing::Joint));
                } else {
                    input.ty.to_tokens(&mut fields);
                }
                fields.append(Punct::new(',', Spacing::Joint));
            }
            tokens.append(Group::new(Delimiter::Brace, fields));
        }
    }
}

struct MethodTest<'s> {
    method: &'s Method,
    compare: bool,
}

impl<'s> quote::ToTokens for MethodTest<'s> {
    #[allow(clippy::too_many_lines)]
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let args: Vec<_> = self
            .method
            .inputs
            .iter()
            .map(|input| {
                let input_name = &input.name;
                match input.passing_mode {
                    PassingMode::ByValue => quote! { #input_name.clone() },
                    PassingMode::ByRef => quote! { #input_name },
                    PassingMode::ByRefMut => quote! { &mut *#input_name },
                }
            })
            .collect();

        let method_name = &self.method.name;

        let keys: Vec<_> = self.method.inputs.iter().map(|input| &input.name).collect();
        let pattern = if keys.is_empty() {
            quote! { Op::#method_name }
        } else {
            quote! { Op::#method_name { #(ref #keys),* } }
        };

        let process_tested_ret_value = self
            .method
            .process_result
            .as_ref()
            .map(|p| quote! { #p(tested_ret_value) })
            .unwrap_or(quote! { tested_ret_value });

        if self.compare {
            let process_model_ret_value = self
                .method
                .process_result
                .as_ref()
                .map(|p| quote! { #p(model_ret_value) })
                .unwrap_or(quote! { model_ret_value });
            tokens.extend(quote! {
                #pattern => {
                    enum Outcome {
                        Equal,
                        #[cfg(not(fuzzing_debug))]
                        Unequal,
                        #[cfg(fuzzing_debug)]
                        Unequal {
                            model_ret_value_debug: String,
                            tested_ret_value_debug: String,
                        },
                    }

                    enum WhichFailed {
                        None(Outcome),
                        First,
                        Second,
                    }

                    struct GalaxyBrain<'a> {
                        value: WhichFailed,
                        to_update: &'a mut WhichFailed,
                    }

                    impl<'a> Drop for GalaxyBrain<'a> {
                        fn drop(&mut self) {
                            std::mem::swap(self.to_update, &mut self.value);
                        }
                    }

                    let mut f = WhichFailed::First;

                    {
                        let mut guard = GalaxyBrain {
                            value: WhichFailed::First,
                            to_update: &mut f,
                        };

                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let model_ret_value = model.#method_name(#(#args),*);
                            guard.value = WhichFailed::Second;
                            let tested_ret_value = tested.#method_name(#(#args),*);

                            let model_ret_value = #process_model_ret_value;
                            let tested_ret_value = #process_tested_ret_value;

                            let outcome = if model_ret_value == tested_ret_value {
                                Outcome::Equal
                            } else {
                                #[cfg(fuzzing_debug)]
                                {
                                    Outcome::Unequal {
                                        model_ret_value_debug: format!("{:?}", model_ret_value),
                                        tested_ret_value_debug: format!("{:?}", tested_ret_value),
                                    }
                                }
                                #[cfg(not(fuzzing_debug))]
                                Outcome::Unequal
                            };
                            guard.value = WhichFailed::None(outcome);
                        }));
                    }

                    match f {
                        WhichFailed::None(outcome) => {
                            #[cfg(fuzzing_debug)]
                            if let Outcome::Unequal { model_ret_value_debug, tested_ret_value_debug } = outcome {
                                rutenspitz::panic!(
                                    "The return values aren't equal: `{}` != `{}`",
                                    model_ret_value_debug,
                                    tested_ret_value_debug
                                );
                            }
                            #[cfg(not(fuzzing_debug))]
                            if let Outcome::Unequal = outcome {
                                rutenspitz::panic!("The return values aren't equal");
                            }
                        }
                        WhichFailed::First => {
                            // First paniced, see if the second one also does
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                let _ = tested.#method_name(#(#args),*);
                            }));
                            if result.is_ok() {
                                rutenspitz::panic!("Implementation did not panic while the model did");
                            }
                        }
                        WhichFailed::Second => {
                            rutenspitz::panic!("Implementation panicked while the model did not");
                        }
                    }
                }
            });
        } else {
            tokens.extend(quote! {
                #pattern => {
                    let _ = tested.#method_name(#(#args),*);
                }
            });
        }
    }
}

struct OperationEnum<'s> {
    spec: &'s Specification,
}

impl<'s> quote::ToTokens for OperationEnum<'s> {
    #[allow(clippy::cognitive_complexity)]
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let lifetimes = &self.spec.lifetimes;
        let type_params_with_bounds = &self.spec.type_params;
        let type_params: Vec<_> = type_params_with_bounds
            .iter()
            .map(|tp| tp.ident.clone())
            .collect();

        let model = &self.spec.model;
        let tested = &self.spec.tested;
        let variants = &self.spec.methods;

        let comp_method_tests: Vec<_> = self
            .spec
            .methods
            .iter()
            .map(|method| MethodTest {
                method,
                compare: true,
            })
            .collect();

        let method_tests: Vec<_> = self
            .spec
            .methods
            .iter()
            .map(|method| MethodTest {
                method,
                compare: false,
            })
            .collect();

        let format_calls: Vec<_> = self
            .spec
            .methods
            .iter()
            .map(|method| {
                let args: Vec<_> = method
                    .inputs
                    .iter()
                    .map(|input| match input.passing_mode {
                        PassingMode::ByValue => "{:?}",
                        PassingMode::ByRef => "&{:?}",
                        PassingMode::ByRefMut => "&mut {:?}",
                    })
                    .collect();

                let method_name = &method.name;
                let format_str = format!("v.{}({});", method_name, args.join(", "));
                let keys: Vec<_> = method.inputs.iter().map(|input| &input.name).collect();
                let pattern = if keys.is_empty() {
                    quote! { Op::#method_name }
                } else {
                    quote! { Op::#method_name { #(#keys),* } }
                };

                quote! { #pattern =>
                    write!(f, #format_str, #(#keys),*)
                }
            })
            .collect();

        let post = &self.spec.post;
        let pre = &self.spec.pre;

        tokens.extend(quote! {
            #[allow(non_camel_case_types)]
            #[derive(rutenspitz::derive::Arbitrary, rutenspitz::derive::IntoStaticStr, Clone, Debug, PartialEq)]
            pub enum Op<#(#type_params_with_bounds),*> {
                #(#variants),*
            }

            impl<#(#type_params_with_bounds),*> std::fmt::Display for Op<#(#type_params),*> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#format_calls),*
                    }
                }
            }

            impl<#(#type_params_with_bounds),*> Op<#(#type_params),*> {
                pub fn execute <#(#lifetimes),*> (self, tested: &mut #tested) {
                    match &self {
                        #(#method_tests),*
                    }
                }

                pub fn execute_and_compare <#(#lifetimes),*> (self, model: &mut #model, tested: &mut #tested) {
                    #[cfg(not(fuzzing_debug))]
                    rutenspitz::lazy_static::initialize(&rutenspitz::NON_DEBUG_PANIC_HOOK);

                    let op_name: &'static str = From::from(&self);
                    #(#pre)*
                    match &self {
                        #(#comp_method_tests),*
                    }
                    #(#post)*
                }

                #[inline(always)]
                pub fn append_to_trace(&self, trace: &mut String) {
                    #[cfg(fuzzing_debug)]
                    trace.push_str(&format!("{}\n", self.to_string()));
                }
            }
        });
    }
}

#[proc_macro]
pub fn arbitrary_stateful_operations(input: pm::TokenStream) -> pm::TokenStream {
    let parsed_spec = parse_macro_input!(input as Specification);

    let operation_enum = OperationEnum { spec: &parsed_spec };

    let output = quote! {
        mod op {
            use super::*;
            #operation_enum
        }
    };

    output.into()
}
