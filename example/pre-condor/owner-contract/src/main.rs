#![no_std]
#![no_main]

extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec};

use casper_contract::{
    contract_api::{
        runtime::{self, call_contract, get_named_arg},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    runtime_args, CLType, ContractHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Key, Parameter, RuntimeArgs,
};

#[no_mangle]
pub fn call_get_free_token() {
    let contract = get_named_arg::<Key>("contract");
    call_contract::<()>(
        contract
            .into_hash()
            .map(ContractHash::new)
            .unwrap_or_revert(),
        "get_free_token",
        runtime_args! {},
    );
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = generate_entry_points();
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        None,
        Some("owner_c_package".to_owned()),
        Some("owner_c_access".to_owned()),
    );
    // this will automatically turn into Key::Hash
    runtime::put_key("owner_c_hash", contract_hash.into());
    runtime::put_key("owner_c_verion", storage::new_uref(contract_version).into());
}

fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        String::from("call_get_free_token"),
        vec![Parameter::new("owner", CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}
