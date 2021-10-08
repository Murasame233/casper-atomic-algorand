from time import sleep
from Crypto.Hash import keccak
from algosdk.future.transaction import assign_group_id

def sleep_sec(sec):
    print("wait for {} sec".format(sec))
    sleep(sec)

def hash_helper(data:str):
    return keccak.new(data=bytes(data,"UTF-8"), digest_bits=256).digest().hex()

def group_two(txs,signer):
    [tx1,tx2] = assign_group_id(txs)
    signed_tx1 = tx1.sign(signer[0])
    signed_tx2 = tx2.sign(signer[0])
    return [signed_tx1,signed_tx2]
    
