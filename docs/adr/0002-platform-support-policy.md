# ADR-0002: Platform support policy

- Status: Accepted
- Date: 2026-11-12
- Deciders: xKony

## Context

The core interaction — a global hotkey, synthetic copy/paste, and a clipboard round-trip — depends on OS APIs and window-system policy. Claiming "cross-platform" without qualification would be dishonest: Wayland compositors restrict global hotkeys and synthetic input, and macOS requires an explicit Accessibility grant.

## Decision

Windows and macOS are first-class targets. Linux is best-effort: X11 is supported, and Wayland is documented as limited. Platform-specific code stays behind the `selection` boundary and Tauri plugins; no OS assumptions leak into `refiny-core`.

## Consequences

- CI builds and tests on all three platforms, with manual smoke tests on each.
- macOS distribution requires signing, notarization, and an Accessibility permission prompt.
- Windows cannot inject input into elevated windows because of UIPI.
- Linux packaging must account for WebKitGTK and libappindicator.
- Wayland users may need XWayland or a future manual "copy, polish, paste" fallback.
- Documentation and issue templates must state the platform matrix explicitly.
