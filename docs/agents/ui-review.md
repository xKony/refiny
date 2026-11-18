# UI review without carrying the pictures

The session that writes the interface does not read screenshots. It reads facts, and it delegates anything that needs eyes.

This exists because of a measured failure: the Marginalia build produced 79 review images, and a single pass over them is on the order of 100k tokens of pictures. The work was fine. Carrying the pictures is what ended sessions.

## Tier 1: facts

Facts answer most review questions and cost nothing.

| Check | Command | Needs the app |
| --- | --- | --- |
| Element ids referenced by `main.js` exist; every class has a rule; tags balance | `node .scratch/audit.cjs` | no |
| Every text token clears 4.5:1 on every surface, both themes; rules clear 3:1 | `node .scratch/contrast-tokens.cjs` | no |
| Live geometry: viewport, overflow, fonts loaded, active tab, ids | `node .e2e/capture-marginalia.mjs` | yes |

The capture script prints a JSON block before it writes any PNG. Read that block and delete the PNGs. Run it in both themes and at the narrow breakpoint when the change touches the frame.

When a fact is missing, add it to the script: the overlay's rect against the cursor, a clamp at each work-area edge, computed colour versus the token map, the tab order after a tab switch. One more field beats one more look.

## Tier 2: eyes

When the question is aesthetic - does the rail read as a margin, is the note too loud, does this feel like a proof sheet - spawn one subagent for the round:

```
subagent(
  description = "Review the settings sheet",
  prompt = "Read these three images: <abs paths>. Judge only against the rubric
            below. Return at most 8 bullet findings, each naming the element,
            the token or value involved, and what to change. No praise, no
            summary, no redesign proposals."
)
```

Rules that keep this honest:

- Three images per child, maximum. Name them in the prompt; no directories.
- Give the rubric, not your taste.
- The child returns bullets, not prose about the design.
- The child's context dies with it. Yours does not grow.
- Never open an image "just to check". A quick look costs the same tokens as a careful one, and it is the thing that ends the session.

## Where screenshots live

`.scratch/review/<surface>-<state>-<theme>.png`, for example `settings-provider-empty-light.png`. The directory is gitignored and is not an archive: delete a round once it has been reviewed. A screenshot that nobody is going to look at again is just context waiting to be spent.

Rasters under `.e2e/` and `.impeccable/` are the same kind of scratch and are also gitignored. Nothing there ships.

## The rubric to hand the child

1. **Hierarchy.** Are the heading, the lede, the controls, and the status in that order of weight? Does one element win?
2. **Grid.** Do edges share a column? Does spacing sit on the 4px rhythm?
3. **Mark.** Is lime on the active thing only, red on wrong or removable only?
4. **Contrast.** Any text that looks quiet but is unreadable? Any hover state that disappears?
5. **Dark parity.** Is every light-theme relationship preserved, or substituted away?
6. **Copy.** Does every visible sentence say what happened or what to do?
7. **Keyboard.** Is the focused thing visible, and in a sensible order?

## What no capture can check

Click-through, focus stealing, where the pill lands relative to the pointer, DPI sharpness, compositor transparency around the overlay. Those are OS behaviours with the webview inside them. They need a running app and a human hand, which is why issue #14 blocks honest verification of anything visual.

## When the user asks to see it

Present the files. Describe the state in words. Do not read them to describe them.
