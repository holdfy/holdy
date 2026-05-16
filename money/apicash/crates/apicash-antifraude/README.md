# apicash-antifraude

Anti-fraud and risk module for **APICash**: **User Score** (0–1000), **SEFAZ / Receita**-style CPF checks, **social** profile signals (Instagram, TikTok, Facebook URLs), and an **on-ramp decision** (`Approve` / `Review` / `Block`) before Stellar funding.

## Features

| Feature | Purpose |
| --- | --- |
| `mock` | Forces deterministic SEFAZ/social paths (no live HTTP assumptions in CI). |

Default builds still use embedded deterministic mocks when no provider URL is wired; enable `mock` for explicit test behaviour.

## Main types

- **`AntiFraudeService::calculate_score(user_id, cpf, social_links)`** — runs validators, applies [`ScoreCalculator`](src/score/score_calculator.rs), persists via [`ScoreRepository`](src/repository/score_repository.rs).
- **`UserScore`** — `score`, [`RiskLevel`](src/score/user_score.rs), `factors`, `decision` ([`OnRampDecision`](src/score/user_score.rs)).
- **`SefazValidator` / `SocialValidator`** — structured for FiscalAPI-style or Meta/TikTok APIs later.

## Scoring (summary)

- Regular CPF (mock: not ending in `00`): **+300**
- At least one social account **≥ 6 months** (mock: URL contains `old` or `verified`): **+150** (once)
- Each **open dispute**: **−120**
- [`RiskLevel`](src/score/user_score.rs) and [`OnRampDecision`](src/score/user_score.rs) from thresholds in [`score_calculator.rs`](src/score/score_calculator.rs)

## Tests

```bash
cargo test -p apicash-antifraude
cargo test -p apicash-antifraude --features mock
```

## Dependencies

Uses [`apicash-shared`](../apicash-shared) for shared constants (e.g. `USER_SCORE_MAX`). Replace [`InMemoryScoreRepository`](src/repository/score_repository.rs) with a SQLx-backed implementation when persistence is required.
