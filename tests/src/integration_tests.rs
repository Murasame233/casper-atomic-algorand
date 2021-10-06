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

            let context = TestContextBuilder::new()
                .with_public_key(user_t_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_a_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_b_pub.clone(), U512::from(500_000_000_000_000_000u64))
                .build();

            let user_t = user_t_pub.to_account_hash();
            let user_a = user_a_pub.to_account_hash();
            let user_b = user_b_pub.to_account_hash();

            let token_hash = user_t.value();

            let mut t = Token {
                user_t,
                user_a,
                user_b,
                token_hash,
                context,
            };

            t.token_hash = t
                .deploy_contract(
                    "erc20_token.wasm",
                    runtime_args! {
                        "name" => "TestToken",
                        "symbol" => "TT",
                        "total_supply" => U256::from(500_000_000u64),
                        "decimals" => 0u8
                    },
                    t.user_t,
                    "erc20_token_contract",
                )
                .value();

            t
        }

        // key is the key of hash store in state
        pub fn deploy_contract(
            &mut self,
            file: &str,
            args: RuntimeArgs,
            account: AccountHash,
            key: &str,
        ) -> ContractHash {
            let session_code = Code::from(file);
            let session = SessionBuilder::new(session_code, args)
                .with_address(account)
                .with_authorization_keys(&[account])
                .build();
            self.context.run(session);

            // get contract_hash
            self.context
                .get_account(account)
                .unwrap()
                .named_keys()
                .get(key)
                .unwrap()
                .normalize()
                .into_hash()
                .unwrap()
                .into()
        }

        // Token methods
        pub fn token_name(&self) -> String {
            self.query_token_contract("name").unwrap()
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
                self.token_hash,
                sender,
                "transfer",
                runtime_args! {
                    "recipient" => recipient,
                    "amount" => amount
                },
            );
        }

        // method for context
        fn call(&mut self, hash: Hash, sender: AccountHash, method: &str, args: RuntimeArgs) {
            let code = Code::Hash(hash, method.to_string());
            let session = SessionBuilder::new(code, args)
                .with_address(sender)
                .with_authorization_keys(&[sender])
                .build();
            self.context.run(session);
        }
        fn query<T: CLTyped + FromBytes>(&self, name: &str, account: AccountHash) -> Option<T> {
            match self.context.query(account, &[name.to_string()]) {
                Err(_) => None,
                Ok(maybe_value) => {
                    let value = maybe_value
                        .into_t()
                        .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                    Some(value)
                }
            }
        }
        fn query_token_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
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
    fn test_deploy_erc20() {
        // Deploy
        let mut t = Token::deploy();
        println!("{}", t.token_name());
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

    #[test]
    fn test_deploy_atomic_swap() {
        // deploy
        let mut t = Token::deploy();
        t.transfer(Key::from(t.user_a), U256::from(50u8), t.user_t);
        println!("Token is deployed");

        // deploy
        let args = runtime_args! {
            "token"=> ContractHash::from(t.token_hash),
            "amount" => U256::from(10),
            "end" => 50u64,
            "recipient" => t.user_b,
            "hash" => _hash("wow".into())
        };
        let atomic_swap_hash =
            t.deploy_contract("atomic_swap.wasm", args, t.user_a, "atomic_contract");
        println!(
            "Atomic Swap Deployed: {}",
            atomic_swap_hash.to_formatted_string()
        );

        // test deploy
        // hash
        let hash_re = t.query::<String>("hash", t.user_a).unwrap();
        assert_eq!(hash_re, _hash("wow".into()));
        // token
        let token = t.query::<ContractHash>("token", t.user_a).unwrap();
        assert_eq!(token, ContractHash::from(t.token_hash));
        // ...
    }

    #[test]
    fn test_atomic_swap_withdraw() {
        // deploy
        let mut t = Token::deploy();
        t.transfer(Key::from(t.user_a), U256::from(50u8), t.user_t);
        println!(
            "Token is deployed, uesr_a have: {}",
            t.balance_of(Key::from(t.user_a)).unwrap()
        );

        // deploy atomic swap
        let args = runtime_args! {
            "token"=> ContractHash::from(t.token_hash),
            "amount" => U256::from(10),
            "end" => 50u64,
            "recipient" => t.user_b,
            "hash" => _hash("wow".into())
        };
        let atomic_swap_hash =
            t.deploy_contract("atomic_swap.wasm", args, t.user_a, "atomic_contract");
        println!(
            "Atomic Swap Deployed: {}",
            atomic_swap_hash.to_formatted_string()
        );

        // after deploy user_a's token has been transfer to contract
        assert_eq!(t.balance_of(Key::from(t.user_a)).unwrap(), U256::from(40));

        // b withdraw
        t.call(
            atomic_swap_hash.value(),
            t.user_b,
            "withdraw",
            runtime_args! { "secret" => "wow"},
        );
        let amount = t.balance_of(Key::from(t.user_b)).unwrap().to_string();
        println!("user_b: {}", &amount);
        assert_eq!(amount, "10".to_string());
    }

    //helper function
    fn _hash(data: String) -> String {
        use sha3::Digest;
        use sha3::Keccak256;
        let mut hasher = Keccak256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

fn main() {
    panic!()
}
