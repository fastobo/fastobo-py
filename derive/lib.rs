#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;
use syn::parse_quote;
use syn::spanned::Spanned;

// ---

#[proc_macro_derive(ClonePy)]
pub fn clonepy_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    match &ast.data {
        syn::Data::Enum(e) => TokenStream::from(clonepy_impl_enum(&ast, &e)),
        syn::Data::Struct(s) => TokenStream::from(clonepy_impl_struct(&ast, &s)),
        _ => panic!("#[derive(ClonePy)] only supports enum or structs"),
    }
}

fn clonepy_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build clone_py for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => #name(x.clone_py(py))));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        #[allow(unused)]
        impl ClonePy for #name {
            fn clone_py(&self, py: Python) -> Self {
                use self::#name::*;
                let gil = pyo3::Python::acquire_gil();
                let py = gil.python();

                match self {
                    #(#variants,)*
                }
            }
        }
    };

    expanded
}

fn clonepy_impl_struct(ast: &syn::DeriveInput, _en: &syn::DataStruct) -> TokenStream2 {
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl ClonePy for #name {
            fn clone_py(&self, _py: Python) -> Self {
                self.clone()
            }
        }
    };

    expanded
}

// ---

#[proc_macro_derive(PyWrapper, attributes(wraps))]
pub fn pywrapper_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let mut output = TokenStream2::new();
    if let syn::Data::Enum(e) = &ast.data {
        // output.extend(clone_impl_enum(&ast, &e));
        output.extend(topyobject_impl_enum(&ast, &e));
        output.extend(intopyobject_impl_enum(&ast, &e));
        output.extend(frompyobject_impl_enum(&ast, &e));
        output.extend(aspyptr_impl_enum(&ast, &e));
        output.extend(intopy_impl_enum(&ast, &e));
        // output.extend(pyobjectprotocol_impl_enum(&ast, &e))
    } else {
        panic!("only supports enums");
    }

    TokenStream::from(output)
}

fn aspyptr_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => x.as_ptr()));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::AsPyPointer for #name {
            fn as_ptr(&self) -> *mut pyo3::ffi::PyObject {
                use self::#name::*;

                match self {
                    #(#variants,)*
                }
            }
        }
    };

    expanded
}

fn topyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => x.to_object(py)));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::ToPyObject for #name {
            fn to_object(&self, py: Python) -> pyo3::PyObject {
                use self::#name::*;
                match self {
                    #(#variants,)*
                }
            }
        }
    };

    expanded
}

fn intopyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => pyo3::IntoPy::into_py(x, py)));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::IntoPy<pyo3::PyObject> for #name {
            fn into_py(self, py: Python) -> pyo3::PyObject {
                use self::#name::*;
                match self {
                    #(#variants,)*
                }
            }
        }
    };

    expanded
}

fn frompyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let wrapped = &ast.ident;
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        // Name of the variant
        let name = &variant.ident;

        // Name of the class wrapped by the variant in a `Py<...>` reference.
        let ty = variant.fields.iter().next().unwrap().ty.clone();
        let args = match ty {
            syn::Type::Path(path) => path.path.segments.iter().next().unwrap().arguments.clone(),
            _ => unreachable!(),
        };
        let arg = match &args {
            syn::PathArguments::AngleBracketed(ref br) => br.args.iter().next().unwrap(),
            _ => unreachable!(),
        };
        let path = match &arg {
            syn::GenericArgument::Type(syn::Type::Path(ref path)) => path.path.clone(),
            _ => unreachable!(),
        };
        let lit = &syn::LitStr::new(
            &path.segments.iter().next().unwrap().ident.to_string(),
            path.segments.iter().next().unwrap().ident.span(),
        );

        variants.push(quote!(#lit => ob.extract::<pyo3::Py<#path>>().map(#wrapped::#name)));
    }

    let meta = ast
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident(&syn::Ident::new("wraps", attr.span())))
        .expect("could not find #[wraps] attribute")
        .parse_meta()
        .expect("could not parse #[wraps] argument");

    let base = match meta {
        syn::Meta::List(l) => match l.nested.iter().next().unwrap() {
            syn::NestedMeta::Meta(syn::Meta::Path(p)) => p.clone(),
            _ => panic!("#[wraps] argument must be a class ident"),
        },
        _ => panic!("#[wraps] argument must be a class ident"),
    };

    // Build FromPyObject implementation
    let err_sub = syn::LitStr::new(
        &format!("subclassing {} is not supported", quote!(#base)),
        base.span(),
    );
    let err_ty = syn::LitStr::new(
        &format!("expected {} instance, {{}} found", quote!(#base)),
        base.span(),
    );
    let expanded = quote! {
        #[automatically_derived]
        impl<'source> pyo3::FromPyObject<'source> for #wrapped {
            fn extract(ob: &'source pyo3::types::PyAny) -> pyo3::PyResult<Self> {
                use pyo3::AsPyPointer;

                let qualname = ob.get_type().name()?;
                let ty = match qualname.rfind('.') {
                    Some(idx) => &qualname[idx+1..],
                    None => &qualname,
                };

                if ob.is_instance::<#base>()? {
                    match ty.as_ref() {
                        #(#variants,)*
                        _ => Err(pyo3::exceptions::PyTypeError::new_err(#err_sub))
                    }
                } else {
                    Err(pyo3::exceptions::PyTypeError::new_err(format!(
                        #err_ty,
                        ob.get_type().name()?,
                    )))
                }
            }
        }
    };

    expanded
}

fn intopy_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build IntoPy for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(
            #name(x) => (&x.as_ref(py).borrow()).clone_py(py).into_py(py)
        ));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::IntoPy<fastobo::ast::#name> for &#name {
            fn into_py(self, py: Python) -> fastobo::ast::#name {
                use std::ops::Deref;
                use self::#name::*;
                match self {
                    #(#variants,)*
                }
            }
        }
    };

    expanded
}

// ---

#[proc_macro_attribute]
pub fn listlike(attr: TokenStream, input: TokenStream) -> TokenStream {
    // extract proc-macro arguments
    let meta = parse_macro_input!(attr as syn::AttributeArgs);
    let field: syn::Ident = if let syn::Lit::Str(ref s) = meta
        .iter()
        .filter_map(|m| match m {
            syn::NestedMeta::Meta(m) => Some(m),
            _ => None,
        })
        .filter_map(|m| match m {
            syn::Meta::NameValue(nv) => Some(nv),
            _ => None,
        })
        .find(|nv| nv.path.get_ident() == Some(&syn::Ident::new("field", Span::call_site())))
        .expect("#[pylist] requires a `field` argument")
        .lit
    {
        s.parse()
            .expect("`field` argument of #[pylist] is not a valid identifier")
    } else {
        panic!("`field` argument of #[pylist] must be a string");
    };
    let ty: syn::Type = if let syn::Lit::Str(ref s) = meta
        .iter()
        .filter_map(|m| match m {
            syn::NestedMeta::Meta(m) => Some(m),
            _ => None,
        })
        .filter_map(|m| match m {
            syn::Meta::NameValue(nv) => Some(nv),
            _ => None,
        })
        .find(|nv| nv.path.get_ident() == Some(&syn::Ident::new("type", Span::call_site())))
        .expect("#[pylist] requires a `type` argument")
        .lit
    {
        s.parse()
            .expect("`type` argument of #[pylist] is not a valid type")
    } else {
        panic!("`type` argument of #[pylist] must be a string");
    };

    // add additional methods to the impl block
    let ast = parse_macro_input!(input as syn::ItemImpl);
    TokenStream::from(listlike_impl_methods(&field, &ty, ast))
}

fn listlike_impl_methods(
    field: &syn::Ident,
    ty: &syn::Type,
    mut imp: syn::ItemImpl,
) -> TokenStream2 {
    imp.items.push(parse_quote! {
        /// Append object to the end of the list.
        ///
        /// Raises:
        ///     TypeError: when the object is not of the right type for
        ///         this container (see type-level documentation for the
        ///         required type).
        #[pyo3(text_signature = "(self, object)")]
        fn append(&mut self, object: &PyAny) -> PyResult<()> {
            let item = <#ty as pyo3::prelude::FromPyObject>::extract(object)?;
            self.#field.push(item);
            Ok(())
        }
    });
    imp.items.push(parse_quote! {
        /// Remove all items from list.
        #[pyo3(text_signature = "(self)")]
        fn clear(&mut self) {
            self.#field.clear();
        }
    });
    imp.items.push(parse_quote! {
        /// Return a shallow copy of the list.
        #[pyo3(text_signature = "(self)")]
        fn copy(&self) -> PyResult<Py<Self>> {
            let gil = Python::acquire_gil();
            let copy = self.clone_py(gil.python());
            Py::new(gil.python(), copy)
        }
    });
    imp.items.push(parse_quote! {
        /// Return number of occurrences of value.
        ///
        /// Raises:
        ///     TypeError: when the object is not of the right type for
        ///         this container (see type-level documentation for the
        ///         required type).
        #[pyo3(text_signature = "(self, value)")]
        fn count(&mut self, value: &PyAny) -> PyResult<usize> {
            let item = <#ty as pyo3::prelude::FromPyObject>::extract(value)?;
            Ok(self.#field.iter().filter(|&x| *x == item).count())
        }
    });
    // |  extend($self, iterable, /)
    // |      Extend list by appending elements from the iterable.
    // |
    // |  index(self, value, start=0, stop=9223372036854775807, /)
    // |      Return first index of value.
    // |
    // |      Raises ValueError if the value is not present.
    // |
    imp.items.push(parse_quote! {
        /// Insert `object` before `index`.
        ///
        /// If `index` is greater than the number of elements in the list,
        /// `object` will be added at the end of the list.
        #[pyo3(text_signature = "(self, index, object)")]
        fn insert(&mut self, mut index: isize, object: &PyAny) -> PyResult<()> {
            let item = <#ty as pyo3::prelude::FromPyObject>::extract(object)?;
            if index >= self.#field.len() as isize {
                self.#field.push(item);
            } else {
                if index < 0 {
                    index %= self.#field.len() as isize;
                }
                self.#field.insert(index as usize, item);
            }
            Ok(())
        }
    });
    imp.items.push(parse_quote! {
        /// Remove and return item at index (default last).
        ///
        /// Raises:
        ///     IndexError: when list is empty or index is out of range.
        #[args(index="-1")]
        #[pyo3(text_signature = "(self, index=-1)")]
        fn pop(&mut self, mut index: isize) -> PyResult<#ty> {
            // Wrap once to allow negative indexing
            if index < 0 {
                index += self.#field.len() as isize;
            }
            // Pop if the index is in vector bounds
            if index >= 0 && index < self.#field.len() as isize {
                Ok(self.#field.remove(index as usize))
            } else {
                Err(pyo3::exceptions::PyIndexError::new_err("pop index out of range"))
            }
        }
    });
    // |  remove(self, value, /)
    // |      Remove first occurrence of value.
    // |
    // |      Raises ValueError if the value is not present.
    imp.items.push(parse_quote! {
        /// Reverse *IN PLACE*.
        #[pyo3(text_signature = "(self)")]
        fn reverse(&mut self) {
            self.#field.reverse()
        }
    });
    // |  sort(self, /, *, key=None, reverse=False)
    // |      Stable sort *IN PLACE*.
    quote!(#imp)
}

// ---

#[proc_macro_derive(FinalClass, attributes(base))]
pub fn finalclass_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    match &ast.data {
        syn::Data::Struct(s) => TokenStream::from(finalclass_impl_struct(&ast, &s)),
        _ => panic!("#[derive(FinalClass)] only supports structs"),
    }
}

fn finalclass_impl_struct(ast: &syn::DeriveInput, _st: &syn::DataStruct) -> TokenStream2 {
    // Get the name of the wrapped struct.
    let name = &ast.ident;

    // Get the name of the base class.
    let meta = ast
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident(&syn::Ident::new("base", attr.span())))
        .expect("could not find #[base] attribute")
        .parse_meta()
        .expect("could not parse #[base] argument");
    let base = match meta {
        syn::Meta::List(l) => match l.nested.iter().next().unwrap() {
            syn::NestedMeta::Meta(syn::Meta::Path(p)) => p.clone(),
            _ => panic!("#[base] argument must be a class ident"),
        },
        _ => panic!("#[base] argument must be a class ident"),
    };

    // derive an implementation of PyClassInitializer using simply the
    // default value of the base class as the initializer value
    quote! {
        impl FinalClass for #name {}
        impl Into<pyo3::pyclass_init::PyClassInitializer<#name>> for #name {
            fn into(self) ->  pyo3::pyclass_init::PyClassInitializer<Self> {
                <#base as AbstractClass>::initializer()
                    .add_subclass(self)
            }
        }
    }
}

// ---

#[proc_macro_derive(AbstractClass, attributes(base))]
pub fn abstractclass_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    match &ast.data {
        syn::Data::Struct(s) => TokenStream::from(abstractclass_impl_struct(&ast, &s)),
        _ => panic!("#[derive(AbstractClass)] only supports structs"),
    }
}

fn abstractclass_impl_struct(ast: &syn::DeriveInput, _st: &syn::DataStruct) -> TokenStream2 {
    // Get the name of the wrapped struct.
    let name = &ast.ident;

    // Get the name of the base class.
    let meta = ast
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident(&syn::Ident::new("base", attr.span())))
        .expect("could not find #[base] attribute")
        .parse_meta()
        .expect("could not parse #[base] argument");
    let base = match meta {
        syn::Meta::List(l) => match l.nested.iter().next().unwrap() {
            syn::NestedMeta::Meta(syn::Meta::Path(p)) => p.clone(),
            _ => panic!("#[base] argument must be a class ident"),
        },
        _ => panic!("#[base] argument must be a class ident"),
    };

    // derive an implementation of PyClassInitializer using simply the
    // default value of the base class as the initializer value
    quote! {
        impl AbstractClass for #name {
            fn initializer() -> pyo3::pyclass_init::PyClassInitializer<Self> {
                <#base as AbstractClass>::initializer()
                    .add_subclass(Self {})
            }
        }

    }
}
