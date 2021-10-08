from pyteal import App, Txn, Cond, Int, OnComplete, Return, Bytes, Seq, Assert, Keccak256, Gtxn, TxnType, And, Global, Add, Not, Mode, compileTeal,Btoi


def atomic():
    ## Global state
    ## - Escrow bytes
    ## - Owner bytes
    ## - Recipient bytes
    ## - Start uint
    ## - End uint
    ## - Hash bytes
    ## - Secret bytes
    ## - Amount uint

    # Name
    escrow = App.globalGet(Bytes("Escrow"))

    # Validator
    is_creator = Txn.sender() == App.globalGet(Bytes("Owner"))
    is_recipient = Txn.sender() == App.globalGet(Bytes("Recipient"))

    is_two_tx = Global.group_size() == Int(2)
    is_time_out = Add(App.globalGet(Bytes("Start")), App.globalGet(
        Bytes("End"))) < Global.latest_timestamp()

    # "Args: {End} {Hash} {Recipient PublicKey}"
    onCreation = Seq([
        App.globalPut(Bytes("Owner"), Txn.sender()),
        App.globalPut(Bytes("End"), Btoi(Txn.application_args[0])),
        App.globalPut(Bytes("Hash"), Txn.application_args[1]),
        App.globalPut(Bytes("Recipient"), Txn.application_args[2]),
        Return(Int(1))
    ])

    # "Args: update {address publicKey}"
    update = Seq([
        Assert(is_creator),
        Assert(Txn.application_args.length() == Int(2)),
        App.globalPut(Bytes("Escrow"), Txn.application_args[1]),
        Return(Int(1))
    ])

    # "Args: fund {amount}"
    fund = Seq([
        # need owner create two transaction
        # - appcall: optin
        # - owner to escrow payment
        Assert(is_two_tx),
        Assert(is_creator),

        # valid payment
        Assert(Gtxn[1].type_enum() == TxnType.Payment),
        Assert(Gtxn[1].receiver() == escrow),
        
        # Add mount
        App.globalPut(Bytes("Amount"),Btoi(Txn.application_args[1])),
App.globalPut(Bytes("Start"),Global.latest_timestamp()),
        Return(Int(1))
    ])

    # "Args: withdraw {secret}"
    withdraw = Seq([
        Assert(Not(is_time_out)),
        # This call only can be recipient
        Assert(is_recipient),
        # '3H==' is a placeholder
        Assert(Keccak256(Txn.application_args[1])== Bytes("base64","3H==")),

        # Valid Payment
        Assert(Gtxn[1].type_enum() == TxnType.Payment),
        Assert(Gtxn[1].sender() == App.globalGet(Bytes("Escrow"))),
        Assert(Gtxn[1].receiver() == App.globalGet(Bytes("Recipient"))),
        Assert(Gtxn[1].amount() == App.globalGet(Bytes("Amount"))),

        # update secret
        App.globalPut(Bytes("Secret"), Txn.application_args[1]),
        Return(Int(1)),
    ])

    # "Args: refund"
    refund = Seq([
        # need owner create two transaction
        # - appcall: refund
        # - escrow to owner payment
        Assert(is_time_out),
        Assert(is_two_tx),
        Assert(is_creator),

        # Valid Payment
        Assert(Gtxn[1].type_enum() == TxnType.Payment),
        Assert(Gtxn[1].receiver() == App.globalGet(Bytes("Owner"))),
        Assert(Gtxn[1].sender() == App.globalGet(Bytes("Escrow"))),

        Return(Int(1))
    ])

    program = Cond(
        [Txn.application_id() == Int(0), onCreation],
        [Txn.on_completion() == OnComplete.DeleteApplication, Return(is_creator)],
        [Txn.on_completion() == OnComplete.UpdateApplication, Return(is_creator)],
        [Txn.application_args[0] == Bytes("update"), update],
        [Txn.application_args[0] == Bytes("fund"), fund],
        [Txn.application_args[0] == Bytes("withdraw"), withdraw],
        [Txn.application_args[0] == Bytes("refund"), refund],
    )

    return program


def escrow(app_id):
    # app_id is a placeholder
    is_two_tx = Global.group_size() == Int(2)
    is_appcall = Gtxn[0].type_enum() == TxnType.ApplicationCall
    is_appid = Gtxn[0].application_id() == Int(app_id)
    acceptable_app_call = Gtxn[0].on_completion() == OnComplete.NoOp
    no_rekey = And(
        Gtxn[0].rekey_to() == Global.zero_address(),
        Gtxn[1].rekey_to() == Global.zero_address()
    )
    return And(
        is_two_tx,
        is_appcall,
        is_appid,
        acceptable_app_call,
        no_rekey,
    )


def clear_program():
    return Return(Int(1))


with open('atomic.teal', 'w') as f:
    compiled = compileTeal(atomic(), Mode.Application, version=4)
    f.write(compiled)

with open('escrow.teal', 'w') as f:
    # 123456 is a placeholder
    compiled = compileTeal(escrow(123456), Mode.Application, version=4)
    f.write(compiled)

with open('clear.teal', 'w') as f:
    compiled = compileTeal(clear_program(), Mode.Application, version=4)
    f.write(compiled)
