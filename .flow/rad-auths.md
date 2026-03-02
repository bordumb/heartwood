  What exists and where                                                                                                
                                                                                                                         
  auths repo — /Users/bordumb/workspace/repositories/auths-base/auths/.flow/                                           
                                                                                                                       
  Tasks that modify auths-radicle, auths-id, and auths-verifier:

  ┌─────────────────┬─────────────────────────────┬────────────────────────────────────────────────────────────┐
  │      File       │       Crate modified        │                        What it does                        │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-5.1.md │ auths-radicle/src/bridge.rs │ Add find_identity_for_device() to RadicleAuthsBridge trait │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-5.2.md │ auths-radicle/src/refs.rs   │ New file — RIP-X ref path constants (KERI_KEL_REF etc.)    │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-5.3.md │ auths-verifier/src/lib.rs   │ Add Attestation::to_bytes() / from_bytes()                 │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-6.1.md │ auths-id/src/keri/          │ Add GitKel::with_ref() constructor                         │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-6.2.md │ auths-id/src/keri/          │ Export RIP_KEL_REF constant                                │
  ├─────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────────┤
  │ tasks/fn-6.3.md │ auths-id/src/keri/          │ Tests for custom ref path                                  │
  └─────────────────┴─────────────────────────────┴────────────────────────────────────────────────────────────┘

  Every file now has ## Repo: auths at the top.

  heartwood repo — /Users/bordumb/workspace/repositories/heartwood/.flow/

  Tasks that modify heartwood crates only — zero auths source changes:

  ┌───────────────────────┬────────────┬───────────────────────────────────────────────────┐
  │         Epic          │   Tasks    │                      Crates                       │
  ├───────────────────────┼────────────┼───────────────────────────────────────────────────┤
  │ fn-1 (types)          │ fn-1.1–1.5 │ radicle::identity                                 │
  ├───────────────────────┼────────────┼───────────────────────────────────────────────────┤
  │ fn-2 (storage)        │ fn-2.1–2.4 │ radicle::identity, radicle::storage               │
  ├───────────────────────┼────────────┼───────────────────────────────────────────────────┤
  │ fn-3 (verification)   │ fn-3.1–3.4 │ radicle::identity, radicle::storage               │
  ├───────────────────────┼────────────┼───────────────────────────────────────────────────┤
  │ fn-4 (fetch/protocol) │ fn-4.1–4.4 │ radicle::storage, radicle-fetch, radicle-protocol │
  ├───────────────────────┼────────────┼───────────────────────────────────────────────────┤
  │ fn-5 (CLI)            │ fn-5.1–5.5 │ radicle-cli                                       │
  └───────────────────────┴────────────┴───────────────────────────────────────────────────┘

  Every file now has ## Repo: heartwood at the top.