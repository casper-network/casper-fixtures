use casper_engine_test_support::{
    utils::create_run_genesis_request, ExecuteRequestBuilder, LmdbWasmTestBuilder,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_storage::data_access_layer::GenesisRequest;
use casper_types::{
    account::AccountHash, AddressableEntityHash, GenesisAccount, Key, Motes, PackageHash,
    PublicKey, RuntimeArgs, SecretKey, U512,
};
use once_cell::sync::Lazy;

pub static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([221u8; 32]).unwrap());
pub static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));

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

pub(crate) fn setup() -> (
    LmdbWasmTestBuilder,
    AddressableEntityHash,
    PackageHash,
    AddressableEntityHash,
    PackageHash,
) {
    let mut builder = LmdbWasmTestBuilder::default();
    builder.run_genesis(create_genesis_request()).commit();

    let install_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        "migration_test_contract_post.wasm",
        RuntimeArgs::default(),
    )
    .build();

    let install_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        "owner_contract_post.wasm",
        RuntimeArgs::default(),
    )
    .build();

    builder.exec(install_request_1).expect_success().commit();
    builder.exec(install_request_2).expect_success().commit();

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let contract_hash = account_named_keys
        .get("hash")
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    let contract_package_hash = account_named_keys
        .get("package")
        .and_then(|key| key.into_package_hash())
        .expect("should have package hash");

    let owner_contract_hash = account_named_keys
        .get("owner_c_hash")
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    let owner_contract_package_hash = account_named_keys
        .get("owner_c_package")
        .and_then(|key| key.into_package_hash())
        .expect("should have package hash");

    (
        builder,
        contract_hash,
        contract_package_hash,
        owner_contract_hash,
        owner_contract_package_hash,
    )
}
