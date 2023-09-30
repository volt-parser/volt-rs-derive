extern crate proc_macro;

use quote::TokenStreamExt;

#[proc_macro_derive(VoltModuleDefinition)]
pub fn module_fn_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_module_fn_macro(&ast)
}

fn impl_module_fn_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let item_name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(s) => {
            match &s.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!("Struct must have named fields (like `struct Sample {{}}`)."),
            }
        },
        _ => panic!("Trait `RuleContainer` is only available for struct."),
    };

    let self_impl = generate_self_impl(item_name, fields);
    let module_assist_impl = generate_module_assist_impl(item_name, fields);

    let gen = quote::quote!{
        #self_impl
        #module_assist_impl
    };

    gen.into()
}

fn generate_self_impl(item_name: &proc_macro2::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let elem_fns = fields.iter().map(|f| {
        let rule_name = match &f.ident {
            Some(v) => v,
            None => unreachable!(),
        };

        let rule_id = format!("{}::{}", item_name, rule_name);

        quote::quote!{
            pub fn #rule_name() -> volt::element::Element {
                volt::element::Element::Expression(volt::element::Expression::Rule(volt::rule::RuleId(#rule_id.to_string())))
            }
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    let mut joined_elem_fns: proc_macro2::TokenStream = quote::quote!{};
    joined_elem_fns.append_all(elem_fns);

    quote::quote!{
        impl #item_name {
            #joined_elem_fns
        }
    }
}

fn generate_module_assist_impl(item_name: &proc_macro2::Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let pushes = fields.iter().map(|f| {
        let rule_name = match &f.ident {
            Some(v) => v,
            None => unreachable!(),
        };

        let rule_id = format!("{}::{}", item_name, rule_name);

        quote::quote!{
            rules.push(volt::rule::Rule::new(volt::rule::RuleId(#rule_id.to_string()), self.#rule_name.clone()).detect_left_recursion());
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    let mut joined_pushes = quote::quote!{};
    joined_pushes.append_all(pushes);

    quote::quote!{
        impl volt::VoltModuleAssist for #item_name {
            fn into_rule_vec(self) -> volt::rule::RuleVec {
                let mut rules = Vec::new();
                #joined_pushes
                volt::rule::RuleVec(rules)
            }
        }
    }
}
