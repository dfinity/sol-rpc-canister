//! Crate for managing canister canlog

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

use darling::FromVariant;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

/// A procedural macro to implement [`LogPriorityLevels`](canlog::LogPriorityLevels) for an enum.
///
/// This macro expects the variants to be annotated with `#[log_level(capacity = N, name = "NAME")]`
/// where `N` is an integer representing buffer capacity and `"NAME"` is a string display 
/// representation for the corresponding log level.
/// 
/// The enum annotated with `#[derive(LogPriorityLevels)]` must also implement the 
/// [`Serialize`](serde::Serialize), [`Deserialize`](serde::Deserialize), 
/// [`Clone`](core::clone::Clone) and [`Copy`](core::marker::Copy) traits
///
/// **Generated Code:**
/// 1. Declares a [`GlobalBuffer`](ic_canister_log::GlobalBuffer) and 
///     [`Sink`](ic_canister_log::Sink) constant for each variant.
/// 2. Implements the [LogPriorityLevels](canlog::LogPriorityLevels) trait for the enum.
///
/// **Usage Example:**
/// ```rust
/// use canlog::{GetLogFilter, LogFilter, LogPriorityLevels, log};
/// use canlog_derive::LogPriorityLevels;
/// 
/// #[derive(LogPriorityLevels)]
/// enum LogPriority {
///     #[log_level(capacity = 100, name = "INFO")]
///     Info,
///     #[log_level(capacity = 500, name = "DEBUG")]
///     Debug,
/// }
///
/// impl GetLogFilter for LogPriority {
///     fn get_log_filter() -> LogFilter {
///         LogFilter::ShowAll
///     }
/// }
///
/// fn main() {
///     log!(LogPriority::Info, "Some rather important message.");
///     log!(LogPriority::Debug, "Some less important message.");
/// }
/// ```
///
/// **Expected Output:**
/// ```text
/// 2025-02-26 08:27:10 UTC: [Canister lxzze-o7777-77777-aaaaa-cai] INFO main.rs:13 Some rather important message.
/// 2025-02-26 08:27:10 UTC: [Canister lxzze-o7777-77777-aaaaa-cai] DEBUG main.rs:14 Some less important message.
/// ```
#[proc_macro_derive(LogPriorityLevels, attributes(log_level))]
pub fn derive_log_priority(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("This trait can only be derived for enums");
    };

    // Declare a buffer and sink for each enum variant
    let buffer_declarations = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let info = LogLevelInfo::from_variant(variant)
            .expect(format!("Invalid attributes for log level: {}", variant_ident).as_str());

        let buffer_ident = get_buffer_ident(variant_ident);
        let sink_ident = get_sink_ident(variant_ident);
        let capacity = info.capacity;

        quote! {
            canlog::declare_log_buffer!(name = #buffer_ident, capacity = #capacity);
            pub const #sink_ident: canlog::PrintProxySink<#enum_ident> = canlog::PrintProxySink(&#enum_ident::#variant_ident, &#buffer_ident);
        }
    });

    // Match arms to get the corresponding buffer, sink and display name for each enum variant
    let buffer_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let buffer_ident = get_buffer_ident(variant_ident);
        quote! {
            Self::#variant_ident => &#buffer_ident,
        }
    });
    let sink_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let sink_ident = get_sink_ident(variant_ident);
        quote! {
            Self::#variant_ident => &#sink_ident,
        }
    });
    let display_name_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let display_name = LogLevelInfo::from_variant(variant).unwrap().name;
        quote! {
            Self::#variant_ident => #display_name,
        }
    });
    let variants_array = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        quote! { Self::#variant_ident, }
    });

    // Generate buffer declarations and trait implementation
    let trait_impl = quote! {
        #(#buffer_declarations)*

        impl canlog::LogPriorityLevels for #enum_ident {
            fn get_buffer(&self) -> &'static canlog::GlobalBuffer {
                match self {
                    #(#buffer_match_arms)*
                }
            }

            fn get_sink(&self) -> &impl canlog::Sink {
                match self {
                    #(#sink_match_arms)*
                }
            }

            fn display_name(&self) -> &'static str {
                match self {
                    #(#display_name_match_arms)*
                }
            }

            fn get_priorities() -> &'static [Self] {
                &[#(#variants_array)*]
            }
        }
    };

    trait_impl.into()
}

#[derive(FromVariant)]
#[darling(attributes(log_level))]
struct LogLevelInfo {
    capacity: usize,
    name: String,
}

fn get_sink_ident(variant_ident: &Ident) -> Ident {
    quote::format_ident!("{}", variant_ident.to_string().to_uppercase())
}

fn get_buffer_ident(variant_ident: &Ident) -> Ident {
    quote::format_ident!("{}_BUF", variant_ident.to_string().to_uppercase())
}
