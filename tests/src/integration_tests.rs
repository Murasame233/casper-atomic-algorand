#[cfg(test)]
mod tests {
    use casper_engine_test_support::{
        AccountHash, Code, Hash, SessionBuilder, TestContext, TestContextBuilder,
    };
    use casper_types::{
        bytesrepr::{FromBytes, ToBytes},
        runtime_args, AsymmetricType, CLTyped, ContractHash, Key, PublicKey, RuntimeArgs, U256,
        U512,
    };

    struct Token {
        user_t: AccountHash,
        user_a: AccountHash,
        user_b: AccountHash,
        token_hash: Hash,
        context: TestContext,
    }

    impl Token {
        fn deploy() -> Token {
            let user_t_pub = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
            let user_a_pub = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
            let user_b_pub = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

            let mut context = TestContextBuilder::new()
                .with_public_key(user_t_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_a_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_b_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .build();

            let user_t = user_t_pub.to_account_hash();
            let user_a = user_a_pub.to_account_hash();
            let user_b = user_b_pub.to_account_hash();

            // deploy the contract
            let session_code = Code::from("erc20_token.wasm");
            let session_args = runtime_args! {
                "name" => "TestToken",
                "symbol" => "TT",
                "total_supply" => U256::from(500_000_000u64),
                "decimals" => 0u8
            };

            let session = SessionBuilder::new(session_code, session_args)
                .with_address(user_t)
                .with_authorization_keys(&[user_t])
                .build();
            context.run(session);

            // get contract_hash
            let contract_hash: ContractHash = context
                .get_account(user_t)
                .unwrap()
                .named_keys()
                .get("erc20_token_contract")
                .unwrap()
                .normalize()
                .into_hash()
                .unwrap()
                .into();
            let token_hash = contract_hash.value();

            Token {
                user_t,
                user_a,
                user_b,
                token_hash,
                context,
            }
        }

        // Token methods
        pub fn token_name(&self) -> String {
            self.query_contract("name").unwrap()
        }

        pub fn balance_of(&self, account: Key) -> Option<U256> {
            let item_key = base64::encode(&account.to_bytes().unwrap());

            let key = Key::Hash(self.token_hash);
            let value = self
                .context
                .query_dictionary_item(key, Some("balances".to_string()), item_key)
                .ok()?;

            Some(value.into_t::<U256>().unwrap())
        }

        pub fn transfer(&mut self, recipient: Key, amount: U256, sender: AccountHash) {
            self.call(
                sender,
                "transfer",
                runtime_args! {
                    "recipient" => recipient,
                    "amount" => amount
                },
            );
        }

        // method for context
        fn call(&mut self, sender: AccountHash, method: &str, args: RuntimeArgs) {
            let code = Code::Hash(self.token_hash, method.to_string());
            let session = SessionBuilder::new(code, args)
                .with_address(sender)
                .with_authorization_keys(&[sender])
                .build();
            self.context.run(session);
        }

        fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
            match self.context.query(
                self.user_t,
                &["erc20_token_contract".into(), name.to_string()],
            ) {
                Err(_) => None,
                Ok(maybe_value) => {
                    let value = maybe_value
                        .into_t()
                        .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                    Some(value)
                }
            }
        }
    }

    #[test]
    fn test_deploy() {
        // Deploy
        let mut t = Token::deploy();
        println!("{}",t.token_name());
        // Read user_t balance
        let balance = t.balance_of(Key::from(t.user_t)).unwrap();
        println!("User_t: {:?}", balance);

        // transfer 50 to user_a
        println!("Transfer 50 from User_t to User_a");
        t.transfer(Key::from(t.user_a), U256::from(50u8), t.user_t);
        let balance_a = t.balance_of(Key::from(t.user_a)).unwrap();
        let balance_t = t.balance_of(Key::from(t.user_t)).unwrap();
        println!("User_t now: {}, User_a now: {}", balance_t, balance_a);

        // transfer 50 to user_b
        println!("Transfer 50 from User_t to User_b");
        t.transfer(Key::from(t.user_b), U256::from(50u8), t.user_t);
        let balance_b = t.balance_of(Key::from(t.user_b)).unwrap();
        let balance_t = t.balance_of(Key::from(t.user_t)).unwrap();
        println!("User_t now: {}, User_b now: {}", balance_t, balance_b);
    }
}

fn main() {
    panic!()
}
