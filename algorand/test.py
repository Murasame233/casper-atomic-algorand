import base64
import os
from time import sleep
from algosdk import account, v2client, mnemonic, encoding
import algosdk
from algosdk.future.transaction import ApplicationCreateTxn, ApplicationNoOpTxn, LogicSigAccount, OnComplete, StateSchema, SuggestedParams, LogicSig, PaymentTxn, assign_group_id
from algosdk.transaction import LogicSigTransaction
from dotenv import load_dotenv
import json
from Crypto.Hash import keccak

load_dotenv()

# new account
private_key_a, public_address_a = account.generate_account()
mnemonic_a: str = mnemonic.from_private_key(private_key_a)
private_key_b, public_address_b = account.generate_account()
mnemonic_b: str = mnemonic.from_private_key(private_key_b)
print("Public Algorand Address a: {}\n".format(
    public_address_a))
print("Public Algorand Address b: {}\n".format(
    public_address_b))
print("Please use faucet: https://bank.testnet.algorand.network")
input()

base = os.path.dirname(os.path.abspath(__file__))

keccak256 = keccak.new(data=b'wow', digest_bits=256).digest()

atomic = open(base+"/contract/atomic.teal", "r").read().replace("3H==",str(base64.b64encode(keccak256).decode("utf-8")))
clear = open(base+"/contract/clear.teal", "r").read()

algod_token = os.environ.get("PURESTACK")
headers = {
    "X-API-Key": algod_token,
}

client = v2client.algod.AlgodClient(
    algod_token, os.environ.get("ALGONET"), headers)

print("wait for 5 sec")
sleep(5)

account_info = client.account_info(public_address_a)

print("now amount: {}".format(account_info['amount']))

compiled_atomic = base64.b64decode(client.compile(atomic)["result"])
compiled_clear = base64.b64decode(client.compile(clear)["result"])

pub = mnemonic.to_public_key(mnemonic_b)



create = ApplicationCreateTxn(
    public_address_a,
    client.suggested_params(),
    OnComplete.NoOpOC.real,
    compiled_atomic,
    compiled_clear,
    StateSchema(3, 5),
    StateSchema(0, 0),
    [
        (1800000).to_bytes(8, 'big'),
        keccak256.hex().encode("UTF-8"),
        encoding.decode_address(public_address_b)
    ]
)

signed_create = create.sign(private_key_a)
tx_id = signed_create.transaction.get_txid()

client.send_transaction(signed_create)

print("wait for 10 sec")
sleep(10)
transaction_response = client.pending_transaction_info(tx_id)
app_id = transaction_response['application-index']
print("Created new app-id: ", app_id)


# Update Escrow
escrow = open(base+"/contract/escrow.teal",
              "r").read().replace("123456", str(app_id))

compiled_escrow = LogicSig(
    base64.b64decode(client.compile(escrow)["result"]))
print(compiled_escrow.address())
escrow_address = compiled_escrow.address()

update = ApplicationNoOpTxn(public_address_a, client.suggested_params(), app_id, [
    "update".encode("UTF-8"), encoding.decode_address(escrow_address)])

signed_update = update.sign(private_key_a)

update_tx_id = signed_update.transaction.get_txid()

client.send_transaction(signed_update)


fund = ApplicationNoOpTxn(public_address_a, client.suggested_params(), app_id, [
    "fund".encode("UTF-8"), (50*100000).to_bytes(8, 'big'), ])

fund_pay = PaymentTxn(
    public_address_a, client.suggested_params(), escrow_address, 50*100000+1000)
[fund, fund_pay] = assign_group_id([fund, fund_pay])

signed_fund = fund.sign(private_key_a)
signed_fund_pay = fund_pay.sign(private_key_a)

tx_id = client.send_transactions([signed_fund, signed_fund_pay])
print(tx_id)

print("wait for 5 sec")
sleep(5)

withdraw = ApplicationNoOpTxn(public_address_b, client.suggested_params(), app_id, [
                              "withdraw".encode("UTF-8"), "wow".encode("UTF-8")])

withdraw_pay = PaymentTxn(
    escrow_address, client.suggested_params(), public_address_b, 50*100000,public_address_a)
[withdraw, withdraw_pay] = assign_group_id([withdraw, withdraw_pay])
signed_withdraw = withdraw.sign(private_key_b)
signed_withdraw_pay = LogicSigTransaction(withdraw_pay, compiled_escrow)

print("wait for 5 sec")
sleep(5)

tx_id = client.send_transactions([signed_withdraw, signed_withdraw_pay])
print(tx_id)
