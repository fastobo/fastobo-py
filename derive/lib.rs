#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
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
                Python::with_gil(|py| {
                    use self::#name::*;
                    match self {
                        #(#variants,)*
                    }
                })
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

// ---

#[proc_macro_derive(EqPy)]
pub fn eqpy_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    match &ast.data {
        syn::Data::Enum(e) => TokenStream::from(eqpy_impl_enum(&ast, &e)),
        syn::Data::Struct(s) => TokenStream::from(eqpy_impl_struct(&ast, &s)),
        _ => panic!("#[derive(EqPy)] only supports enums or structs"),
    }
}

fn eqpy_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // Build eq_py for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote! {
            ( #name(l), #name(r) ) => (*l.bind(py).borrow()).eq_py(&*r.bind(py).borrow(), py)
        });
    }

    // Build eq implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        #[allow(unused)]
        impl EqPy for #name {
            fn eq_py(&self, other: &Self, py: Python) -> bool {
                use self::#name::*;
                match (self, other) {
                    #(#variants,)*
                    _ => false
                }
            }
        }
    };

    expanded
}

fn eqpy_impl_struct(ast: &syn::DeriveInput, en: &syn::DataStruct) -> TokenStream2 {
    let mut expression: syn::Expr = parse_quote!(true);

    // let fields: Vec<_> = match &en.fields {
    //     syn::Fields::Named(n) => n.named.iter().map(|field| field.ident.as_ref().unwrap()).collect(),
    //     _ => unreachable!(),
    // };

    if let syn::Fields::Named(n) = &en.fields {
        for field in n.named.iter() {
            let name = field.ident.as_ref().unwrap();
            let condition = parse_quote!(self.#name.eq_py(&other.#name, py));
            expression = syn::Expr::from(syn::ExprBinary {
                attrs: Vec::new(),
                left: Box::new(expression),
                op: syn::BinOp::And(<syn::Token![&&]>::default()),
                right: Box::new(condition),
            })
        }
    } else {
        unreachable!()
    }

    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl EqPy for #name {
            fn eq_py(&self, other: &Self, py: Python) -> bool {
                #expression
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
        output.extend(intopyobject_impl_enum(&ast, &e));
        output.extend(frompyobject_impl_enum(&ast, &e));
        output.extend(aspyptr_impl_enum(&ast, &e));
        output.extend(intopy_impl_enum(&ast, &e));
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
        unsafe impl pyo3::AsPyPointer for #name {
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

fn intopyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream2 {
    let mut variants = Vec::new();

    // extract name of base struct
    let meta = &ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(&syn::Ident::new("wraps", attr.span())))
        .expect("could not find #[wraps] attribute")
        .meta;
    let base: syn::Ident = match meta {
        syn::Meta::List(l) => syn::parse2(l.tokens.clone()).unwrap(),
        _ => panic!("#[wraps] argument must be a class ident"),
    };

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => x.extract(py)));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl<'py> pyo3::IntoPyObject<'py> for &#name {
            type Error = PyErr;
            type Target = #base;
            type Output = Bound<'py, Self::Target>;
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                use self::#name::*;
                match self {
                    #(#variants,)*
                }
            }
        }

        #[automatically_derived]
        impl<'py> pyo3::IntoPyObject<'py> for #name {
            type Error = PyErr;
            type Target = #base;
            type Output = Bound<'py, Self::Target>;
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (&self).into_pyobject(py)
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

    // extract name of base struct
    let meta = &ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(&syn::Ident::new("wraps", attr.span())))
        .expect("could not find #[wraps] attribute")
        .meta;
    let base: syn::Ident = match meta {
        syn::Meta::List(l) => syn::parse2(l.tokens.clone()).unwrap(),
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
            fn extract_bound(ob: &Bound<'source, pyo3::types::PyAny>) -> pyo3::PyResult<Self> {
                use pyo3::AsPyPointer;

                let qualname = ob.get_type().name()?;
                let q = qualname.to_str()?;

                let ty = match q.rfind('.') {
                    Some(idx) => &q[idx+1..],
                    None => &q,
                };

                if ob.is_instance_of::<#base>() {
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
            #name(x) => (&x.bind(py).borrow()).clone_py(py).into_py(py)
        ));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl IntoPy<fastobo::ast::#name> for &#name {
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
pub fn listlike(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut field: Option<syn::Ident> = None;
    let mut ty: Option<syn::Type> = None;
    let listlike_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("field") {
            let name: syn::LitStr = meta.value()?.parse()?;
            field = Some(syn::parse_str(&name.value())?);
            Ok(())
        } else if meta.path.is_ident("type") {
            let name: syn::LitStr = meta.value()?.parse()?;
            ty = Some(syn::parse_str(&name.value())?);
            Ok(())
        } else {
            Err(meta.error("unsupported listlike property"))
        }
    });
    parse_macro_input!(args with listlike_parser);

    // add additional methods to the impl block
    let ast = parse_macro_input!(input as syn::ItemImpl);
    TokenStream::from(listlike_impl_methods(&field.unwrap(), &ty.unwrap(), ast))
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
        fn append<'py>(&mut self, object: &Bound<'py, PyAny>) -> PyResult<()> {
            let item = <#ty as pyo3::prelude::FromPyObject>::extract_bound(object)?;
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
            Python::with_gil(|py| {
                let copy = self.clone_py(py);
                Py::new(py, copy)
            })
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
        fn count<'py>(&mut self, value: &Bound<'py, PyAny>) -> PyResult<usize> {
            let py = value.py();
            let item = <#ty as pyo3::prelude::FromPyObject>::extract_bound(value)?;
            Ok(self.#field.iter().filter(|&x| x.eq_py(&item, py)).count())
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
        fn insert<'py>(&mut self, mut index: isize, object: &Bound<'py, PyAny>) -> PyResult<()> {
            let item = <#ty as pyo3::prelude::FromPyObject>::extract_bound(object)?;
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
        #[pyo3(text_signature = "(self, index=-1)", signature=(index=-1))]
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
    // Get the `base` attribute.
    let meta = &ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(&syn::Ident::new("base", attr.span())))
        .expect("could not find #[base] attribute")
        .meta;
    let base: syn::Type = match meta {
        syn::Meta::List(l) => syn::parse2(l.tokens.clone()).unwrap(),
        _ => panic!("#[base] argument must be a class ident"),
    };

    // Get the name of the wrapped struct.
    let name = &ast.ident;

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
    // Get the `base` attribute.
    let meta = &ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(&syn::Ident::new("base", attr.span())))
        .expect("could not find #[base] attribute")
        .meta;
    let base: syn::Type = match meta {
        syn::Meta::List(l) => syn::parse2(l.tokens.clone()).unwrap(),
        _ => panic!("#[base] argument must be a class ident"),
    };

    // Get the name of the wrapped struct.
    let name = &ast.ident;

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
