# Trieve Website

This is the source code for the Trieve website. It is built using [Astro](https://astro.build/), [Tailwind CSS](https://tailwindcss.com/), [TypeScript](https://www.typescriptlang.org/) and [Keystatic CMS](https://keystatics.com/).

## 🚀 Project Structure

Inside of the project, you'll see the following folders and files:

```text
/
├── public/ - Static files
├── src/
│   ├── assets/ - Images, fonts, etc. (also assets uploaded through Keystatic)
│   ├── components/ - Reusable Astro components
│   ├── content/ - Markdown files for content (managed by Keystatic)
│   ├── layouts/ - Layout Astro components
│   ├── lib/ - Utility functions (e.g. fetching data)
│   │   └── keystatic/ - Keystatic API client, collection types and definitions
│   ├── pages/ - Astro pages
│   ├── styles/ - Global styles
│   └── content.config.ts - Configuration for Astro content components
├── astro.config.mjs - Astro configuration
├── keystatic.config.ts - Keystatic configuration
└── package.json
```

To learn more about the folder structure of an Astro project, refer to [our guide on project structure](https://docs.astro.build/en/basics/project-structure/).

## 🧞 Commands

All commands are run from the root of the project, from a terminal:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help` | Get help using the Astro CLI                     |

## 👀 Want to learn more?

Feel free to check [our documentation](https://docs.astro.build) or jump into our [Discord server](https://astro.build/chat).
