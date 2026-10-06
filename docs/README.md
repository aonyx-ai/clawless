# Documentation

The public website and documentation for Clawless is built using [Docusaurus],
following the principles of the [Diátaxis] framework.

## Development

Working on the site is quite straightforward. [Docusaurus] provides great
documentation on its different features and content types, which can be found
here: <https://docusaurus.io/docs/category/guides>.

[Mise] provisions the version of Node.js that the site needs. Run
`mise install` once from the project root, and then start the development
server from the `docs/` directory:

```sh
npm ci
npm run start
```

`npm run build` creates the static build of the site that CI deploys.

[diátaxis]: https://diataxis.fr/
[docusaurus]: https://docusaurus.io/
[mise]: https://mise.jdx.dev/
