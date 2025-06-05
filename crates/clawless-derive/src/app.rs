use proc_macro2::TokenStream;
use quote::quote;

use crate::app::params::AppParams;

mod params;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AppGenerator {
    params: AppParams,
}

impl AppGenerator {
    pub fn new(input: TokenStream) -> Self {
        let params = syn::parse2::<AppParams>(input).unwrap();

        Self { params }
    }

    pub fn app_function(&self) -> TokenStream {
        let name = self.params.name().clone();

        quote! {
            #[clawless::command(noop = true)]
            pub async fn #name() {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_function_with_name() {
        let input = quote! { { name: "clawless" } };
        let generator = AppGenerator::new(input);

        let actual = generator.app_function();
        let expected = quote! {
            #[clawless::command(noop = true)]
            pub async fn clawless() {}
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    #[should_panic(expected = "missing required field `name`")]
    fn app_function_without_name() {
        let input = quote! { {} };
        let _generator = AppGenerator::new(input);
    }
}
