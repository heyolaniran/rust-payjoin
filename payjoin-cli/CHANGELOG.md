# payjoin-cli Changelog

## 0.2.0

- Support sender and receiver being generic over their typestate ([#728](https://github.com/payjoin/rust-payjoin/pull/728), [#719](https://github.com/payjoin/rust-payjoin/pull/719))
- Remove existing persistence strategy and replace with new SessionPersister trait ([#789](https://github.com/payjoin/rust-payjoin/pull/789), [#750](https://github.com/payjoin/rust-payjoin/pull/750))
- Update references to BIP-77 ([#733](https://github.com/payjoin/rust-payjoin/pull/733))
- Rename RelayState to RelayManager for clarity ([#727](https://github.com/payjoin/rust-payjoin/pull/727))
- Refactor config to use proc macros over builder ([#703](https://github.com/payjoin/rust-payjoin/pull/703))
- Create fallback ohttp-relay logic for payjoin-cli ([#607](https://github.com/payjoin/rust-payjoin/pull/607))
- Separate v2 cli logic into mod and ohttp components ([#714](https://github.com/payjoin/rust-payjoin/pull/714))
- Update Nigiri instructions ([#691](https://github.com/payjoin/rust-payjoin/pull/691), [#682](https://github.com/payjoin/rust-payjoin/pull/682))
- Dedupe ImplementationError ([#669](https://github.com/payjoin/rust-payjoin/pull/669))
- Add Quick Start, Configuration, Reference to CLI README ([#624](https://github.com/payjoin/rust-payjoin/pull/624))

## 0.1.0

- Bump payjoin to 0.23.0 with stable wire protocol
- Allow mixed input scripts [#367](https://github.com/payjoin/rust-payjoin/pull/367) [#505](https://github.com/payjoin/rust-payjoin/pull/505)
- Fix bug to propagate missing config parameter or argument error [#441](https://github.com/payjoin/rust-payjoin/pull/441)
- Don't pause between long polling requests [#463](https://github.com/payjoin/rust-payjoin/pull/463)
- Hide danger-local-https feature with _ prefix [#423](https://github.com/payjoin/rust-payjoin/pull/423)
- Allow specifying a max-feerate for receivers [#332](https://github.com/payjoin/rust-payjoin/pull/332)
- Fix e2e tests and coverage reporting [#443](https://github.com/payjoin/rust-payjoin/pull/443) [#497](https://github.com/payjoin/rust-payjoin/pull/497) [#532](https://github.com/payjoin/rust-payjoin/pull/532)
- Handle recoverable receiver errors by replying to sender with error response [#474](https://github.com/payjoin/rust-payjoin/pull/474) [#526](https://github.com/payjoin/rust-payjoin/pull/526) [#534](https://github.com/payjoin/rust-payjoin/pull/534)
- Make config.toml hierarchical [#538](https://github.com/payjoin/rust-payjoin/pull/538)
- Make v1/v2 features additive [#538](https://github.com/payjoin/rust-payjoin/pull/538)

## 0.0.9-alpha

- Make backwards-compatible v2 to v1 sends possible
- Bump payjoin to v0.20.0

## 0.0.8-alpha

This release attempts to stabilize the Payjoin V2 Bitcoin URI format. That includes placing v2-specific parameters in the URI's pj parameter's fragment and including the exp expiration parameter.

- Update to `payjoin-0.19.0`
  - Error if send or receive session expires with `exp` parameter [#299](https://github.com/payjoin/rust-payjoin/pull/299)
  - Encode `&ohttp=` and `&exp=` parameters in the `&pj=` URL as a fragment instead of as URI params [#298](https://github.com/payjoin/rust-payjoin/pull/298)
  - Allow receivers to make payjoins out of sweep transactions [#259](https://github.com/payjoin/rust-payjoin/pull/259)

## 0.0.7-alpha

- Resume multiple payjoins easily with the `resume` subcommand. A repeat `send`
  subcommand will also resume an existing session ([#283](https://github.com/payjoin/rust-payjoin/pull/283))
- Normalize dash-separated long args ([#295](https://github.com/payjoin/rust-payjoin/pull/295))
- Use sled database. Old .json storage files will no longer be read and should be deleted.
- read Network::from_core_arg ([#304](https://github.com/payjoin/rust-payjoin/pull/304))
- Don't needlessly substitute outputs for v2 receivers ([#277](https://github.com/payjoin/rust-payjoin/pull/277))
- Print instructions and info on interrupt ([#303](https://github.com/payjoin/rust-payjoin/pull/303))

### Contributors:

@DanGould, @grizznaut, @thebrandonlucas

## 0.0.6-alpha

- fetch ohttp keys from `payjoin/io` feature
- add example.config.toml
- Rename config.toml & CLI argument field pj_host to port (#253)
- add `--version` & `-V` CLI arguments
- replace dependency on `ureq` with `reqwest`
- Unify `pj_host`, `--host-port` arguments to `port` for v1 receivers
- remove `sub_only` CLI argument and config option
- Include more verbose context when bitcoind fails (#251)
- Use `*rpcpassword` instead of `*rpcpass` config and option to match bitcoind
- Test with JoinMarket
- respect `disableoutputsubtitution` send parameter
- depend on `payjoin-0.16.0`
- separate V1 `pj_endpoint` and V2 `pj_directory` config params / cli arguments

Contributors:

@jbesraa, @grizznaut, @thebrandonlucas, @DanGould

## 0.0.5-alpha

- fetch ohttp keys through CONNECT tunnel (#194) instead of manual configuration
- Name payjoin-directory and OHTTP relay according to BIP 77 (#203)

## 0.0.4-alpha

- Remove annoying duplicate code in tests. (#197)
- Refactor payjoin-cli v1, v2 features into modules (#198)
- Parse AppConfig types when they're passed (#195)
- Use spec OHTTP media types (#160)
- Handle ResponseError version-unsupported variant supported field (#165)

## 0.0.3-alpha

- Parse `WellKnownError` `ResponseError` from receivers (#120)
- Show OHTTP Config issue was unclear (#153)
- Better compatibility for `receive` on taproot wallets (#147)

## 0.0.2-alpha

- New `v2` oblivious, asynchronous, serverless payjoin support

## 0.0.1-alpha

- Release initial payjoin-cli to send and receive payjoin from bitcoind
