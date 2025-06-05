use getset::Getters;
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Colon, Comma};
use syn::{braced, Ident, LitStr};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Getters)]
pub struct AppParams {
    #[getset(get = "pub")]
    name: Ident,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Field {
    name: Ident,
    value: LitStr,
}

impl Parse for AppParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);

        let fields = Punctuated::<Field, Comma>::parse_terminated(&content)?;

        let name_field = fields
            .iter()
            .find(|field| field.name == "name")
            .ok_or_else(|| syn::Error::new(content.span(), "missing required field `name`"))?;

        let name = format_ident!("{}", name_field.value.value());

        Ok(AppParams { name })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _: Colon = input.parse()?;
        let value = input.parse()?;

        Ok(Field { name, value })
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;
    use syn::LitStr;

    use super::*;

    #[test]
    fn parse_field() {
        let input = quote! { name: "clawless" };

        let actual = syn::parse2::<Field>(input).unwrap();
        let expected = Field {
            name: format_ident!("name"),
            value: LitStr::new("clawless", Span::call_site()),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_params_with_name() {
        let input = quote! { { name: "clawless" } };

        let actual = syn::parse2::<AppParams>(input).unwrap();
        let expected = AppParams {
            name: format_ident!("clawless"),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_params_without_name() {
        let input = quote! { { } };

        let actual = syn::parse2::<AppParams>(input).unwrap_err();

        assert_eq!(actual.to_string(), "missing required field `name`");
    }
}
