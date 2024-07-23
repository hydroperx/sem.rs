use crate::*;

pub struct ProcessingStep3_6();

impl ProcessingStep3_6 {
    pub fn exec(&self, host: &mut SModelHost, node: &Rc<SmType>, smtype: &Symbol, base_accessor: &str, smodel_path: &proc_macro2::TokenStream) {
        let smtype_name_debug = format!("{}()", smtype.name());
        let smtype_name = node.name.clone();
        let attributes = node.attributes.clone();
        let visi = node.visibility.clone();

        // Define the structure M, as in
        //
        // ```
        // #[derive(Clone)]
        // struct M(Weak<#DATA::M>);
        // ```
        //
        // or as in:
        //
        // ```
        // #[derive(Clone, PartialEq, Hash)]
        // struct M(InheritedM);
        // ```
        //
        // if there is an inherited base.
        if let Some(inherits) = smtype.inherits() {
            let inherited_name = Ident::new(&inherits.name(), Span::call_site());
            host.output.extend::<TokenStream>(quote! {
                #(#attributes)*
                #[derive(Clone, PartialEq, Hash)]
                #visi struct #smtype_name(#inherited_name);

                impl ::std::ops::Deref for #smtype_name {
                    type Target = #inherited_name;
                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }
            }.try_into().unwrap());
        } else {
            let data_id = Ident::new(DATA, Span::call_site());
            let smtype_data_name = Ident::new(&format!("{DATA_PREFIX}{}", smtype.name()), Span::call_site());
            host.output.extend::<TokenStream>(quote! {
                #(#attributes)*
                #[derive(Clone)]
                #visi struct #smtype_name(::std::rc::Weak<#data_id::#smtype_data_name>);

                impl PartialEq for #smtype_name {
                    fn eq(&self, other: &Self) -> bool {
                        self.0.ptr_eq(&other.0)
                    }
                }

                impl ::std::hash::Hash for #smtype_name {
                    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                        self.0.as_ptr().hash(state)
                    }
                }
            }.try_into().unwrap());
        }

        // Implement Eq and Debug
        host.output.extend::<TokenStream>(quote! {
            impl Eq for #smtype_name {}

            impl ::std::fmt::Debug for #smtype_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, #smtype_name_debug)
                }
            }
        }.try_into().unwrap());

        // Output From<M> for InheritedM implementation (covariant conversion)
        let mut base = "v.0.0".to_owned();
        let mut m = smtype.clone();
        while let Some(m1) = m.inherits() {
            let inherited_name = Ident::new(&m1.name(), Span::call_site());
            let base_tokens = proc_macro2::TokenStream::from_str(&base).unwrap();
            host.output.extend::<TokenStream>(quote! {
                impl From<#smtype_name> for #inherited_name {
                    fn from(v: #smtype_name) -> Self {
                        #inherited_name(#base_tokens.clone())
                    }
                }
            }.try_into().unwrap());
            m = m1;
            base = format!("{base}.0");
        }

        // Output a TryFrom<M> for SubtypeM implementation (contravariant conversion)
        for sm in smtype.subtypes().iter() {
            self.contravariance(host, &base_accessor.replacen("self", "v", 1), smtype, &sm, smodel_path);
        }
    }

    fn contravariance(&self, host: &mut SModelHost, base_accessor: &str, base_smtype: &Symbol, subtype: &Symbol, smodel_path: &proc_macro2::TokenStream) {
        let base_smtype_name = Ident::new(&base_smtype.name(), Span::call_site());
        let subtype_name = Ident::new(&subtype.name(), Span::call_site());
        let m = proc_macro2::TokenStream::from_str(&self.match_contravariant(&subtype.asc_smtype_list(), 0, &format!("{base_accessor}.upgrade().unwrap()"), &base_accessor, smodel_path)).unwrap();

        host.output.extend::<TokenStream>(quote! {
            impl TryFrom<#base_smtype_name> for #subtype_name {
                type Error = #smodel_path::SModelError;
                fn try_from(v: #base_smtype_name) -> Result<Self, Self::Error> {
                    #m
                }
            }
        }.try_into().unwrap());

        for sm1 in subtype.subtypes().iter() {
            self.contravariance(host, base_accessor, base_smtype, &sm1, smodel_path);
        }
    }

    /// Matches a contravariant type.
    /// 
    /// * `base` is assumed to be a `Rc<#DATA::M>` value.
    /// * `original_base` is assumed to be a `Weak<#DATA::FirstM>` value.
    fn match_contravariant(&self, asc_smtype_list: &[Symbol], smtype_index: usize, base: &str, original_base: &str, smodel_path: &proc_macro2::TokenStream) -> String {
        let (smtype, inherited) = if smtype_index + 1 >= asc_smtype_list.len() {
            (asc_smtype_list[smtype_index].clone(), None)
        } else {
            (asc_smtype_list[smtype_index + 1].clone(), Some(asc_smtype_list[smtype_index].clone()))
        };

        let Some(inherited) = inherited else {
            return format!("Ok({})", Symbol::create_layers_over_weak_root(original_base, asc_smtype_list));
        };
        format!("if let {DATA}::{}::{}(_o) = &{base}.{DATA_VARIANT_FIELD} {{ {} }} else {{ Err({}::SModelError::Contravariant) }}",
            DATA_VARIANT_PREFIX.to_owned() + &inherited.name(),
            DATA_PREFIX.to_owned() + &smtype.name(),
            self.match_contravariant(asc_smtype_list, smtype_index + 1, "_o", original_base, smodel_path),
            smodel_path.to_string())
    }
}