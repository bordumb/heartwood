# Epic fn-5: CLI Commands for Device Management

## Goal

Provide `rad identity` subcommands for creating KERI identities, attesting new devices,
listing devices, revoking devices, and rotating keys. These commands are the primary user
interface for multi-device identity management.

## Background

A user who wants to use Heartwood from two machines needs:
1. A KERI identity repo (created once, stored on the network)
2. An attestation for each device key (signed by both the identity key and the device key)
3. A way to revoke a device if it's lost or compromised
4. A way to rotate the master KERI identity key periodically

## New Commands

### `rad identity init-keri`
Creates a new KERI identity repo locally and seeds it to the network.
- Generates a new KERI inception event (or accepts an existing KERI prefix)
- Creates a new git repo with `refs/keri/kel` pointing to the inception commit
- Seeds the repo via `rad seed <rid>`
- Prints the new KERI DID and RID

### `rad identity device add [--key <node-id>] [--expires <date>]`
Creates a 2-way attestation between the KERI identity and a device key.
- Defaults to current node's key if `--key` not given
- Prompts the user to also run the command on the new device to get its signature
- Writes attestation blobs to `refs/keys/<nid>/signatures` in the identity repo
- Pushes updated identity repo to the network

### `rad identity device list`
Lists all currently attested devices under the user's KERI identity.
- Shows: NodeId, alias, attestation date, expiry, capabilities, status (active/revoked)
- Sources data from the local identity repo (fetches first if `--fetch` given)

### `rad identity device revoke <node-id>`
Revokes an attested device.
- Sets `revoked_at` on the attestation and re-signs with the KERI identity key
- Writes updated attestation to identity repo
- Pushes to network

### `rad identity rotate [--next-key <pubkey>]`
Rotates the KERI identity key by appending a rotation event to the KEL.
- Requires pre-commitment to have been set at inception or last rotation
- Creates a `RotEvent` via `auths-id::keri::rotate_keys()`
- Appends to `refs/keri/kel`

## Affected Crates

- `crates/radicle-cli/src/commands/identity/` — new subcommands
- `crates/radicle-cli/src/commands/identity/keri.rs` (new)
- `crates/radicle-cli/src/commands/identity/device.rs` (new)
- `crates/radicle/src/identity/keri.rs` — additional init helpers
- `crates/radicle-term/src/` — display formatting for device list

## Key Design Decisions

1. **Two-phase attestation for `device add`**: The identity owner signs first (from machine A).
   They share the partial attestation (a JSON blob) with the new device owner. The new device
   signs the partial attestation and returns the completed version. The identity owner stores
   it. This mirrors the RIP 2-way signing requirement.

2. **`--key` flag**: Accepts a hex `NodeId`. The common case is attesting your own current
   device (flag omitted), but it also supports attesting a device without physical access
   (if you have the public key and they'll co-sign later).

3. **`rad identity device list` output**: Tabular output aligned with existing `rad` UX.
   Revoked devices shown in grey/strikethrough (in color-capable terminals).

4. **Error messages**: If `rad identity device add` is run without a KERI identity, it
   suggests running `rad identity init-keri` first.

## Acceptance Criteria

- [ ] `rad identity init-keri` creates a valid KERI identity repo with inception event
- [ ] `rad identity device add` creates correct 2-way attestation blobs in git
- [ ] `rad identity device list` shows all attested devices with correct status
- [ ] `rad identity device revoke <nid>` marks attestation as revoked and re-signs
- [ ] `rad identity rotate` appends a valid `RotEvent` to the KEL
- [ ] All commands fail gracefully with clear error messages for invalid inputs
- [ ] `cargo test -p radicle-cli` passes
