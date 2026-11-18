# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

A Tauri desktop application whose two windows are static HTML, CSS, and JavaScript rendered in the system webview. Not a website, but the web is the design medium.

## Users

Primary user today is the author: a developer who writes prose in other applications all day (mail, issue trackers, chat, editors) and who works in more than one natural language. Refiny is used mid-sentence, from inside someone else's application, with the hands already on the keyboard.

Secondary users arrive shortly after: technically comfortable people setting up their own instance, with no onboarding and no support channel. They must succeed from the interface alone.

## Product Purpose

Select text anywhere, press a hotkey, and get it back with grammar, punctuation, and capitalization fixed, without leaving the application you were writing in. Success is that the correction lands and the user keeps typing, never having visited a settings screen.

## Positioning

The rewriting happens in place, in the application that owns the text, so the tool never becomes a place you go. Two things make that trustworthy rather than reckless: a preservation list keeps names, technical terms, and each language span byte-identical, and the provider is any OpenAI-compatible endpoint, so a local model keeps the text on the machine.

## Operating Context

- Invoked by global hotkey over a foreign application. The user's attention is on their document, not on Refiny.
- The wait is real: a model call takes seconds, so a waiting state is on screen during a task the user cares about.
- A window that takes focus destroys the selection it was meant to act on. Refiny's surfaces must never steal focus.
- Configured rarely, in one sitting: provider, key, hotkey, prompt. Then left alone for months.
- The tray is the resting state. The settings window is opened rarely and closed quickly.
- Mixed-language text and technical vocabulary are normal input, not edge cases.
- Undo is one keystroke away in the host application, which is the real safety net.

## Capabilities and Constraints

- Two windows: settings (four tabs: Provider, Hotkey, Prompts, Playground) and a transparent always-on-top overlay. A tray icon holds the app alive.
- Four states matter: no key configured, first run, polishing, and failure. Failure is common enough to design for, not exceptional.
- The stack is static HTML, CSS, and JavaScript with no bundler, embedded at build time. No framework, no component library, no Node toolchain at runtime.
- A restrictive content security policy: same-origin scripts and styles, inline styles permitted, no remote fonts, no remote images, no external CDNs.
- The overlay must render on a transparent, undecorated, non-activating window.
- Light and dark themes follow the OS, each held to WCAG AA contrast. Fonts are self-hosted.
- Publishing state: pre-alpha, version 0.0.1, no releases, no installers.

## Brand Commitments

- The name Refiny. It stays.
- A pixel-art mascot as the waiting state, replacing the current low-poly dog. The reference the user supplied is a style reference the user does not have rights to: its palette, pose, and character must not be copied. What carries over is the idea of a small pixel creature.
- The waiting state is anchored at the mouse cursor, minimal, pill-shaped, and animated. It is a status cue, not a dialog.

## Evidence on Hand

- No users, no testimonials, no benchmarks, no case studies, no pricing, no press. None may be fabricated.
- No marketing screenshots exist. The images under the e2e directory are gitignored scratch captures from manual test runs, not assets.
- No logo exists beyond a generated letter mark in a circle.
- The app itself is the only demonstration. Screenshots of the real interface in a real state are the only honest imagery available.

## Product Principles

1. The tool disappears. Every surface earns its interruption or it does not ship. The settings window is a place to finish a task and leave.
2. Never steal focus, never touch the clipboard without putting it back, never claim a success that did not happen.
3. Configuration is a one-sitting task. Prefer a decision already made over an empty field, and make the recovery path visible when a save is refused.
4. Density over decoration in the app; one moment of character, not many. The pixel mascot is that moment, which is why nothing else competes with it.
5. State is always legible: what is configured, what is missing, what is running, and what failed. A calm interface that hides a broken key is worse than a blunt one.

## Accessibility & Inclusion

Keyboard-first by nature: every action is reachable without a pointer, and the hotkey is the primary interface. Text must hold contrast in both light and dark themes. Motion in the waiting state and the cursor pill must respect a reduced-motion preference.
