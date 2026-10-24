// The README screenshots live in docs/screenshots (re-rendered by `make screenshots`). The site
// serves copies from public/screenshots, which is gitignored, so there is one source of truth.
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";

const from = fileURLToPath(new URL("../../docs/screenshots", import.meta.url));
const to = fileURLToPath(new URL("../public/screenshots", import.meta.url));

rmSync(to, { recursive: true, force: true });
mkdirSync(to, { recursive: true });
cpSync(from, to, { recursive: true });
