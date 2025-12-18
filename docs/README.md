# Documentation

The public website and documentation for Clawless is built using [Docusaurus],
following the principles of the [Diátaxis] framework.

## Development

Working on the site is quite straightforward. [Docusaurus] provides great
documentation on its different features and content types, which can be found
here: <https://docusaurus.io/docs/category/guides>.

The necessary tooling is included in the [Flox] environment for this project.
Simply run `flox activate` to enter the environment, and then use `just` to
run the development server:

```sh
# From the project root
just docs dev

# Or, from the docs/ directory
just dev
```

[diátaxis]: https://diataxis.fr/
[docusaurus]: https://docusaurus.io/
[flox]: https://flox.dev/
