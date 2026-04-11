# Branch Protection Setup

Configure at: **Settings → Rules → Rulesets** (or legacy Branch protection rules)

## main branch ruleset

| Setting | Value |
|---------|-------|
| **Enforcement** | Active |
| **Target branches** | `main` |
| **Bypass list** | kookyleo (admin only) |

### Rules to enable

- [x] **Restrict deletions**
- [x] **Require a pull request before merging**
  - Required approvals: 0 (solo project, CI is the gate)
  - Dismiss stale reviews: off
- [x] **Require status checks to pass**
  - Required checks (must match CI job names):
    - `Format`
    - `Clippy`
    - `Unit Tests`
    - `Reference Tests`
    - `Integration Tests`
    - `Release Build`
  - Require branches to be up to date: yes
- [x] **Block force pushes**

### Result

- Only kookyleo can push directly to main (bypass)
- All others must go through PR
- PR merge requires all 6 CI jobs green
- No force pushes allowed
