# ADR-0003: OpenAI-compatible provider interface

- Status: Accepted
- Date: 2026-11-12
- Deciders: xKony

## Context

Refiny should work with hosted providers, self-hosted gateways, and local models without maintaining a separate integration per vendor. Users should be able to keep text on their own machine when privacy matters.

## Decision

Use a single HTTP client that speaks the OpenAI chat-completions shape: `POST {base_url}/chat/completions` with a system message and a user message. Configuration names `base_url`, `model`, and `api_key_env`. The API key is optional so local servers that do not require one still work.

## Consequences

- Works with OpenAI, OpenRouter, Ollama, LM Studio, vLLM, and compatible gateways.
- Provider-specific features (tool calling, structured output extensions, vendor headers) are unavailable by design.
- Error mapping is generic: HTTP status, provider message, or empty response.
- Tests use a local HTTP mock rather than a live provider.
- Users can switch providers without changing the prompt or preservation list.
