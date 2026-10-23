# Design

How Voice Studio should look and feel. The mockups are plain HTML/CSS rendered to PNG; they are the
reference the desktop UI is built against (`desktop/src/styles/tokens.css` uses the same colour tokens).

## Principles

- Quiet and tool-like: system fonts, one accent colour (deep green), neutral warm greys.
- Real content in every mockup (names, times, sizes), never lorem ipsum.
- No gradients, glassmorphism, emoji icons or decorative cards. Borders and spacing do the work.
- Dense but readable: 13px base text, monospace for timestamps and sizes.
- Speaker colours are four muted hues used for text and thin bars, never large fills.
- macOS window chrome is drawn in the mockups; Windows and Linux use their own native title bars.

## Screens

| #   | Screen                       | Light                                  | Dark                                        |
| --- | ---------------------------- | -------------------------------------- | ------------------------------------------- |
| 01  | Transcribe: drop zone        | [png](mockups/01-transcribe-empty.png) | [png](mockups/01-transcribe-empty-dark.png) |
| 02  | Transcribe: queue            | [png](mockups/02-transcribe-queue.png) |                                             |
| 03  | Transcript viewer            | [png](mockups/03-transcript.png)       | [png](mockups/12-transcript-dark.png)       |
| 04  | Record                       | [png](mockups/04-record.png)           | [png](mockups/04-record-dark.png)           |
| 05  | Speak (text to speech)       | [png](mockups/05-speak.png)            | [png](mockups/05-speak-dark.png)            |
| 06  | Models                       | [png](mockups/06-models.png)           | [png](mockups/06-models-dark.png)           |
| 07  | Library                      | [png](mockups/07-library.png)          |                                             |
| 08  | Settings                     | [png](mockups/08-settings.png)         | [png](mockups/08-settings-dark.png)         |
| 09  | First-run onboarding         | [png](mockups/09-onboarding.png)       |                                             |
| 10  | Tray menu and dictation pill | [png](mockups/10-tray-dictation.png)   |                                             |
| 11  | Export dialog                | [png](mockups/11-export-dialog.png)    |                                             |

Features behind each screen: [../product/features.md](../product/features.md).

## Theming (light, dark, system)

- Three modes: **System** (default, follows the OS and switches live when the OS does), **Light**, **Dark**.
- Quick switch: a compact three-stop toggle at the bottom of the sidebar on every screen (system, light, dark as circular icon stops with a sliding circular thumb; labels appear as tooltips). The same setting, with text labels, is under Settings → General → Appearance. Both change the same stored preference.
- Implementation: all colours are CSS custom properties (the tokens in `mockups/src/common.css`). Dark mode swaps the token values; no component has its own colours. System mode reads `prefers-color-scheme`.
- The accent and the four speaker colours each have a separate dark value tuned for contrast (a straight inversion fails contrast on the accent and on speaker labels).
- Native pieces follow the same mode: Tauri window theme, title bar and tray icon (a template icon on macOS so it adapts to the menu bar).
- No flash on launch: the saved preference is applied before first paint.
- Contrast target: WCAG AA for text in both themes, checked per release.
- Dark mockups: [01](mockups/01-transcribe-empty-dark.png), [04](mockups/04-record-dark.png), [05](mockups/05-speak-dark.png), [06](mockups/06-models-dark.png), [08](mockups/08-settings-dark.png), [12](mockups/12-transcript-dark.png). The other screens use the same tokens.

## Regenerating the mockups

Edit the HTML in `mockups/src/` and run `./render.sh` (needs Google Chrome; set `CHROME=/path/to/chrome`
to use another binary). Dark variants are rendered by appending `?theme=dark`.
