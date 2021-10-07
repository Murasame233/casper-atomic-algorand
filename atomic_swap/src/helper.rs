use alloc::string::String;
use casper_contract::contract_api::{
    runtime::{self, call_contract},
    storage,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, U256};

// Use sha3 for hashing
pub fn _hash(data: String) -> String {
    use sha3::Digest;
    use sha3::Keccak256;
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// pub fn transfer_from(from:AccountHash,to:Key,amount:U256){
//     let token: ContractHash =
//     storage::read(runtime::get_key("token").unwrap().into_uref().unwrap())
//         .unwrap()
//         .unwrap();
// }

pub fn transfer_to(to: Key, amount: U256) {
    let token: ContractHash =
        storage::read(runtime::get_key("token").unwrap().into_uref().unwrap())
            .unwrap()
            .unwrap();
    call_contract::<()>(
        token,
        "transfer",
        runtime_args! {
            "recipient" => to,
            "amount" => amount
        },
    );
}

pub fn is_time_out() -> bool {
    let end: u64 = storage::read(runtime::get_key("end").unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap();
    let now: u64 = runtime::get_blocktime().into();
    now > end
}
