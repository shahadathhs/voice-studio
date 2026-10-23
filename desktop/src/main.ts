import { mount } from "svelte";

import "./styles/tokens.css";
import "./styles/base.css";
import App from "./App.svelte";
import DictationPill from "./DictationPill.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("Missing #app mount point in index.html");
}

// The floating dictation window loads the same page with this marker.
mount(window.location.hash === "#dictation" ? DictationPill : App, { target });
