# Website

The `site/` folder is the public website: a static [Next.js](https://nextjs.org) 16 app (React 19,
Tailwind 4, TypeScript 7) with no client-side JavaScript of its own.

| Page     | Source                   | Content                                                                         |
| -------- | ------------------------ | ------------------------------------------------------------------------------- |
| `/`      | `src/app/page.tsx`       | Hero, one section per screen, privacy, command line, get it (numbered sections) |
| `/build` | `src/app/build/page.tsx` | Build from source: prerequisites, commands, where the app lands                 |

The per-screen copy lives in `src/lib/screens.ts`, the command-line examples in `src/lib/cli-examples.ts`. Both must
match the README and `docs/product/features.md`.

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
- **Motion** is CSS only: sections fade up once on scroll (`animation-timeline: view()`), hovers are 150 ms, and everything is off under reduced motion.
- **Build instructions** on `/build` come from `src/lib/build-steps.ts` and must match `docs/building.md`.
- **Copy** must match `docs/product/features.md` and the roadmap.

## Deploy (Vercel)

Import the repository in Vercel and set the **root directory** to `site`. The framework (Next.js) is
detected. Set the production domain, then update `SITE_URL` in `src/lib/links.ts`. Vercel rebuilds on every push
to `main`.
