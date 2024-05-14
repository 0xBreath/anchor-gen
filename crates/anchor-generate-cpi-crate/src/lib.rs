//! Generates a crate for cross-program invocations to an Anchor program from a JSON IDL.
//!
//! # Usage
//!
//! In a new crate, write:
//!
//! ```skip
//! anchor_gen::generate_cpi_crate!("../../examples/govern-cpi/idl.json");
//!
//! declare_id!("GjphYQcbP1m3FuDyCTUJf2mUMxKPE3j6feWU1rxvC7Ps");
//! ```
//!
//! This will generate a fully functional Rust CPI client for your IDL.
//!
//! More examples can be found in the [examples/](https://github.com/saber-hq/anchor-gen/tree/master/examples) directory.

use quote::quote;
use anchor_idl::GeneratorOptions;
use syn::{parse_macro_input, LitStr};

/// Generates an Anchor CPI crate from a JSON file.
///
/// # Arguments
///
/// * `input` - Path to a JSON IDL relative to the crate's the Cargo.toml.
///
/// # Examples
///
/// ```
/// anchor_generate_cpi_crate::generate_cpi_crate!("../../examples/govern-cpi/idl.json");
/// declare_id!("GjphYQcbP1m3FuDyCTUJf2mUMxKPE3j6feWU1rxvC7Ps");
/// # fn main() -> Result<()> {
/// let _my_governor = GovernanceParameters {
///     quorum_votes: 0,
///     timelock_delay_seconds: 0,
///     voting_period: 0,
///     voting_delay: 0,
/// };
/// #   Ok(())
/// # }
/// ```
#[proc_macro]
pub fn generate_cpi_crate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let id_literal = parse_macro_input!(input as LitStr);
    let opts = GeneratorOptions {
        idl_path: id_literal.value(),
        ..Default::default()
    };

    let gen = opts.to_generator();
    let mut ts: proc_macro::TokenStream = gen.generate_cpi_interface().into();

    let acct_idents = gen.account_idents();
    let acct_variants = acct_idents.into_iter().map(|ident| {
        let variant_name = ident.clone();
        quote! { #variant_name(#ident) }
    });
    let account_ts: proc_macro::TokenStream = quote! {
        anchor_gen::decode_account!(
            pub enum AccountType {
                #(#acct_variants,)*
            }
        );
    }.into();
    ts.extend(account_ts);

    let ix_idents = gen.instruction_idents();
    let ix_variants = ix_idents.into_iter().map(|ident| {
        let variant_name = ident.clone();

        // Construct the path prefix
        let path_prefix: syn::Path = syn::parse_str("instruction").unwrap();

        // Create a new PathSegment with the input Ident
        let mut segments = path_prefix.segments.clone();
        segments.push(syn::PathSegment::from(ident));

        // Combine the path prefix and the Ident
        let full_path = syn::Path {
            leading_colon: path_prefix.leading_colon,
            segments,
        };
        
        quote! { #variant_name(#full_path) }
    });
    let ix_ts: proc_macro::TokenStream = quote! {
        anchor_gen::decode_instruction!(
            pub enum InstructionType {
                #(#ix_variants,)*
            }
        );
    }.into();
    ts.extend(ix_ts);

    ts
}