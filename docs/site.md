# Website

The `site/` folder is the public website: a static [Next.js](https://nextjs.org) 16 app (React 19,
Tailwind 4, TypeScript 7) with no client-side JavaScript of its own.

| Page               | Source                             | Content                                                            |
| ------------------ | ---------------------------------- | ------------------------------------------------------------------ |
| `/`                | `src/app/page.tsx`                 | Hero, a card for every feature, privacy, command line, build       |
| `/features`        | `src/app/features/page.tsx`        | All seven screens as cards                                         |
| `/features/[slug]` | `src/app/features/[slug]/page.tsx` | One page per screen: what it does, the CLI, honest limits, related |
| `/privacy`         | `src/app/privacy/page.tsx`         | What stays on your computer                                        |
| `/cli`             | `src/app/cli/page.tsx`             | `vtt` examples by task, and exit codes                             |
| `/build`           | `src/app/build/page.tsx`           | Build from source: prerequisites, commands, where the app lands    |

The docs section (`/docs`, `/docs/[slug]`, `/docs/models`, `/docs/languages`) is rendered from the repository itself:
`src/lib/docs-manifest.ts` lists the pages, and the markdown files in `docs/` and `CONTRIBUTING.md` are read at build time,
with relative links rewritten to site pages (or to GitHub for other files). Models and licences come from
`catalog/models.json`, so they cannot drift. Languages come from `src/lib/languages.ts` (keep it in step with
`docs/product/speech.md` and `docs/product/cli.md`). Code fences with a language are syntax highlighted. To add a doc, add a line to the manifest. Vercel must have
"Include source files outside of the Root Directory" enabled, because the build reads `../docs` and `../catalog`.

The per-feature copy lives in `src/lib/features.ts`, the command-line examples in `src/lib/cli-groups.ts`. Both must
match the README and `docs/product/features.md`, including the limits.

## Work on it

```bash
make site-dev     # http://localhost:3000
make site-build   # production build, includes the type check
```

`pnpm exec oxlint --type-aware .` and `pnpm exec prettier --check .` at the root cover the site too, and CI
builds it on every pull request.

## How it stays true to the product

- **Screenshots** are the README's: `make screenshots` renders `docs/screenshots`, and the site copies them
  at build time. Nothing is committed twice.
- **Colours** mirror `desktop/src/styles/tokens.css` in `site/src/app/globals.css`. The site follows the system theme and has a light/dark toggle (the one client component); both screenshot themes are on the page and CSS shows the matching one.
- **Motion** is CSS only, in `src/app/motion.css`: a reading-progress bar (`scroll()`), blocks that fade up once, staggered grid cards, and a screenshot that settles into place (`view()`). Hovers are 150 ms. Everything is off under reduced motion, and a browser without scroll-driven animation sees the finished page.
- **SEO:** each page sets its title, description, canonical URL and social cards through `pageMetadata` in `src/lib/seo.ts`. The home page carries `SoftwareApplication` structured data and feature pages carry breadcrumbs. There is a sitemap, `robots.txt`, a web manifest, a generated Open Graph image and favicons.
- **Logo:** `src/components/logo-mark.tsx` and `src/app/icon.svg` draw the same mark: a sound wave that dips toward the middle on the accent colour. `src/lib/logo-bars.ts` holds the bar heights.
- **Build instructions** on `/build` come from `src/lib/build-steps.ts` and must match `docs/building.md`.
- **Search** (header button, Ctrl or ⌘ K) reads `/search-index.json`, built at deploy time by `src/lib/search-index.ts` from the docs sections, features, FAQ and models. It is ranked by a small function in `src/lib/search-rank.ts`; there is no server and no dependency. It loads the index only when first opened.
- **Copy** must match `docs/product/features.md` and the roadmap.

## Deploy (Vercel)

Import the repository in Vercel and set the **root directory** to `site`. The framework (Next.js) is
detected. Set the production domain, then update `SITE_URL` in `src/lib/links.ts`. Vercel rebuilds on every push
to `main`.
