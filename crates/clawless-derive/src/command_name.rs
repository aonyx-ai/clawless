use syn::ext::IdentExt;
use syn::{Error, Ident, Result};

/// Name under which a leaf appears on the command line
///
/// Rust writes a function name in snake case, and a command line writes a command name in kebab
/// case. This type holds the translated name, so that the function `deploy_staging` becomes the
/// command `deploy-staging`.
///
/// The name reaches clap along two separate paths. It names the clap `Command` that parses the
/// arguments and prints the help, and it is the key that the generated dispatch code looks up
/// after parsing. Both paths read the name from this type, and the two therefore cannot drift
/// apart. A command that the help shows but that never runs is thus impossible.
///
/// [`CommandName::try_from`] is the only constructor. It rejects each identifier whose translation
/// a user could not type or read.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct CommandName {
    /// The translated name, which contains no underscore and no `r#` prefix
    name: String,
}

impl CommandName {
    /// Returns the name as the user types it on the command line
    pub(crate) fn as_str(&self) -> &str {
        &self.name
    }
}

impl TryFrom<&Ident> for CommandName {
    type Error = syn::Error;

    /// Translates the name of a function into the name of a command
    ///
    /// The translation removes the `r#` prefix of a raw identifier and writes each underscore as
    /// a hyphen.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the identifier starts or ends with an underscore, because the
    /// translation would then start or end with a hyphen. Returns [`Error`] if the identifier
    /// contains two underscores in a row, because the translation would then contain two hyphens
    /// in a row. The error points at the name of the function, so that the user sees the mistake
    /// in place.
    fn try_from(ident: &Ident) -> Result<Self> {
        let name = ident.unraw().to_string();

        if name.starts_with('_') || name.ends_with('_') {
            return Err(Error::new_spanned(
                ident,
                "command name must not start or end with an underscore\n\n\
                 = help: Clawless writes each underscore of the function name as a hyphen, and a command name must not start or end with a hyphen. Rename the function so that it starts and ends with a letter.",
            ));
        }

        if name.contains("__") {
            return Err(Error::new_spanned(
                ident,
                "command name must not contain two underscores in a row\n\n\
                 = help: Clawless writes each underscore of the function name as a hyphen, and one hyphen separates two words of a command name. Rename the function with one underscore between two words.",
            ));
        }

        Ok(Self {
            name: name.replace('_', "-"),
        })
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use proc_macro2::Span;

    use super::*;

    #[test]
    fn command_name_try_from_with_consecutive_underscores_returns_error() {
        let ident = Ident::new("deploy__staging", Span::call_site());

        let error = CommandName::try_from(&ident).expect_err("should fail");

        assert!(
            error
                .to_string()
                .contains("must not contain two underscores in a row")
        );
    }

    #[test]
    fn command_name_try_from_with_leading_underscore_returns_error() {
        let ident = Ident::new("_deploy", Span::call_site());

        let error = CommandName::try_from(&ident).expect_err("should fail");

        assert!(
            error
                .to_string()
                .contains("must not start or end with an underscore")
        );
    }

    #[test]
    fn command_name_try_from_with_multiple_words_returns_hyphenated_name() {
        let ident = Ident::new("deploy_staging_now", Span::call_site());

        let name = CommandName::try_from(&ident).expect("should succeed");

        assert_eq!("deploy-staging-now", name.as_str());
    }

    #[test]
    fn command_name_try_from_with_raw_identifier_returns_name_without_prefix() {
        let ident = Ident::new_raw("type", Span::call_site());

        let name = CommandName::try_from(&ident).expect("should succeed");

        assert_eq!("type", name.as_str());
    }

    #[test]
    fn command_name_try_from_with_single_word_returns_same_name() {
        let ident = Ident::new("deploy", Span::call_site());

        let name = CommandName::try_from(&ident).expect("should succeed");

        assert_eq!("deploy", name.as_str());
    }

    #[test]
    fn command_name_try_from_with_trailing_underscore_returns_error() {
        let ident = Ident::new("deploy_", Span::call_site());

        let error = CommandName::try_from(&ident).expect_err("should fail");

        assert!(
            error
                .to_string()
                .contains("must not start or end with an underscore")
        );
    }

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CommandName>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<CommandName>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<CommandName>();
    }
}
