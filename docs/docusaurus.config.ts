import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Clawless",
  tagline: "A framework to build command-line applications in Rust",
  favicon: "img/favicon.png",

  future: {
    v4: true,
  },

  url: "https://clawless.rs",
  baseUrl: "/",

  organizationName: "aonyx-ai",
  projectName: "clawless",

  onBrokenLinks: "throw",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  markdown: {
    format: "detect",
    mermaid: true,
  },

  themes: ["@docusaurus/theme-mermaid"],

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl:
            "https://https://github.com/aonyx-ai/clawless/tree/main/docs/",
        },
        blog: {
          showReadingTime: true,
          feedOptions: {
            type: ["rss", "atom"],
            xslt: true,
          },
          onInlineTags: "warn",
          onInlineAuthors: "warn",
          onUntruncatedBlogPosts: "warn",
        },
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/social-card.png",
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: "Clawless",
      logo: {
        alt: "Clawless logo",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "tutorialSidebar",
          position: "left",
          label: "Docs",
        },
        { to: "/blog", label: "Blog", position: "left" },
        {
          href: "https://github.com/aonyx-ai/clawless",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Quick Start",
              to: "/docs/quick-start",
            },
            {
              label: "Concepts",
              to: "/docs/concepts",
            },
            {
              label: "How-to Guides",
              to: "/docs/how-to",
            },
          ],
        },
        {
          title: "Resources",
          items: [
            {
              label: "GitHub",
              href: "https://github.com/aonyx-ai/clawless",
            },
            {
              label: "crates.io",
              href: "https://crates.io/crates/clawless",
            },
            {
              label: "docs.rs",
              href: "https://docs.rs/clawless/latest/clawless/",
            },
          ],
        },
        {
          title: "More",
          items: [
            {
              label: "Blog",
              to: "/blog",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Aonyx AI`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
