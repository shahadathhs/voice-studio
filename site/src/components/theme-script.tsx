/** Runs before the first paint: applies the saved theme (otherwise the system theme applies), and marks
 *  browsers without scroll-driven animation so a small script can reveal sections instead. */
const SCRIPT = `try{var r=document.documentElement,t=localStorage.getItem("voice-studio-site-theme");if(t==="light"||t==="dark")r.dataset.theme=t;if(!(window.CSS&&CSS.supports("animation-timeline","view()"))&&!matchMedia("(prefers-reduced-motion: reduce)").matches)r.classList.add("reveal-js")}catch(e){}`;

export function ThemeScript() {
  return <script dangerouslySetInnerHTML={{ __html: SCRIPT }} />;
}
