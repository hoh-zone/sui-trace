# Security Rule Library v1

Each rule lives under `crates/trace-security/src/rules/`. Adding a new rule is
a single file plus one line in `rules/mod.rs`.

| ID    | Name                            | Severity | Confidence | What it catches |
| ----- | ------------------------------- | -------- | ---------- | --------------- |
| R001  | Visibility Confusion            | High     | 0.85       | `public(package) entry` mistakes |
| R002  | Missing Sender Authorization    | Critical | 0.70       | Admin-flavoured public functions without sender or capability check |
| R003  | Clock Unit Mismatch             | High     | 0.80       | `clock::timestamp_ms` compared against second-based fields |
| R004  | Unsafe Arithmetic               | Medium   | 0.60       | `*` on user-controlled values without a checked-math helper |
| R005  | Mutable Shared Object w/o Lock  | High     | 0.55       | Shared object holding `Balance/Coin` mutated by an open `&mut Self` |
| R006  | Capability Leak                 | Critical | 0.75       | Public function returns a `Cap`/`Capability`/`Witness` |
| R007  | Untrusted External Call         | Medium   | 0.50       | Cross-package call into an unvetted publisher |
| R008  | Unbounded Loop / DoS Vector     | Medium   | 0.60       | Loop body iterates over user-supplied `vector`/`Table` without a length cap |
| R009  | Insecure Randomness             | High     | 0.70       | Random derived from `clock` or object-id instead of `0x8::random` |
| R010  | Loose Upgrade Policy            | Medium   | 0.60       | `init` keeps upgrade cap mutable and exposes admin caps |

## Score & severity aggregation

```
score = sum(severity_weight * confidence) / 4   // capped at 10
severity_weight: info=0.5, low=1, medium=3, high=7, critical=10
```

The maximum severity is the worst single finding's severity.

## Adding a rule

1. Create `crates/trace-security/src/rules/r0XX_my_rule.rs`.
2. Implement the `Rule` trait, returning a `SecurityFinding` for every match.
3. Append the new struct to `all_rules()` in `rules/mod.rs`.
4. Document it in this table.

The engine takes care of fanning out execution, score aggregation and DB
persistence; new rules don't need to touch anything else.
