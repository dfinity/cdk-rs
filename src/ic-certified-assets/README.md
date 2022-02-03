# Certified Assets Library

Rust support for asset certification.

Certified assets can also be served from any Rust canister by including this library.

## Adding to a canister

```
[dependencies]
ic-certified-assets = "0.1.0"
```

The assets are preserved over upgrades by including the corresponding functions in the `init/pre_upgrade/upgrade`
hooks which can be mixed with the other state from the canister:

```
#[derive(Clone, Debug, CandidType, Deserialize)]
struct StableState {
  my_state: MyState,
  assets: crate::assets::StableState,
}

#[init]
fn init() {
  crate::assets::init();
}

>>#[pre_upgrade]
fn pre_upgrade() {
  let stable_state = STATE.with(|s| StableState {
    my_state: s.my_state,
    assets: crate::assets::pre_upgrade(),
  });
  ic_cdk::storage::stable_save((stable_state,)).expect("failed to save stable state");
}

>>#[post_upgrade]
fn post_upgrade() {
  let (StableState { assets, my_state },): (StableState,) =
                                         ic_cdk::storage::stable_restore().expect("failed to restore stable state");
  crate::assets::post_upgrade(assets);
  STATE.with(|s| {
      s.my_state = my_state;
  };
}
```

## Uploading assets

```
cd assets
icx-asset --pem ~/.config/dfx/identity/default/identity.pem --replica https://ic0.app sync <canister_id> .
```
