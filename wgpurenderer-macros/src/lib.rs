use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_attribute]
pub fn immediate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let vis = &input.vis;
    let ident = &input.ident;
    let internal_ident = format_ident!("__Internal_{}", ident);

    let fields = if let Data::Struct(ref mut data_struct) = input.data {
        match &data_struct.fields {
            Fields::Named(fields) => fields.named.clone(),
            _ => panic!("Only named fields are supported for #[immediate]"),
        }
    } else {
        panic!("Only structs are supported for #[immediate]");
    };

    let field_accessors = fields.iter().map(|f| {
        let f_ident = &f.ident;
        let f_ty = &f.ty;
        let setter_name = format_ident!("set_{}", f_ident.as_ref().unwrap());
        let getter_name = format_ident!("get_{}", f_ident.as_ref().unwrap());

        quote! {
            #vis fn #setter_name(&mut self, value: #f_ty) {
                let off = core::mem::offset_of!(#internal_ident, #f_ident);
                self.immediate.write(self.offset + off, bytemuck::bytes_of(&value));
            }

            #vis fn #getter_name(&self) -> #f_ty {
                let off = core::mem::offset_of!(#internal_ident, #f_ident);
                let bytes = self.immediate.read::<{ core::mem::size_of::<#f_ty>() }>(self.offset + off);
                *bytemuck::from_bytes::<#f_ty>(&bytes)
            }
        }
    });

    let expanded = quote! {
        #[repr(C)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, Default)]
        struct #internal_ident {
            #fields
        }

        #[derive(Debug, Clone)]
        #vis struct #ident {
            immediate: crate::Immediate,
            offset: usize,
        }

        impl #ident {
            #vis fn new(immediate: crate::Immediate, offset: usize) -> Self {
                Self { immediate, offset }
            }

            #(#field_accessors)*
        }
    };

    TokenStream::from(expanded)
}
