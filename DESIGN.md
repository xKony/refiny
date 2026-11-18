# Design system

The settings window and the overlay are static HTML, CSS, and JavaScript inside a Tauri webview, so the browser is the design medium but not the delivery target. There is no framework, no component library, and no build step for the frontend.

The world is **Marginalia**: configuration read the way a copy desk reads a marked-up proof. Chalk ground, warm graphite rules, pencil annotations in the margin, a lime highlighter on the one active thing, correction red on what is wrong. It refuses the centred-card settings template and the icon-per-tab sidebar.

Two files hold the system: `src/tokens.css` (values) and `src/styles.css` (components). The overlay adds `src/overlay.css`. If a component needs a value that is not a token, the value is wrong and the token file is where it belongs.

## Tokens

Every colour is named by role, never by appearance, so the dark theme is a substitution rather than a second stylesheet.

| Token | Light | Dark | Role |
| --- | --- | --- | --- |
| `--ink` | `#1b1b1b` | `#f2f1ec` | Primary text, filled buttons, strong rules |
| `--ink-2` | `#4a4844` | `#b8b5ad` | Lede and hint text |
| `--ink-3` | `#6b6861` | `#8b8880` | Labels, quiet metadata |
| `--ink-4` | `#736f68` | `#8a877f` | Hover borders, lowest readable text |
| `--ground` | `#fafaf8` | `#16150f` | Window ground |
| `--surface` | `#ffffff` | `#1e1d16` | Raised fields, preset boxes, rows |
| `--surface-sunk` | `#f4f3ef` | `#12110c` | Code chips, recessed areas |
| `--rule` | `#c9c6bd` | `#3a3830` | Structural rules |
| `--rule-strong` | `#8f8b82` | `#6c6960` | Field underlines, box borders |
| `--rule-soft` | `#eceae4` | `#2a2822` | Section separators |
| `--mark` | `#d8f32b` | `#cbe32c` | The highlighter |
| `--mark-ink` | `#1b1b1b` | `#16150f` | Text sitting inside the highlighter |
| `--correction` | `#c2291f` | `#f0736a` | Errors, removal, the active tab's number |
| `--correction-soft` | `#fdeceb` | `#2b1a18` | Error field ground |
| `--verified` | `#1f7a3d` | `#7fce9b` | Success status |

Shape has three radii and no others: `2px` on inline marks and chips, `4px` on fields, buttons, and boxes, `10px` on the one container that floats (the overlay card). Spacing runs on a 4px rhythm: 4, 8, 12, 16, 22, 30, 40.

Two fonts, both self-hosted with their OFL licences beside them in `src/fonts/`:

- **Bricolage Grotesque** (400-800) for prose.
- **JetBrains Mono** (400-700) for anything the user could retype or measure: URLs, model names, keys, env vars, paths, hotkeys, statuses, timestamps.

The mono is a semantic marker, not a decoration. A value in the sans is a sentence; a value in the mono is data.

## Type scale

| Role | Size / weight / tracking |
| --- | --- |
| Sheet heading `h1` | 25px / 600 / -0.03em, 21px under 660px |
| Section and note title `h2` | 14px / 600 / -0.01em |
| Body | 13px / 400 / 1.5 |
| Lede | 12.5px / 1.55, max 62ch |
| Rail tab | 14px / 500, 600 when active |
| Field and metadata (mono) | 11px to 12.5px |
| Labels and kickers | 11px mono, uppercase, 0.09-0.12em |
| Wordmark | 16px / 600 / -0.02em |

Line length caps at 62ch for the lede and 66ch for hints. Nothing is justified.

## Components

**Frame.** Three rows: masthead, body, footer. Inside the body, a 208px rail and the sheet. Only the active sheet's `.panel-body` scrolls; its `.actions` bar is outside the scroll area, so Save is always on screen. The narrow breakpoint at 660px turns the rail into a horizontal tab strip above the sheet and drops the rail's head, facts, and note.

**Masthead.** Wordmark with its period under the highlighter (the one highlighter mark outside a control, and the reason the wordmark reads as a finished sentence), the tagline, then the current model and hotkey in the mono voice, right aligned.

**Rail.** Numbered `01`-`04` marks down the left margin. The active tab puts the whole word inside a lime smear with `--mark-ink` text and rules its number in correction red: the copy desk's mark for the page being worked on. Hover lays a `--rule-soft` hem under the word instead. Below the tabs, a definition list of Key, Prompt, and Last action, then the rail note pinned to the bottom, whose bold first line is the complete state ("Ready.", "Key missing.", "Not configured yet."). The rail note and the first-run banner never show at the same time.

**Sheet.** Heading, one-sentence lede, then the content. Every panel ends in a ruled `.actions` bar with one filled primary. A `.chip` carries a short state next to the heading where the heading alone would mislead.

**First-run note.** A ruled annotation, not a card: heavy rule above, soft rule below, title under the highlighter, two actions on the right. It disappears for good once dismissed and the rail note takes over.

**Provider presets.** A radio group of boxed entries on `--surface` with a `--rule-strong` border. Name in the sans, endpoint in the mono. Selected is a ruled box with a `2px 2px` lime offset shadow: the highlighter's second pass, not elevation. Hover deepens the border to `--ink-4`. Arrow keys move and select, Home and End jump, and only the selected entry is in the tab order.

**Fields.** A label in uppercase mono over an underlined input: no box, no fill, one `--rule-strong` underline that firms to `--ink` on hover. Focus draws a second and third lime hairline under the rule, so focus reads as a pencil pass under the line rather than a browser ring. Textareas are the exception: they are boxed on `--surface` at `4px` radius, with an ink ring and a lime halo on focus. Placeholders are set in the sans to read as guidance, while typed text is mono because it is data.

**Buttons.** One filled primary per sheet: `--ink` ground, `--ground` text. Secondary is the same ink outline on nothing, and on hover gains the same `2px 2px` lime offset as a selected preset. A press settles 1px. Disabled dims to 0.6 and switches the cursor to progress. Links are underlined text at 12px; a danger link is correction red.

**Status.** A mono line at the end of the action bar, in `--verified` for success and `--correction` for failure, always naming what happened rather than "Saved".

**Empty and error states.** Dashed `--rule-strong` boxes at 4px radius, quiet mono-free text. A missing key is never hidden: it is a red `warn` fact in the rail and, when setup is complete, a rail note.

**Native parts.** Selection is lime. Scrollbars are themed to the rules. Checkboxes take `accent-color: --correction`, so the browser's own chrome belongs to the world.

## Motion

One grammar. 120ms, `cubic-bezier(0.22, 1, 0.36, 1)`, on colour, border, transform, filter, and shadow; 200ms is reserved for surfaces that travel further. Everything moves in or out, nothing loops. `prefers-reduced-motion` removes every duration, keeping only the end state.

## Contrast

Measured by ratio, not by eye, in both themes: every text token clears 4.5:1 against every surface it lands on, and structural rules clear 3:1. The floor is `--ink-4` on `--surface-sunk` at 4.50:1, which is why that token is not used for body text. The check lives in `.scratch/contrast-tokens.cjs` and is re-run whenever a token changes; the script is scratch, the rule is not.

## Provenance

- Fonts: Bricolage Grotesque and JetBrains Mono, SIL Open Font License 1.1, licence files in `src/fonts/`.
- Screenshots under `.e2e/` and `.impeccable/` are scratch captures from review runs. They are gitignored, are not assets, and ship with nothing.
- The overlay's low-poly corgi predates this system. It is existing brand art, untouched by the settings redesign, and issue #13 replaces it with a pixel mascot. The reference image the user supplied for that mascot is a style reference only: its palette, pose, and character must not be copied.

## Rules for the next agent

1. No component carries a raw colour, radius, or duration. Add the token instead.
2. Lime only marks the active thing. Red only marks what is wrong or is being removed. Neither is decoration.
3. One filled primary per sheet. If two things need weight, one of them is a link.
4. State is stated, not implied: every save and every test names its outcome in the status line.
5. Image review does not run in the main session. See [docs/agents/ui-review.md](docs/agents/ui-review.md).
