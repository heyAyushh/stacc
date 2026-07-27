# Explaining and configuring rules

Explain React Doctor rules and edit `doctor.config.*` safely. Use this when a user wants to understand a rule or change which rules run, not for fixing diagnostics.

Triggers: "why did this rule fire", "I disagree with this rule", "turn this rule off", "stop flagging X", "too noisy", "disable design rules".

## Workflow

1. Identify the rule key from the diagnostic, such as `react-doctor/no-array-index-as-key`.
2. Explain it before changing anything:

```bash
npx react-doctor@latest rules explain react-doctor/no-array-index-as-key
```

3. Pick the narrowest control that matches the user's intent.
4. Apply it with a `rules` subcommand, which edits `doctor.config.*` or `package.json#reactDoctor` in place.
5. Validate the change:

```bash
npx react-doctor@latest --verbose --diff
```

## Commands

```bash
npx react-doctor@latest rules list
npx react-doctor@latest rules list --configured
npx react-doctor@latest rules list --category Performance
npx react-doctor@latest rules explain <rule>
npx react-doctor@latest rules disable <rule>
npx react-doctor@latest rules enable <rule>
npx react-doctor@latest rules set <rule> warn
npx react-doctor@latest rules category "React Native" off
npx react-doctor@latest rules ignore-tag design
npx react-doctor@latest rules unignore-tag design
```

Rule references accept the full key, the bare id, or a legacy key.

## Decision guide

- User disagrees with one rule or it is a false positive for them: `rules disable <rule>`.
- Rule is fine but wrong severity: `rules set <rule> warn` or `rules set <rule> error`.
- A disabled-by-default rule should be on: `rules enable <rule>`.
- A whole area is unwanted: `rules category "<Category>" off`.
- A behavioral family is noisy: `rules ignore-tag <tag>`.
- Keep the rule locally but hide it from a PR comment, score, or CI gate only: edit `surfaces` in config instead of disabling the rule.

`ignore.tags` disables every rule carrying that tag before linting. For rules that are not tag-disabled, `rules` overrides `categories`, which override the rule default. `surfaces` is visibility-only and never changes whether a rule runs.

## Config shape

Config lives in `doctor.config.ts`, `doctor.config.js`, `doctor.config.mjs`, `doctor.config.cjs`, `doctor.config.json`, `doctor.config.jsonc`, or the `reactDoctor` key in `package.json`.

```ts
export default {
  rules: { "react-doctor/no-array-index-as-key": "off" },
  categories: { "React Native": "warn" },
  ignore: { tags: ["design"] },
};
```

When explaining a rule, lead with the "Why it matters" guidance from `rules explain` and, when depth is needed, the per-rule recipe at `https://www.react.doctor/prompts/rules/<plugin>/<rule>.md`.
