// Renders the real desktop UI with made-up sample data and saves screenshots to docs/screenshots.
// Needs Google Chrome and a built UI (make screenshots does both). The Tauri backend is replaced by a
// small stand-in (scripts/screenshots-data.mjs), so nothing here reads real audio or models.
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

import { SAMPLE_BACKEND } from "./screenshots-data.mjs";

const CHROME = process.env.CHROME ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const OUT = new URL("../docs/screenshots/", import.meta.url).pathname;
const SITE = "http://127.0.0.1:4173";
const DEBUG_PORT = 9333;

/** page: sidebar label to click, then optional text of a row to open. */
const SHOTS = [
  { name: "transcribe", page: "Transcribe" },
  { name: "transcript", page: "Library", open: "Quarterly planning meeting" },
  {
    name: "speak",
    page: "Speak",
    text: "Welcome back. Your recordings stay on this computer, and so does everything you make from them.",
  },
  { name: "models", page: "Models", height: 1000 },
];

const mockScript = (theme) => `
  localStorage.setItem("voice-studio-theme", "${theme}");
  const backend = ${JSON.stringify(SAMPLE_BACKEND)};
  let callbackId = 1;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
    transformCallback: () => callbackId++,
    unregisterCallback: () => {},
    convertFileSrc: (path) => path,
    invoke: async (command, args) => {
      if (command.startsWith("plugin:")) return 0;
      if (command in backend) return backend[command];
      return null;
    },
  };
`;

class Session {
  #socket;
  #next = 1;
  #pending = new Map();

  static async open() {
    const target = await (
      await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/new?about:blank`, { method: "PUT" })
    ).json();
    const socket = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve) => socket.addEventListener("open", resolve, { once: true }));
    const session = new Session(socket);
    await session.send("Page.enable");
    return session;
  }

  constructor(socket) {
    this.#socket = socket;
    socket.addEventListener("message", ({ data }) => {
      const message = JSON.parse(data);
      this.#pending.get(message.id)?.(message);
    });
  }

  send(method, params = {}) {
    const id = this.#next++;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, (message) =>
        message.error ? reject(new Error(message.error.message)) : resolve(message.result),
      );
      this.#socket.send(JSON.stringify({ id, method, params }));
    });
  }

  evaluate(expression) {
    return this.send("Runtime.evaluate", { expression, awaitPromise: true });
  }

  close() {
    this.#socket.close();
  }
}

const clickByText = (text) => `
  (() => {
    const candidates = [...document.querySelectorAll("button")];
    const target = candidates.find((el) => el.textContent.trim().includes(${JSON.stringify(text)}));
    if (!target) throw new Error("No button with text " + ${JSON.stringify(text)});
    target.click();
  })()`;

async function shoot(shot, theme) {
  const session = await Session.open();
  await session.send("Emulation.setDeviceMetricsOverride", {
    width: 1280,
    height: shot.height ?? 800,
    deviceScaleFactor: 2,
    mobile: false,
  });
  await session.send("Page.addScriptToEvaluateOnNewDocument", { source: mockScript(theme) });
  await session.send("Page.navigate", { url: SITE });
  await sleep(1200);
  await session.evaluate(clickByText(shot.page));
  await sleep(400);
  if (shot.open) {
    await session.evaluate(clickByText(shot.open));
    await sleep(600);
  }
  if (shot.text) {
    await session.evaluate(`(() => {
      const box = document.querySelector("textarea");
      box.value = ${JSON.stringify(shot.text)};
      box.dispatchEvent(new Event("input", { bubbles: true }));
    })()`);
    await sleep(300);
  }
  const { data } = await session.send("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(OUT, `${shot.name}-${theme}.png`), Buffer.from(data, "base64"));
  session.close();
}

mkdirSync(OUT, { recursive: true });
const server = spawn(
  "pnpm",
  [
    "--dir",
    "desktop",
    "exec",
    "vite",
    "preview",
    "--port",
    "4173",
    "--strictPort",
    "--host",
    "127.0.0.1",
  ],
  { stdio: "ignore" },
);
const chrome = spawn(
  CHROME,
  [
    "--headless=new",
    `--remote-debugging-port=${DEBUG_PORT}`,
    "--hide-scrollbars",
    `--user-data-dir=${mkdtempSync(join(tmpdir(), "vtt-shots-"))}`,
    "--no-first-run",
    "about:blank",
  ],
  { stdio: "ignore" },
);
/** Polls until the URL answers, so Chrome never loads a page before its server is up. */
async function waitFor(url) {
  for (let attempt = 0; attempt < 60; attempt++) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {
      // not up yet
    }
    await sleep(250);
  }
  throw new Error(`Timed out waiting for ${url}`);
}

try {
  await waitFor(SITE);
  await waitFor(`http://127.0.0.1:${DEBUG_PORT}/json/version`);
  for (const theme of ["light", "dark"]) {
    for (const shot of SHOTS) await shoot(shot, theme);
  }
  process.stdout.write(`Wrote ${SHOTS.length * 2} screenshots to ${OUT}` + "\n");
} finally {
  chrome.kill();
  server.kill();
}
