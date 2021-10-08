import base64
import os
from algosdk import account, v2client
from algosdk.future.transaction import ApplicationNoOpTxn, LogicSig, LogicSigTransaction, OnComplete, StateSchema, PaymentTxn, ApplicationCreateTxn, assign_group_id
from algosdk.v2client.algod import AlgodClient
from dotenv import load_dotenv
from helper import group_two, hash_helper, sleep_sec

load_dotenv()


class Algorand:
    # Client
    client: AlgodClient

    # Accounts
    Account_A_Key = None
    Account_A_Addre = None
    Account_B_Key = None
    Account_B_Addre = None

    # Pass
    secret = None
    hash = None

    # file base
    base = None

    # Program
    atomic = None
    clear = None
    escrow = None

    # program schema
    global_schema = StateSchema(3, 5)
    local_schema = StateSchema(0, 0)

    appid = None

    def __init__(self) -> None:
        self.base = os.path.dirname(os.path.abspath(__file__))
        self.algo_client()
        self.two_account()
        print("faucet: https://bank.testnet.algorand.network\naccount_a: {}\naccount_b: {}".format(
            self.Account_A_Addre, self.Account_B_Addre))

    def algo_client(self):
        algod_token = os.environ.get("PURESTACK")
        headers = {"X-API-Key": algod_token}
        self.client = v2client.algod.AlgodClient(
            algod_token, os.environ.get("ALGONET"), headers)

    def two_account(self):
        self.Account_A_Key, self.Account_A_Addre = account.generate_account()
        self.Account_B_Key, self.Account_B_Addre = account.generate_account()

    # set_secret

    def set_secret(self, secret: str):
        self.secret = secret
        self.hash = hash_helper(self.secret)

    def set_hash(self, hash: str):
        self.hash = hash
        self.secret = None

    # Deploy
    def deploy_atomic(self):
        # compile
        self.compile_atomic()
        self.compile_clear()

        # create
        create = ApplicationCreateTxn(self.Account_A_Addre,
                                      self.suggest_param(),
                                      OnComplete.NoOpOC.real,
                                      self.atomic, self.clear,
                                      self.global_schema,
                                      self.local_schema,
                                      [
                                          (3000).to_bytes(8, "big"),
                                          self.hash.encode("UTF-8"),
                                          self.address_for_arg(
                                              self.Account_B_Addre)
                                      ])
        signed_create = create.sign(self.Account_A_Key)
        txid = self.client.send_transaction(signed_create)
        sleep_sec(10)
        self.appid = self.client.pending_transaction_info(txid)[
            "application-index"]

        self.compile_escrow()

        # update
        update = ApplicationNoOpTxn(
            self.Account_A_Addre,
            self.suggest_param(),
            self.appid,
            [
                "update".encode("UTF-8"),
                self.address_for_arg(self.escrow.address())
            ]
        )
        signed_udpate = update.sign(self.Account_A_Key)
        self.client.send_transaction(signed_udpate)
        sleep_sec(10)
        # fund
        fund = ApplicationNoOpTxn(
            self.Account_A_Addre,
            self.suggest_param(),
            self.appid,
            [
                "fund".encode("UTF-8"),
                (50 * 100000).to_bytes(8, "big")
            ]
        )
        fund_pay = self.payment(
            self.Account_A_Addre,
            self.escrow.address(),
            50*100000 + 1000
        )

        signed_txs = group_two(
            [fund, fund_pay], [self.Account_A_Key, self.Account_B_Key])
        self.client.send_transactions(signed_txs)

    def withdraw(self, secret: str):
        withdraw = ApplicationNoOpTxn(self.Account_B_Addre, self.suggest_param(), self.appid,
                                      [
            "withdraw".encode("UTF-8"),
            secret.encode("UTF-8")
        ])
        withdraw_pay = self.payment(self.escrow.address(
        ), self.Account_B_Addre, 50 * 100000, self.Account_A_Addre)
        [withdraw, withdraw_pay] = assign_group_id([withdraw, withdraw_pay])
        signed_withdraw = withdraw.sign(self.Account_B_Key)
        signed_withdraw_pay = LogicSigTransaction(withdraw_pay, self.escrow)

        self.client.send_transactions([signed_withdraw, signed_withdraw_pay])
    # Compiles

    def compile_atomic(self):
        keccak256 = bytes.fromhex(self.hash)
        program = open(self.base+"/contract/atomic.teal", "r").read().replace(
            "3H==", str(base64.b64encode(keccak256).decode("utf-8")), -1)
        self.atomic = self.compile_contract(program)

    def compile_clear(self):
        self.clear = self.compile_contract(
            open(self.base+"/contract/clear.teal", "r").read())

    def compile_escrow(self):
        escrow = open(self.base+"/contract/escrow.teal",
                      "r").read().replace("123456", str(self.appid))
        self.escrow = LogicSig(self.compile_contract(escrow))

    # helpers

    def suggest_param(self):
        return self.client.suggested_params()

    def payment(self, sender, to, amount, close=None):
        return PaymentTxn(sender, self.suggest_param(), to, amount, close)

    def address_for_arg(self, address):
        from algosdk import encoding
        return encoding.decode_address(address)

    def compile_contract(self, contract: str):
        return base64.b64decode(self.client.compile(contract)["result"])
