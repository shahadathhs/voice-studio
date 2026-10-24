// Plain JS on purpose: the config stays trivial. Type checking runs in the build and separately
// via `pnpm typecheck`. NEXT_DIST_DIR lets parallel builds use separate output folders.
const nextConfig = {
  distDir: process.env.NEXT_DIST_DIR || ".next",
};

export default nextConfig;
