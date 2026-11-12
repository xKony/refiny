# ADR-0001: Rust and Tauri for the application shell

- Status: Accepted
- Date: 2026-11-12
- Deciders: xKony

## Context

Refiny needs an always-running desktop daemon with a global hotkey, clipboard access, synthetic input, a tray icon, and a small animated overlay. It must run on Windows, macOS, and Linux. Development is AI-assisted, so language-learning time is not a constraint; ecosystem fit, cross-platform behavior, and long-term maintainability are.

Go was considered first because of its standard-library HTTP/JSON support, fast builds, and gentle learning curve. Its desktop story is weaker: Wails plus cgo-based input and clipboard libraries, and a more painful Linux packaging path.

## Decision

Use Rust for the backend and Tauri 2 for the application shell, tray, windows, and webview overlay. Split pure logic into the `refiny-core` crate so it is testable without a GUI. Use Tauri's official global-shortcut plugin, and Rust crates (`arboard`, `enigo`) for clipboard and synthetic input.

## Consequences

- One codebase targets Windows, macOS, and Linux, with per-platform installers from Tauri.
- Direct, well-typed access to Win32, Cocoa, and X11 through mature crates.
- Tauri's capability model limits what the webview can access.
- Larger dependency tree and longer clean builds than a small Go program.
- The overlay depends on a system webview (WebView2, WKWebView, or WebKitGTK).
- Wayland input restrictions remain; Linux support is best-effort (see ADR-0002).
- Contributors need Rust familiarity, and AI agents need more compile iterations.
