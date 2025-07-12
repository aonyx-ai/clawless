use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AppGenerator {}

impl AppGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn app_function(&self) -> TokenStream {
        quote! {
            #[clawless::command(noop = true, root = true)]
            pub async fn clawless() -> clawless::Result {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_function() {
        let generator = AppGenerator::new();

        let actual = generator.app_function();
        let expected = quote! {
            #[clawless::command(noop = true, root = true)]
            pub async fn clawless() -> clawless::Result {
                Ok(())
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
