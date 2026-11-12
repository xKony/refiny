# ADR-0004: Clipboard round-trip for selection capture and replacement

- Status: Accepted
- Date: 2026-11-12
- Deciders: xKony

## Context

Refiny must read the current selection in an arbitrary foreground application and later replace it. Three approaches were considered:

1. **Accessibility APIs** (UI Automation on Windows, AXUIElement on macOS, AT-SPI on Linux). They expose text directly and avoid clipboard churn, but coverage is inconsistent across browsers, Electron apps, terminals, and custom controls, and the implementation is large and platform-specific.
2. **Synthetic Unicode keystrokes.** This avoids the clipboard, but is slow, layout- and IME-dependent, and unreliable for long text.
3. **Clipboard round-trip.** Snapshot the clipboard, send the platform copy shortcut, read the text, restore the snapshot. Replacement reverses the flow with paste.

## Decision

Use the clipboard round-trip for the MVP on all platforms, isolated in `src-tauri/src/selection.rs`. Preserve and restore the previous clipboard contents after every action.

## Consequences

- Works almost everywhere the standard copy/paste shortcuts work.
- Clipboard managers may observe the temporary text; this is disclosed in the privacy section.
- Apps with custom copy behavior or clipboard blockers may fail; the user sees a clear error.
- Timing needs tuning per platform (poll interval, paste delay, RDP latency).
- Accessibility APIs remain a future upgrade for apps where the clipboard path is unreliable.
