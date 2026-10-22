import { mount } from "svelte";

import "./styles/tokens.css";
import "./styles/base.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("Missing #app mount point in index.html");
}

mount(App, { target });
