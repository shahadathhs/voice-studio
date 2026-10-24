# site: guide for AI coding agents

Next.js 16 static marketing site for Voice Studio, deployed on Vercel. Inherits the root `AGENTS.md`
and `docs/guidelines.md`. Details for people: `docs/site.md`.

- TypeScript 7, Tailwind 4, React 19. Server components only; every page renders with JavaScript off.
- Colours come from the CSS variables in `src/app/globals.css`, which mirror `desktop/src/styles/tokens.css`.
  Change them in the same commit as the app's tokens. The site follows the system theme with CSS only.
- One exported component per file, kebab-case file names. Data lives in `src/lib/`, not in JSX.
- One filled primary button per screen; other actions are quiet links.
- Screenshots are copied from `docs/screenshots` by `scripts/copy-screenshots.mjs` into the gitignored
  `public/screenshots`. Never commit copies.
- Claims on the site must match the product (`docs/product/features.md`, `docs/product/roadmap.md`). Do not
  advertise anything the roadmap marks as not built.
- The Download page reads the latest GitHub release at build time and revalidates hourly.
- Same lint limits as the rest of the repo (oxlint at the root, Prettier). Run `make site-build` before finishing.
