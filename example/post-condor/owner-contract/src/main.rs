#![no_std]
#![no_main]

extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec};

use casper_contract::contract_api::{
    runtime::{self, call_contract, get_named_arg, revert},
    storage,
};
use casper_types::{
    runtime_args, AddressableEntityHash, ApiError, CLType, EntityAddr, EntryPoint,
    EntryPointAccess, EntryPointPayment, EntryPointType, EntryPoints, Key, Parameter,
};

#[no_mangle]
pub fn call_get_free_token() {
    let contract = get_named_arg::<Key>("contract");
    let addressable_entity_hash = AddressableEntityHash::new(match contract {
        Key::Hash(contract_hash) => contract_hash,
        Key::AddressableEntity(EntityAddr::SmartContract(contract_hash)) => contract_hash,
        _ => revert(ApiError::User(100)),
    });
    call_contract::<()>(addressable_entity_hash, "get_free_token", runtime_args! {});
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = generate_entry_points();
    let (addressable_contract_entity_hash, contract_version) = storage::new_contract(
        entry_points,
        None,
        Some("owner_c_package".to_owned()),
        Some("owner_c_access".to_owned()),
        None,
    );
    // this will automatically turn into Key::Hash
    runtime::put_key(
        "owner_c_hash",
        Key::AddressableEntity(EntityAddr::SmartContract(
            addressable_contract_entity_hash.value(),
        )),
    );
    runtime::put_key("owner_c_verion", storage::new_uref(contract_version).into());
}

fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        String::from("call_get_free_token"),
        vec![Parameter::new("owner", CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));
    entry_points
}
