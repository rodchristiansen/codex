# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.

## Usage-limit warnings

To hide proactive usage announcements (including the 25%, 10%, and 5% remaining
warnings) and the persistent quota banner above the input, add this to
`~/.codex/config.toml`:

```toml
[notice]
hide_rate_limit_warnings = true
```

Warnings remain enabled when this setting is omitted or `false`. Usage tracking,
`/status`, and errors when a limit blocks a request remain available. The model
switch reminder is controlled separately by `notice.hide_rate_limit_model_nudge`.
