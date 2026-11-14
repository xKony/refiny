## Agent skills

### Issue tracker

Issues live in GitHub Issues, managed with the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical triage roles, using the default label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Work tracking

Work is only real once it is written down in the repository. Any agent, starting cold in any session, must be able to find what is planned, what is in flight, and why past decisions were made, without access to a chat log.

- **Before you build**, the work exists as a GitHub issue. If it does not, create it first.
- **While you build**, record anything a future agent could not infer from the code: abandoned approaches, surprising constraints, links to the discussion. Use a comment on the issue.
- **Before you stop**, the issue says where you got to. If the work is incomplete, say what remains and what is blocking. If you finished, close it with a comment naming what shipped.
- **Anything out of scope** you notice on the way becomes a new issue, not a quiet fix and not a mention in a reply.
- **Anything hard to reverse** becomes an ADR in `docs/adr/`.
- **A term that was ambiguous** becomes an entry in `GLOSSARY.md`.

Never leave a plan, a decision, or a TODO only in a conversation.
