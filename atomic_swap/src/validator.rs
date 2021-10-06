use casper_contract::contract_api::{runtime, storage};
use casper_types::account::AccountHash;

pub fn caller_is_recipient() -> bool {
    let caller = runtime::get_caller();
    let recipient: AccountHash =
        storage::read(runtime::get_key("recipient").unwrap().into_uref().unwrap())
            .unwrap()
            .unwrap();
    caller == recipient
}

pub fn caller_is_owner() -> bool {
    let caller = runtime::get_caller();
    let owner = runtime::get_key("owner").unwrap().into_account().unwrap();
    caller == owner
}
