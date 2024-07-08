use casper_engine_test_support::{ExecuteRequestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{bytesrepr::FromBytes, runtime_args, CLTyped, Key, RuntimeArgs};

use crate::installer_request_builders::{setup, ACCOUNT_2_ADDR_KEY};

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
    contract_key: &Key,
    dictionary_name: &str,
    dictionary_key: &str,
) -> T {
    let seed_uref = *builder
        .query(None, *contract_key, &[])
        .expect("must have nft contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(dictionary_name)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, seed_uref, dictionary_key)
        .expect("should have dictionary value")
        .as_cl_value()
        .expect("T should be CLValue")
        .to_owned()
        .into_t()
        .unwrap()
}

#[test]
fn test_mint_and_burn_tokens() {
    let (
        mut builder,
        contract_hash,
        _contract_package_hash,
        owner_contract_hash,
        owner_contract_package_hash,
    ) = setup();
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_hash,
        "give_free_token",
        runtime_args! {"owner" => *ACCOUNT_2_ADDR_KEY},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

    let event = builder
        .query(
            None,
            Key::Hash(contract_hash.value()),
            &["event".to_owned()],
        )
        .unwrap();
    let event: String = event.as_cl_value().unwrap().to_owned().into_t().unwrap();
    println!("{event:?}");

    let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_hash,
        "get_free_token",
        runtime_args! {},
    )
    .build();
    builder.exec(mint_request_2).expect_success().commit();

    let event = builder
        .query(
            None,
            Key::Hash(contract_hash.value()),
            &["event".to_owned()],
        )
        .unwrap();
    let event: String = event.as_cl_value().unwrap().to_owned().into_t().unwrap();
    println!("{event:?}");

    let mint_request_3 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        owner_contract_hash,
        "call_get_free_token",
        runtime_args! {"contract" => Key::Hash(contract_hash.value())},
    )
    .build();
    builder.exec(mint_request_3).expect_success().commit();

    let defaults_token: u64 = get_dictionary_value_from_key(
        &builder,
        &Key::Hash(contract_hash.value()),
        "tokens",
        &Key::Account(*DEFAULT_ACCOUNT_ADDR).to_string(),
    );
    let account_2_token: u64 = get_dictionary_value_from_key(
        &builder,
        &Key::Hash(contract_hash.value()),
        "tokens",
        &ACCOUNT_2_ADDR_KEY.to_string(),
    );
    let contract_token: u64 = get_dictionary_value_from_key(
        &builder,
        &Key::Hash(contract_hash.value()),
        "tokens",
        &Key::Hash(owner_contract_package_hash.value()).to_string(),
    );
    assert_eq!(defaults_token, account_2_token);
    assert_eq!(defaults_token, contract_token);
}
