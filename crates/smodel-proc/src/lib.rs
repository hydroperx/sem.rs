#![feature(proc_macro_diagnostic)]

use proc_macro2::Span;

#[macro_use]
mod shared_array;
use shared_array::*;

#[macro_use]
mod shared_map;
use shared_map::*;

mod symbol;
use symbol::*;

mod tree_semantics;
use syn::spanned::Spanned;
use tree_semantics::*;

mod processing;
use processing::*;

// use std::iter::FromIterator;
use proc_macro::TokenStream;
// use proc_macro2::Span;
use quote::{quote, ToTokens};
// use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::Comma;
// use syn::spanned::Spanned;
use syn::{braced, parenthesized, parse_macro_input, Attribute, Expr, FnArg, Generics, Ident, Pat, Path, Stmt, Token, Type, Visibility, WhereClause};

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use by_address::ByAddress;

/// Data module name.
const DATA: &'static str = "__data__";

const DATA_PREFIX: &'static str = "__data_";

/// Field name used for holding an enumeration of subtypes.
const DATA_VARIANT_FIELD: &'static str = "__variant";

/// Prefix used for enumerations of subtypes.
const DATA_VARIANT_PREFIX: &'static str = "__variant_";

/// Variant name used for indicating that no subtype is instantiated.
const DATA_VARIANT_NO_SUBTYPE: &'static str = "__Nothing";

struct SmTypeTree {
    smodel_path: proc_macro2::TokenStream,
    arena_type_name: proc_macro2::TokenStream,
    data_types: Vec<Rc<SmType>>,
}

struct SmType {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    inherits: Option<Ident>,
    fields: Vec<Rc<SmTypeField>>,
    constructor: Option<SmTypeConstructor>,
    methods: Vec<Rc<SmTypeMethod>>,
}

struct SmTypeField {
    is_ref: bool,
    name: Ident,
    type_annotation: Type,
    default_value: Expr,
}

enum SmTypeMethodOrConstructor {
    Method(SmTypeMethod),
    Constructor(SmTypeConstructor),
}

struct SmTypeConstructor {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    generics: Generics,
    name: Ident,
    inputs: Punctuated<FnArg, Comma>,
    super_arguments: Punctuated<Expr, Comma>,
    statements: Vec<Stmt>,
}

struct SmTypeMethod {
    attributes: RefCell<Vec<Attribute>>,
    visibility: Visibility,
    is_override: bool,
    name: Ident,
    generics: Generics,
    inputs: Punctuated<FnArg, Comma>,
    result_type: Option<Type>,
    statements: proc_macro2::TokenStream,
}

impl Parse for SmTypeTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut smodel_path: Option<Path> = None;
        if input.peek(Token![mod]) {
            input.parse::<Token![mod]>()?;
            input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            smodel_path = Some(parse_full_qualified_id(input)?);
            input.parse::<Token![;]>()?;
        }
        let arena_type_name = parse_smtype_arena_type_name(input)?.to_token_stream();
        let mut data_types = vec![];
        while !input.is_empty() {
            data_types.push(Rc::new(input.parse::<SmType>()?));
        }
        Ok(Self {
            smodel_path: smodel_path.map(|p| p.to_token_stream()).unwrap_or(proc_macro2::TokenStream::from_str("::smodel").unwrap()),
            arena_type_name,
            data_types,
        })
    }
}

fn parse_full_qualified_id(input: ParseStream) -> Result<Path> {
    Ok(Path::parse_mod_style(input)?)
}

impl Parse for SmType {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse::<Visibility>()?;
 
        input.parse::<Token![struct]>()?;
 
        let name = input.parse::<Ident>()?;
        let name_str = name.to_string();

        // Inherits
        let mut inherits: Option<Ident> = None;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            inherits = Some(input.parse::<Ident>()?);
        }

        let mut fields: Vec<Rc<SmTypeField>> = vec![];
        let mut constructor: Option<SmTypeConstructor> = None;
        let mut methods: Vec<Rc<SmTypeMethod>> = vec![];
        let braced_content;
        let _ = braced!(braced_content in input);

        while !braced_content.is_empty() {
            if braced_content.peek(Token![let]) {
                fields.push(Rc::new(parse_smtype_field(&braced_content)?));
            } else {
                match parse_smtype_method(&braced_content, &name_str)? {
                    SmTypeMethodOrConstructor::Constructor(ctor) => {
                        constructor = Some(ctor);
                    },
                    SmTypeMethodOrConstructor::Method(m) => {
                        methods.push(Rc::new(m));
                    },
                }
            }
        }

        Ok(Self {
            attributes,
            visibility,
            name,
            inherits,
            fields,
            constructor,
            methods,
        })
    }
}

fn parse_smtype_field(input: ParseStream) -> Result<SmTypeField> {
    input.parse::<Token![let]>()?;
    let is_ref = if input.peek(Token![ref]) {
        input.parse::<Token![ref]>()?;
        true
    } else {
        false
    };
    let name = input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let type_annotation = input.parse::<Type>()?;
    input.parse::<Token![=]>()?;
    let default_value = input.parse::<Expr>()?;
    input.parse::<Token![;]>()?;

    Ok(SmTypeField {
        is_ref,
        name,
        type_annotation,
        default_value,
    })
}

fn parse_smtype_method(input: ParseStream, smtype_name: &str) -> Result<SmTypeMethodOrConstructor> {
    let attributes = Attribute::parse_outer(input)?;
    let visibility = input.parse::<Visibility>()?;
    let is_override = if input.peek(Token![override]) {
        input.parse::<Token![override]>()?;
        true
    } else {
        false
    };
    input.parse::<Token![fn]>()?;
    let mut is_constructor = false;
    let id = input.parse::<Ident>()?;
    if !is_override && id.to_string() == smtype_name {
        // id.span().unwrap().error("Identifier must be equals \"constructor\"").emit();
        is_constructor = true;
    }
    let mut generics = input.parse::<Generics>()?;

    let parens_content;
    parenthesized!(parens_content in input);
    let inputs = parens_content.parse_terminated(FnArg::parse, Comma)?;

    let result_type: Option<Type> = if !is_constructor && input.peek(Token![->]) {
        input.parse::<Token![->]>()?;
        Some(input.parse::<Type>()?)
    } else {
        None
    };

    generics.where_clause = if input.peek(Token![where]) { Some(input.parse::<WhereClause>()?) } else { None };

    let braced_content;
    let _ = braced!(braced_content in input);

    if !is_constructor {
        let statements = braced_content.parse::<proc_macro2::TokenStream>()?;
        return Ok(SmTypeMethodOrConstructor::Method(SmTypeMethod {
            attributes: RefCell::new(attributes),
            visibility,
            is_override,
            name: id,
            generics,
            inputs,
            result_type,
            statements,
        }));
    }

    braced_content.parse::<Token![super]>()?;

    let paren_content;
    let _ = parenthesized!(paren_content in braced_content);
    let super_arguments = paren_content.parse_terminated(Expr::parse, Comma)?;
    braced_content.parse::<Token![;]>()?;

    let mut statements = vec![];
    while !braced_content.is_empty() {
        statements.push(braced_content.parse::<Stmt>()?);
    }

    Ok(SmTypeMethodOrConstructor::Constructor(SmTypeConstructor {
        attributes,
        visibility,
        generics,
        name: id,
        inputs,
        super_arguments,
        statements,
    }))
}

fn parse_smtype_arena_type_name(input: ParseStream) -> Result<Path> {
    input.parse::<Token![type]>()?;
    let id = input.parse::<Ident>()?;
    if id.to_string() != "Arena" {
        id.span().unwrap().error("Identifier must be equals \"Arena\"").emit();
    }
    input.parse::<Token![=]>()?;
    let path = Path::parse_mod_style(input)?;
    input.parse::<Token![;]>()?;
    Ok(path)
}

#[proc_macro]
pub fn smodel(input: TokenStream) -> TokenStream {
    let SmTypeTree {
        smodel_path, arena_type_name, data_types
    } = parse_macro_input!(input as SmTypeTree);

    let mut host = SModelHost::new();

    // # Validations

    // 1. Ensure there is at least one data type.

    if data_types.is_empty() {
        panic!("There must be at least one data type.");
    }

    // 2. Ensure the first type inherits no other one.

    if data_types[0].inherits.is_some() {
        data_types[0].name.span().unwrap().error("First data type must inherit no base.").emit();
        return TokenStream::new();
    }
    let base_smtype_data_name = Ident::new(&(DATA_PREFIX.to_string() + &data_types[0].name.to_string()), Span::call_site());

    // 3. Ensure all other types inherit another one.

    for m in data_types[1..].iter() {
        if m.inherits.is_none() {
            m.name.span().unwrap().error("Data type must inherit a base.").emit();
            return TokenStream::new();
        }
    }

    // # Processing steps

    let data_id = Ident::new(DATA, Span::call_site());

    // 1. Output the arena type.
    host.output.extend::<TokenStream>(quote! {
        pub type #arena_type_name = #smodel_path::Arena<#data_id::#base_smtype_data_name>;
    }.try_into().unwrap());

    // 2. Traverse each type in a first pass.
    for smtype_node in data_types.iter() {
        if !ProcessingStep2().exec(&mut host, smtype_node) {
            return TokenStream::new();
        }
    }

    // 3. Traverse each type in a second pass.
    for smtype_node in data_types.iter() {
        let Some(smtype) = host.semantics.get(smtype_node) else {
            continue;
        };

        let asc_smtype_list = smtype.asc_smtype_list();
        let mut field_output = proc_macro2::TokenStream::new();
        let smtype_name = smtype.name();

        // 3.1. Write out the base data accessor
        //
        // A `Weak<#DATA::FirstM>` value.
        //
        // For example, for the basemost data type, this
        // is always "self.0"; for a direct subtype of the basemost
        // data type, this is always "self.0.0".

        let mut base_accessor = "self.0".to_owned();
        let mut m1 = smtype.clone();
        while let Some(m2) = m1.inherits() {
            base_accessor.push_str(".0");
            m1 = m2;
        }

        // 3.2. Traverse each field.
        for field in smtype_node.fields.iter() {
            if !ProcessingStep3_2().exec(&mut host, &smtype, field, &base_accessor, &asc_smtype_list, &mut field_output) {
                return TokenStream::new();
            }
        }

        // 3.3. Contribute a #DATA_VARIANT_FIELD field to #DATA::M
        // holding the enumeration of subtypes.
        let subtype_enum = Ident::new(&(DATA_VARIANT_PREFIX.to_owned() + &smtype_name), Span::call_site());
        let data_variant_field_id = Ident::new(DATA_VARIANT_FIELD, Span::call_site());
        field_output.extend(quote! {
            pub #data_variant_field_id: #subtype_enum,
        });

        // 3.4. Contribute an enumeration of subtypes at the `#DATA` module.
        let mut variants: Vec<proc_macro2::TokenStream> = vec![];
        for subtype in smtype.subtypes().iter() {
            let sn = DATA_PREFIX.to_owned() + &subtype.name();
            variants.push(proc_macro2::TokenStream::from_str(&format!("{sn}(::std::rc::Rc<{sn}>)")).unwrap());
        }
        let data_variant_no_subtype = Ident::new(DATA_VARIANT_NO_SUBTYPE, Span::call_site());
        variants.push(data_variant_no_subtype.to_token_stream());
        host.data_output.extend(quote! {
            pub enum #subtype_enum {
                #(#variants),*
            }
        });

        let smtype_data_id = Ident::new(&format!("{DATA_PREFIX}{}", smtype_name), Span::call_site());

        // 3.5. Define the data structure #DATA::M at the #DATA module output,
        // containing all field output.
        host.data_output.extend(quote! {
            pub struct #smtype_data_id {
                #field_output
            }
        });

        // 3.6. Define the structure M
        ProcessingStep3_6().exec(&mut host, &smtype_node, &smtype, &base_accessor, &smodel_path);

        // 3.7. Define the constructor
        ProcessingStep3_7().exec(&mut host, smtype_node.constructor.as_ref(), &smtype, &asc_smtype_list, &arena_type_name.to_string());

        // 3.8. Traverse each method
        for method in smtype_node.methods.iter() {
            if !ProcessingStep3_8().exec(&mut host, method, &smtype) {
                return TokenStream::new();
            }
        }
    }

    // 4. Traverse each type in a third pass.
    for smtype_node in data_types.iter() {
        let Some(smtype) = host.semantics.get(smtype_node) else {
            continue;
        };

        let smtype_name = smtype.name();
        let smtype_name_id = Ident::new(&smtype_name, Span::call_site());

        // 4.1. Traverse each method
        for method in smtype_node.methods.iter() {
            ProcessingStep4_1().exec(&mut host, method, &smtype);
        }

        // * Contribute a `to::<T: TryFrom<M>>()` method.
        // * Contribute an `is::<T>()` method.
        smtype.method_output().borrow_mut().extend(quote! {
            pub fn to<T: TryFrom<#smtype_name_id, Error = #smodel_path::SModelError>>(&self) -> Result<T, #smodel_path::SModelError> {
                T::try_from(self.clone())
            }
            pub fn is<T: TryFrom<#smtype_name_id, Error = #smodel_path::SModelError>>(&self) -> bool {
                T::try_from(self.clone()).is_ok()
            }
        });

        let method_output = smtype.method_output().borrow().clone();

        // Output the code of all methods to an `impl` block for the data type.
        host.output.extend::<TokenStream>(quote! {
            impl #smtype_name_id {
                #method_output
            }
        }.try_into().unwrap());
    }

    let data_output = host.data_output;

    // 5. Output the `mod #DATA { use super::*; ... }` module with its respective contents
    host.output.extend::<TokenStream>(quote! {
        #[allow(non_camel_case_types, non_snake_case)]
        mod #data_id {
            use super::*;

            #data_output
        }
    }.try_into().unwrap());

    // 5. Return output.
    host.output
}

fn convert_function_input_to_arguments(input: &Punctuated<FnArg, Comma>) -> Punctuated<proc_macro2::TokenStream, Comma> {
    let mut out = Punctuated::<proc_macro2::TokenStream, Comma>::new();
    for arg in input.iter() {
        if let FnArg::Receiver(_) = arg {
            arg.span().unwrap().error("Unexpected receiver.").emit();
            continue;
        } else {
            let FnArg::Typed(pt) = arg else {
                panic!();
            };
            let Pat::Ident(id) = pt.pat.as_ref() else {
                pt.pat.span().unwrap().error("Pattern must be an identifier.").emit();
                continue;
            };
            out.push(id.to_token_stream());
        }
    }
    out
}
