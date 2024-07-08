use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, UpgradeRequestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_fixtures::LmdbFixtureState;
use casper_types::{
    contracts::ContractPackageHash, runtime_args, AddressableEntityHash, EraId, Key,
    ProtocolVersion,
};

use crate::{
    installer_request_builders::ACCOUNT_2_ADDR_KEY, tokens::get_dictionary_value_from_key,
};

pub fn upgrade_v1_5_6_fixture_to_v2_0_0_ee(
    builder: &mut LmdbWasmTestBuilder,
    lmdb_fixture_state: &LmdbFixtureState,
) {
    // state hash in builder and lmdb storage should be the same
    assert_eq!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    // we upgrade the execution engines protocol from 1.x to 2.x
    let mut upgrade_config = UpgradeRequestBuilder::new()
        .with_current_protocol_version(lmdb_fixture_state.genesis_protocol_version())
        .with_new_protocol_version(ProtocolVersion::V2_0_0)
        .with_migrate_legacy_accounts(true)
        .with_migrate_legacy_contracts(true)
        .with_activation_point(EraId::new(1))
        .build();

    builder
        .upgrade(&mut upgrade_config)
        .expect_upgrade_success()
        .commit();

    // the state hash should now be different
    assert_ne!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );
}

// the difference between the two is that in v1_binary the contract hash is fetched at [u8;32],
// while in v2_binary it is an AddressaleEntityHash
pub fn get_contract_hash_v1_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let maybe_hash_key = account_named_keys
        .get("hash")
        .expect("should have hash key");

    let hash = maybe_hash_key
        .into_hash_addr()
        .expect("should have contract hash");

    AddressableEntityHash::new(hash)
}

pub fn get_contract_hash_v2_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let contract_hash = account_named_keys
        .get("hash")
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    contract_hash
}

// in this test we upgrade the execution engine, but don't upgrade the contract
#[test]
fn test_condor_without_contract_upgrade() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("old_version");

    // upgrade the execution engine from 1.5.6 to 2.0.0
    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let contract_hash = get_contract_hash_v1_binary(&builder);

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();

    let account_named_keys = account.named_keys();
    let maybe_package_hash = account_named_keys
        .get("owner_c_package")
        .expect("should have data");

    let owner_contract_package_hash = maybe_package_hash
        .into_hash_addr()
        .map(ContractPackageHash::new)
        .expect("should have package hash");

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_hash,
        "give_free_token",
        runtime_args! {"owner" => *ACCOUNT_2_ADDR_KEY},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();
    let defaults_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &Key::Account(*DEFAULT_ACCOUNT_ADDR).to_string(),
    );
    let account_2_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &ACCOUNT_2_ADDR_KEY.to_string(),
    );
    let contract_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &Key::Hash(owner_contract_package_hash.value()).to_string(),
    );
    assert!(defaults_token < account_2_token);
    assert_eq!(defaults_token, contract_token);
}

// we upgrade the execution engine, then upgrade our contract to a version that handles
// the hashes and keys according to what's available in 2.0.0
// TAKE NOTE that when we rewrote our contract for 2.0.0 we did not make a migration entry_point
// nor did we write our entry_points in a way that would convert the new data types to the old ones
// during execution. This caused our owner_contract to have two different "accounts" with different
// balances.
#[test]
fn test_condor_with_contract_upgrade() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("old_version");

    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        "migration_test_contract_post.wasm",
        runtime_args! {},
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let contract_hash = get_contract_hash_v2_binary(&builder);

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();

    let account_named_keys = account.named_keys();

    let maybe_owner_contract_package_hash = account_named_keys
        .get("owner_c_package")
        .and_then(|key| key.into_hash_addr());

    let owner_contract_package_hash =
        maybe_owner_contract_package_hash.expect("should have package hash");

    let maybe_owner_contract_hash = account_named_keys
        .get("owner_c_hash")
        .and_then(|key| key.into_hash_addr());

    let owner_contract_hash = maybe_owner_contract_hash.expect("should have package hash");

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_hash,
        "give_free_token",
        runtime_args! {"owner" => *ACCOUNT_2_ADDR_KEY},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        owner_contract_hash.into(),
        "call_get_free_token",
        runtime_args! {"contract" => &Key::Hash(contract_hash.value())},
    )
    .build();
    builder.exec(mint_request_2).expect_success().commit();

    let defaults_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &Key::Account(*DEFAULT_ACCOUNT_ADDR).to_string(),
    );
    let account_2_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &ACCOUNT_2_ADDR_KEY.to_string(),
    );
    let contract_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &Key::Hash(owner_contract_package_hash).to_string(),
    );
    let new_contract_token: u64 = get_dictionary_value_from_key(
        &builder,
        &contract_hash,
        "tokens",
        &Key::Package(owner_contract_package_hash).to_string(),
    );
    assert_eq!(defaults_token, 1_u64);
    assert_eq!(account_2_token, 2_u64);
    assert!(defaults_token < account_2_token);
    assert_eq!(defaults_token, contract_token);
    assert_eq!(new_contract_token, contract_token);
}
