use casper_engine_test_support::{
    ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR, MINIMUM_ACCOUNT_CREATION_BALANCE,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_fixtures::generate_fixture;
use casper_types::{runtime_args, system::mint, ContractHash, RuntimeArgs};

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

fn main() {
    generate_fixture(
        "old_version",
        PRODUCTION_RUN_GENESIS_REQUEST.clone(),
        |builder| {
            let id: Option<u64> = None;
            let transfer_1_args = runtime_args! {
                mint::ARG_TARGET => *ACCOUNT_1_ADDR,
                mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
                mint::ARG_ID => id,
            };
            let transfer_2_args = runtime_args! {
                mint::ARG_TARGET => *ACCOUNT_2_ADDR,
                mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
                mint::ARG_ID => id,
            };

            let transfer_request_1 =
                ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_1_args).build();
            let transfer_request_2 =
                ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_2_args).build();

            let install_request_1 = ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                "migration_test_contract_pre.wasm",
                RuntimeArgs::default(),
            )
            .build();
            let install_request_2 = ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                "owner_contract.wasm",
                RuntimeArgs::default(),
            )
            .build();

            builder.exec(transfer_request_1).expect_success().commit();
            builder.exec(transfer_request_2).expect_success().commit();
            builder.exec(install_request_1).expect_success().commit();
            builder.exec(install_request_2).expect_success().commit();

            let account = builder
                .get_account(*DEFAULT_ACCOUNT_ADDR)
                .expect("should have account");

            let contract_hash = account
                .named_keys()
                .get("hash")
                .and_then(|key| key.into_hash())
                .map(ContractHash::new)
                .expect("should have contract hash");

            let owner_contract_hash = account
                .named_keys()
                .get("owner_c_hash")
                .and_then(|key| key.into_hash())
                .map(ContractHash::new)
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

            let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                owner_contract_hash,
                "call_get_free_token",
                runtime_args! {"contract" => Key::Hash(contract_hash.value())},
            )
            .build();
            builder.exec(mint_request_2).expect_success().commit();
        },
    )
    .unwrap();
}
