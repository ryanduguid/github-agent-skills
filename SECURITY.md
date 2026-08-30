# Security policy

Please use GitHub's private vulnerability reporting for this repository when
it is available. If it is not available, open a minimal public issue asking
for a private reporting channel; do not include vulnerability details there.

In a private report, describe the affected skill or script, the impact, and
safe reproduction steps. Do not include credentials, tokens, personal data,
or a full private repository. A small sanitised example is preferred when it
is needed to explain the issue.

The public-file scanner rejects assignment-shaped credential examples,
including quoted JSON-, TOML-, and YAML-like keys. Policy prose may name keys
such as `api_key` or `token` when it does not assign them a value. Split
synthetic regression fixtures in test source so the tracked test itself never
contains a credential-shaped assignment.
