/** Sets the saved theme before the first paint, so there is no flash. Without it the system theme applies. */
const SCRIPT = `try{var t=localStorage.getItem("voice-studio-site-theme");if(t==="light"||t==="dark")document.documentElement.dataset.theme=t}catch(e){}`;

export function ThemeScript() {
  return <script dangerouslySetInnerHTML={{ __html: SCRIPT }} />;
}
