# Intro

This document will describe the principles.

# describe

## how it be safe

### time

- user create smart contract with time limit
- if timeout they can refund their money
- when time is not time out, user cannot get back money

### recipient

- all contract specify the recipient that although other people know the secret they cannot get the money

### hash

- user_b must waits user_a to input the secret to valid hash and get the secret.

# Simply conclude

Just a HTLC(Hash Time Lock Contract) implement on the casper chain.

# Prepare

- User_a: a trader trade with user_b
- User_b: a trader trade with user_a
- User_t: create token on casper

All usre must have `casper` on their Casper wallet and `Algo` on their Algorand wallet for smart contract call.

A secret for user_a validate.

# Step

## Deploy token on casper

> Var:
>
> - `Token Contract Hash`

1. Follow the official guid to do this with User_t.
2. Get the `Token Contract Hash`
3. Send some Token to user_a

## User A deploy atomic swap

> Var:
>
> - `hash`
> - `atomic contract hash`

1. With the compiled contract and this session args
   > - token (`Token Contract Hash`)
   > - amount (`U256`)
   > - end (`u64`, time limit, **ms**, not the end time, just how long this swap is valid)
   > - recipient (`AccountHash`)
   > - hash (`String`, user_a's secret to hash)
2. Deploy it, it will automatically transfer the token that user_t created to contract.
3. User_b ask the user_a for `atomic contract hash`.
4. User_b query atomic swap contract global state to get the user_a's secret `hash`

## User B deploy atomic swap

> Var:
>
> - `app_id`

1. Replace `3H==` placeholder on the `atomic.teal` file to the hash
2. Compile it
3. Deploy it with these args (Must be **same order** as below) and get `app_id`
   > `End`: time with sec \
   > `hash`: user_a secret `hash` \
   > `Recipient`: use algorand encoding.decode_address \
   > `{End} {Hash} {Recipient}`
4. Replace `123456` placeholder on the `escrow.teal` file to the `app_id`
5. compile get address.
6. update application's escrow address with args `update escrow address`
7. fund this address with algo
8. tell user_a `app_id`

## With draw

### user_a withdraw

1. call algorand app with `app_id` and args `withdraw {secret}`

### user_b withdraw

1. user_b query algorand app state for get the real secret and call the `atomic contract hash` entrypoint `withdraw` with args `{secret}` to get the money
