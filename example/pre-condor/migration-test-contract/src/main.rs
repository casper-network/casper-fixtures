#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use casper_contract::{
    contract_api::{
        runtime::{self, get_call_stack, get_named_arg, put_key},
        storage::{self, named_dictionary_get, named_dictionary_put, new_dictionary},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, system::CallStackElement, CLType, ContractHash,
    ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
    RuntimeArgs,
};

#[no_mangle]
pub fn init() {
    new_dictionary("tokens").unwrap_or_revert();
}

#[no_mangle]
pub fn get_free_token() {
    let call_stack = get_call_stack();
    let immediate_caller: Key = match call_stack.iter().rev().nth(1).unwrap_or_revert() {
        CallStackElement::Session { account_hash } => (*account_hash).into(),
        CallStackElement::StoredSession {
            account_hash,
            contract_package_hash: _,
            contract_hash: _,
        } => (*account_hash).into(),
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => (*contract_package_hash).into(),
    };
    let tokens_so_far: u64 = named_dictionary_get("tokens", &immediate_caller.to_string())
        .unwrap_or_revert()
        .unwrap_or(0_u64);
    named_dictionary_put("tokens", &immediate_caller.to_string(), tokens_so_far + 1);
    put_key(
        "event",
        storage::new_uref(immediate_caller.to_string()).into(),
    );
}

#[no_mangle]
pub fn give_free_token() {
    let owner = get_named_arg::<Key>("owner");
    let tokens_so_far: u64 = named_dictionary_get("tokens", &owner.to_string())
        .unwrap_or_revert()
        .unwrap_or(0_u64);
    named_dictionary_put("tokens", &owner.to_string(), tokens_so_far + 1);
    put_key("event", storage::new_uref(owner.to_string()).into());
}

pub fn upgrade() {
    let entry_points = generate_entry_points();

    let contract_package_hash = runtime::get_key("package")
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert();

    let previous_contract_hash = runtime::get_key("hash")
        .unwrap_or_revert()
        .into_hash()
        .map(ContractHash::new)
        .unwrap_or_revert();

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert();
    runtime::put_key("hash", contract_hash.into());
    runtime::put_key("version", storage::new_uref(contract_version).into());
}

pub fn install_contract() {
    let entry_points = generate_entry_points();
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        None,
        Some("package".to_owned()),
        Some("access".to_owned()),
    );
    // this will automatically turn into Key::Hash
    runtime::put_key("hash", contract_hash.into());
    runtime::put_key("version", storage::new_uref(contract_version).into());
    // Call contract to initialize it.
    runtime::call_contract::<()>(contract_hash, "init", runtime_args! {});
}

#[no_mangle]
pub extern "C" fn call() {
    match runtime::get_key("access") {
        Some(_) => {
            upgrade();
        }
        None => {
            install_contract();
        }
    }
}

fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_free_token"),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("give_free_token"),
        vec![Parameter::new("owner", CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("init"),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}
