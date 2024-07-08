use casper_engine_test_support::{
    utils::create_run_genesis_request, ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_fixtures::generate_fixture;
use casper_storage::data_access_layer::GenesisRequest;
use casper_types::{runtime_args, GenesisAccount, Motes, RuntimeArgs, U512};

use casper_types::{account::AccountHash, Key, PublicKey, SecretKey};
use once_cell::sync::Lazy;

pub static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([221u8; 32]).unwrap());
pub static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
pub static ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());
pub static ACCOUNT_1_ADDR_KEY: Lazy<Key> = Lazy::new(|| {
    let key: Key = (*ACCOUNT_1_ADDR).into();
    key
});

pub static ACCOUNT_2_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([212u8; 32]).unwrap());
pub static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_2_SECRET_KEY));
pub static ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_2_PUBLIC_KEY.to_account_hash());
pub static ACCOUNT_2_ADDR_KEY: Lazy<Key> = Lazy::new(|| {
    let key: Key = (*ACCOUNT_2_ADDR).into();
    key
});

pub fn create_genesis_request() -> GenesisRequest {
    create_run_genesis_request(vec![
        GenesisAccount::Account {
            public_key: DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_1_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_2_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
    ])
}

fn main() {
    generate_fixture("old_version", create_genesis_request(), |builder| {
        let install_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            "migration_test_contract_pre.wasm",
            RuntimeArgs::default(),
        )
        .build();

        builder.exec(install_request).expect_success().commit();

        let account = builder
            .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
            .unwrap();
        let account_named_keys = account.named_keys();

        let contract_hash = account_named_keys
            .get("hash")
            .and_then(|key| key.into_entity_hash())
            .expect("should have contract hash");

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
            contract_hash,
            "get_free_token",
            runtime_args! {},
        )
        .build();
        builder.exec(mint_request_2).expect_success().commit();
    })
    .unwrap();
}
