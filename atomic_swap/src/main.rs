#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

mod error;
mod helper;
mod validator;
use error::Error;
use helper::{_hash, is_time_out, transfer_to};

use alloc::{
    string::{String, ToString},
    vec,
};
use casper_contract::contract_api::{
    runtime::{self, call_contract, get_caller, revert},
    storage::{self, add_contract_version},
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, contracts::NamedKeys, runtime_args, CLType, CLTyped,
    ContractHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
    RuntimeArgs, U256,
};
use validator::{caller_is_owner, caller_is_recipient};

// user use this for update hash
#[no_mangle]
fn update_hash() {
    if !caller_is_owner() {
        revert(Error::InValidCaller)
    }
    let hash: String = runtime::get_named_arg("hash");
    let k = runtime::get_key("hash").unwrap();
    storage::write(k.into_uref().unwrap(), hash);
}

#[no_mangle]
fn withdraw() {
    if !caller_is_recipient() {
        revert(Error::InValidCaller)
    };
    if is_time_out() {
        revert(Error::TimeOut)
    }
    let secret: String = runtime::get_named_arg("secret");
    let hash = _hash(secret.clone());
    let _hash: String = storage::read(runtime::get_key("hash").unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap();
    if hash == _hash {
        // write secret
        storage::write(
            runtime::get_key("secret").unwrap().into_uref().unwrap(),
            secret,
        );

        //transfer to recipient
        let amount_uref = runtime::get_key("amount").unwrap().into_uref().unwrap();
        let amount: U256 = storage::read(amount_uref).unwrap().unwrap();
        let recipient: AccountHash =
            storage::read(runtime::get_key("recipient").unwrap().into_uref().unwrap())
                .unwrap()
                .unwrap();
        transfer_to(Key::from(recipient), amount);

        // Set amount to 0;
        storage::write(amount_uref, U256::from(0))
    } else {
        revert(Error::InValidSecret)
    }
}

// When timeout user can refund their money
#[no_mangle]
fn refund() {
    // Valid
    if !caller_is_owner() {
        revert(Error::InValidCaller)
    };
    if !is_time_out() {
        revert(Error::TimeUnOut)
    }

    // Get amount to send
    let amount_uref = runtime::get_key("amount").unwrap().into_uref().unwrap();
    let amount: U256 = storage::read(amount_uref).unwrap().unwrap();
    let recipient: AccountHash =
        storage::read(runtime::get_key("owner").unwrap().into_uref().unwrap())
            .unwrap()
            .unwrap();
    transfer_to(Key::from(recipient), amount);

    // Set amount to 0;
    storage::write(amount_uref, U256::from(0))
}

#[no_mangle]
fn call() {
    // Helper functions for initial

    fn put_key<T>(name: &str, value: T, named_keys: &mut NamedKeys)
    where
        T: CLTyped + ToBytes,
    {
        let uref = storage::new_uref(value);
        let data = Key::URef(uref);
        runtime::put_key(name, data);
        named_keys.insert(name.into(), Key::URef(uref));
    }

    fn get_entries() -> EntryPoints {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(EntryPoint::new(
            "update_hash".to_string(),
            vec![Parameter::new("hash", CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "refund".to_string(),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "withdraw".to_string(),
            vec![Parameter::new("secret", CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
        entry_points
    }

    fn finish_setup(named_keys: NamedKeys) {
        let (contract_package_hash, _) = storage::create_contract_package_at_hash();
        let (contract_hash, _) =
            add_contract_version(contract_package_hash, get_entries(), named_keys);

        let token: ContractHash = runtime::get_named_arg("token");
        let balance: U256 = call_contract(
            token,
            "balance_of",
            runtime_args! {"address"=> Key::from(get_caller())},
        );
        let amount = runtime::get_named_arg("amount");
        if balance < amount {
            revert(Error::NotEnoughBalance)
        } else {
            // There is must to say, It looks like a `defects` in the erc-20 casper implement
            // use callstack to validate is a good idea, but when call a not `stored contract`(just like when creating),
            // the callstack will not contain the unstored contract, so the contract can tracsfer token from user
            // I think it need to be repair
            call_contract::<()>(
                token,
                "transfer",
                runtime_args! {
                "recipient" => Key::from(contract_package_hash),
                "amount" => amount
                },
            );
            runtime::put_key("atomic_contract", contract_hash.into());
            let contract_hash_pack = storage::new_uref(contract_hash);
            runtime::put_key("atomic_contract_hash", contract_hash_pack.into());
        }
    }

    //End of helper functions.

    // Global state
    // Have these keys
    // - secret
    // - hash
    // - owner
    // - recipient
    // - start
    // - end
    // - amount
    // - token

    let mut named_keys = NamedKeys::new();

    // Password
    put_key("secret", "".to_string(), &mut named_keys);
    put_key::<String>("hash", runtime::get_named_arg("hash"), &mut named_keys);

    // Swaping Users
    let owner = runtime::get_caller();
    put_key::<AccountHash>("owner", owner, &mut named_keys);
    put_key::<AccountHash>(
        "recipient",
        runtime::get_named_arg("recipient"),
        &mut named_keys,
    );

    // Time
    let now: u64 = runtime::get_blocktime().into();
    put_key::<u64>("start", now, &mut named_keys);
    let end = now + runtime::get_named_arg::<u64>("end");
    put_key::<u64>("end", end, &mut named_keys);

    // Token Amount
    put_key::<U256>("amount", runtime::get_named_arg("amount"), &mut named_keys);
    put_key::<ContractHash>("token", runtime::get_named_arg("token"), &mut named_keys);
    finish_setup(named_keys);
}
