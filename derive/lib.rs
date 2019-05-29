#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use quote::quote;

// ---

#[proc_macro_derive(ClonePy)]
pub fn clonepy_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        syn::Data::Enum(e) => clonepy_impl_enum(&ast, &e),
        syn::Data::Struct(s) => clonepy_impl_struct(&ast, &s),
        _ => panic!("#[derive(ClonePy)] only supports enum or structs"),
    }
}

fn clonepy_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
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

    TokenStream::from(expanded)
}

fn clonepy_impl_struct(ast: &syn::DeriveInput, _en: &syn::DataStruct) -> TokenStream {

    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl ClonePy for #name {
            fn clone_py(&self, _py: Python) -> Self {
                self.clone()
            }
        }
    };

    TokenStream::from(expanded)
}

// ---

#[proc_macro_derive(PyWrapper, attributes(wraps))]
pub fn pywrapper_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let mut output = TokenStream::new();

    if let syn::Data::Enum(e) = &ast.data {
        // output.extend(clone_impl_enum(&ast, &e));
        output.extend(topyobject_impl_enum(&ast, &e));
        output.extend(intopyobject_impl_enum(&ast, &e));
        output.extend(frompyobject_impl_enum(&ast, &e));
        output.extend(aspyptr_impl_enum(&ast, &e));
        output.extend(frompy_impl_enum(&ast, &e));
        // output.extend(pyobjectprotocol_impl_enum(&ast, &e))
    } else {
        panic!("only supports enums");
    }

    output
}

fn aspyptr_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
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

    TokenStream::from(expanded)
}

fn topyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
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

    TokenStream::from(expanded)
}

fn intopyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => x.into_object(py)));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::IntoPyObject for #name {
            fn into_object(self, py: Python) -> pyo3::PyObject {
                use self::#name::*;
                match self {
                    #(#variants,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn frompyobject_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
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
        let ref arg = match args {
            syn::PathArguments::AngleBracketed(ref br) => br.args.iter().next().unwrap(),
            _ => unreachable!(),
        };
        let ref path = match arg {
            syn::GenericArgument::Type(syn::Type::Path(ref path)) => path.path.clone(),
            _ => unreachable!(),
        };
        let ref lit = syn::LitStr::new(
            &path.segments.iter().next().unwrap().ident.to_string(),
            path.segments.iter().next().unwrap().ident.span(),
        );

        variants.push(quote!(
            #lit => Ok(#name(pyo3::Py::from_borrowed_ptr(ob.as_ptr())))
        ));
    }

    let meta = ast.attrs
        .iter()
        .find(|attr| attr.path.is_ident(syn::Ident::new("wraps", attr.span())))
        .expect("could not find #[wraps] attribute")
        .parse_meta()
        .expect("could not parse #[wraps] argument");

    let base = match meta {
        syn::Meta::List(l) => match l.nested.iter().next().unwrap() {
            syn::NestedMeta::Meta(syn::Meta::Word(w)) => w.clone(),
            _ => panic!("#[wraps] argument must be a class ident"),
        }
        _ => panic!("#[wraps] argument must be a class ident"),
    };

    // Build FromPyObject implementation
    let name = &ast.ident;
    let err_sub = syn::LitStr::new(
        &format!("subclassing {} is not supported", base),
        base.span()
    );
    let err_ty = syn::LitStr::new(
        &format!("expected {} instance, {{}} found", base),
        base.span()
    );
    let expanded = quote! {
        #[automatically_derived]
        impl<'source> pyo3::FromPyObject<'source> for #name {
            fn extract(ob: &'source pyo3::types::PyAny) -> pyo3::PyResult<Self> {
                use self::#name::*;
                use pyo3::AsPyPointer;

                let ty = ob.get_type().name();
                if ob.py().is_instance::<#base, _>(ob)? {
                    unsafe {
                        match ty.as_ref() {
                            #(#variants,)*
                            _ => pyo3::exceptions::TypeError::into(#err_sub)
                        }
                    }
                } else {
                    pyo3::exceptions::TypeError::into(#err_ty)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn frompy_impl_enum(ast: &syn::DeriveInput, en: &syn::DataEnum) -> TokenStream {
    let mut variants = Vec::new();

    // Build clone for each variant
    for variant in &en.variants {
        let name = &variant.ident;
        variants.push(quote!(#name(x) => Self::from_py(x.as_ref(py).deref().clone_py(py), py)));
    }

    // Build clone implementation
    let name = &ast.ident;
    let expanded = quote! {
        #[automatically_derived]
        impl pyo3::FromPy<&#name> for fastobo::ast::#name {
            fn from_py(obj: &#name, py: Python) -> Self {
                use std::ops::Deref;
                use self::#name::*;
                match obj {
                    #(#variants,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// ---

#[proc_macro_derive(PyList)]
pub fn pylist_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        syn::Data::Struct(s) => pylist_impl_struct(&ast, &s),
        _ => panic!("#[derive(PyList)] only supports structs"),
    }
}

fn pylist_impl_struct(ast: &syn::DeriveInput, st: &syn::DataStruct) -> TokenStream {


    let mut field = None;   // the name of the Vec field
    let mut elem = None;    // the type of Vec elements
    match &st.fields {
        syn::Fields::Named(n) => {
            for f in n.named.iter() {
                if let syn::Type::Path(p) = &f.ty {
                    if let Some(c) = p.path.segments.first() {
                        if c.value().ident == "Vec" {
                            if let syn::PathArguments::AngleBracketed(g) = &c.value().arguments {
                                if let Some(syn::GenericArgument::Type(t)) = g.args.first().map(|p| p.into_value()) {
                                    elem = Some(t.clone());
                                    field = Some(f.ident.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => panic!("#[derive(PyList)] only supports struct with named fields"),
    }

    let name = &ast.ident;
    let attr = field.expect("could not find a field with `Vec` type");
    let ty = elem.expect("could not find a field with `Vec` type");

    TokenStream::from(quote! {
        #[pymethods]
        impl #name {
            /// append(self, object)
            ///
            /// Append object to the end of the list.
            ///
            /// Raises:
            #[doc("")]
            fn append(&mut self, object: &PyAny) -> PyResult<()> {
                let item = #ty::extract(object)?;
                self.#attr.push(item);
                Ok(())
            }
        }
    })
}
