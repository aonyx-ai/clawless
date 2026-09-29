use clawless::prelude::*;

/// Arguments for the `check` command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Args)]
pub struct CheckArgs {
    /// Version to check, such as 1.4.0
    version: String,
}

/// Check that a version is a stable release
///
/// A stable release has a major version of 1 or more. The command exits with one of these codes:
///
/// - 0 when the version is a stable release
/// - 1 when the version cannot be read
/// - 3 when the version is not a stable release
#[command]
// A command's doc comment is its `--help` text, so an `# Errors` section would render as a
// raw Markdown heading in the terminal rather than documenting an API.
#[allow(clippy::missing_errors_doc)]
pub async fn check(args: CheckArgs, context: Context) -> CommandResult<Exit> {
    let CheckArgs { version } = args;

    message!("Checking version {version}.");

    let major = major_version(&version).with_context(|| format!("read the version {version}"))?;

    if major == 0 {
        let text = format!(
            "error: {version} is not a stable release\n  \
             help: a stable release has a major version of 1 or more\n"
        );

        return Ok(Exit::builder().code(ExitCode::from(3)).text(text).build());
    }

    message!("Version {version} is a stable release.");

    Ok(Exit::from(()))
}

/// Returns the major version of a version with three numeric parts
///
/// # Errors
///
/// Returns an error if a part is not a number, or if the version does not have three parts.
fn major_version(version: &str) -> Result<u64, Error> {
    let numbers = version
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .with_context(|| format!("read the part {part}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let [major, _minor, _patch] = numbers.as_slice() else {
        return Err(Error::msg("a version has three parts, such as 1.4.0"));
    };

    Ok(*major)
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every test
    // would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn major_version_with_a_part_that_is_not_a_number_returns_error() {
        let version = "1.x.0";

        let error = major_version(version).expect_err("should fail");

        assert_eq!(error.to_string(), "read the part x");
    }

    #[test]
    fn major_version_with_four_parts_returns_error() {
        let version = "1.4.0.2";

        let error = major_version(version).expect_err("should fail");

        assert_eq!(
            error.to_string(),
            "a version has three parts, such as 1.4.0"
        );
    }

    #[test]
    fn major_version_with_three_numbers_returns_the_major_version() {
        let version = "12.4.0";

        let major = major_version(version).expect("should succeed");

        assert_eq!(major, 12);
    }

    #[test]
    fn major_version_with_two_parts_returns_error() {
        let version = "1.4";

        let error = major_version(version).expect_err("should fail");

        assert_eq!(
            error.to_string(),
            "a version has three parts, such as 1.4.0"
        );
    }
}
