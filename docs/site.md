# Website

The `site/` folder is the public website: a static [Next.js](https://nextjs.org) 16 app (React 19,
Tailwind 4, TypeScript 7) with no client-side JavaScript of its own.

| Page        | Source                      | Content                                     |
| ----------- | --------------------------- | ------------------------------------------- |
| `/`         | `src/app/page.tsx`          | Pitch, a real screenshot, features, privacy |
| `/features` | `src/app/features/page.tsx` | Screenshots of each screen                  |
| `/cli`      | `src/app/cli/page.tsx`      | `vtt` examples, link to the CLI reference   |
| `/privacy`  | `src/app/privacy/page.tsx`  | What stays on your computer                 |
| `/download` | `src/app/download/page.tsx` | Installers from the latest GitHub release   |

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
- **Colours** mirror `desktop/src/styles/tokens.css` in `site/src/app/globals.css`.
- **Downloads** come from the GitHub releases API (`src/lib/releases.ts`), revalidated hourly. Before the
  first release the page says so and links to the releases list.
- **Copy** must match `docs/product/features.md` and the roadmap.

## Deploy (Vercel)

Import the repository in Vercel and set the **root directory** to `site`. The framework (Next.js) is
detected. Set the production domain, then update `SITE_URL` in `src/lib/links.ts`. Vercel rebuilds on every push
to `main`; to refresh the Download page right after a release, add a deploy hook and call it from the release job.
