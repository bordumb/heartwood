# fn-3.3: Add `SignedRefs::verify_with_authority()` method

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/storage/refs.rs`

## What to Do

Add a new verification method that runs the full two-layer check: crypto then authority.
The existing `verify()` method is NOT modified — it remains as the backwards-compatible
single-key path.

```rust
// In impl SignedRefs<Unverified>

/// Verify both the cryptographic signature AND the device's authority to sign.
///
/// This is the preferred verification method for multi-device identity.
/// - Step 1: Ed25519 signature check (unchanged, owned by Radicle)
/// - Step 2: Device authority check via `checker` (attested or direct delegate)
pub fn verify_with_authority(
    self,
    doc: &Doc,
    checker: &dyn DeviceAuthorityChecker,
    repo_id: &RepoId,
    now: DateTime<Utc>,
) -> Result<(SignedRefs<Verified>, DeviceAuthority), VerifyError> {
    // Step 1: crypto (existing logic, unchanged)
    let canonical = self.refs.canonical();
    self.id.verify(canonical.as_ref(), &self.signature)
        .map_err(|_| VerifyError::InvalidSignature)?;

    // Check identity root ref if present (existing logic, unchanged)
    ...

    // Step 2: authority
    let authority = checker.check(&self.id, doc, repo_id, now)
        .map_err(VerifyError::Authority)?;

    Ok((
        SignedRefs {
            id: self.id,
            refs: self.refs,
            signature: self.signature,
            _verified: PhantomData,
        },
        authority,
    ))
}
```

### Update `VerifyError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("invalid signature")]
    InvalidSignature,
    #[error("wrong identity")]
    WrongIdentity,
    #[error("authority check failed: {0}")]
    Authority(#[from] AuthorityError),  // NEW variant
}
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `verify_with_authority()` compiles and returns `(SignedRefs<Verified>, DeviceAuthority)`
- [ ] Existing `verify()` method unchanged
- [ ] Test: valid direct delegate → `Ok((_, DirectDelegate { .. }))`
- [ ] Test: valid attested device → `Ok((_, AttestedDevice { .. }))`
- [ ] Test: invalid signature → `Err(VerifyError::InvalidSignature)` (before authority check)
- [ ] Test: valid crypto, unauthorized device → `Err(VerifyError::Authority(..))`
