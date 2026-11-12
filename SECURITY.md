# Security Policy

## Reporting a vulnerability

Please do not open a public issue for security problems. Use GitHub's private vulnerability reporting on this repository, or contact the maintainer directly through the GitHub profile linked in the repository metadata.

Include a description, reproduction steps, the affected platform, and the Refiny version.

## Scope

Refiny handles three sensitive things:

- **Selected text**, which is sent to the provider configured by the user.
- **API keys**, which are read from an environment variable named in `config.toml` and never written to disk by Refiny.
- **The clipboard**, which is temporarily used to capture and replace selections and is restored afterwards.

Security-relevant areas include prompt injection through selected text, clipboard leakage, provider URL handling, and the Tauri capability configuration.

## Supported versions

Refiny is pre-alpha. Only the latest commit on `main` is supported.
