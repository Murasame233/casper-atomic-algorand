from pyteal import App, Txn, Cond, Int, OnComplete, Return, Bytes, Seq, Assert, Keccak256, Gtxn, TxnType, And, Global, Add, Not, Mode, compileTeal


def atomic():
    # Name
    escrow = App.globalGet(Bytes("Escrow"))

    # Validator
    is_creator = Txn.sender() == App.globalGet(Bytes("Owner"))
    is_recipient = Txn.sender() == App.globalGet(Bytes("Recipient"))

    is_two_tx = Global.group_size() == Int(2)
    is_time_out = Add(App.globalGet(Bytes("Start")), App.globalGet(
        Bytes("End"))) > Global.latest_timestamp()

    optinpayment = Seq([Assert(Gtxn[1].receiver() == escrow)])
    # Helper Function
    start = Seq([
        Assert(Gtxn[1].asset_receiver() == escrow),
        App.globalPut(Bytes("Start"), Global.latest_timestamp()),
        App.globalPut(Bytes("amount"), Gtxn[1].asset_amount())
    ])

    # "Args: {End} {Hash} {Recipient PublicKey}"
    onCreation = Seq([
        App.globalPut(Bytes("Owner"), Txn.sender()),
        App.globalPut(Bytes("End"), Txn.application_args[1]),
        App.globalPut(Bytes("Hash"), Txn.application_args[2]),
        App.globalPut(Bytes("Recipient"), Txn.application_args[3]),
        Return(Int(1))
    ])

    # "Args: update {address publicKey}"
    update = Seq([
        Assert(is_creator),
        Assert(Txn.application_args.length() == Int(2)),
        App.globalPut(Bytes("Escrow"), Txn.application_args[1]),
        Return(Int(1))
    ])

    # "Args: fund"
    fund = Seq([
        # need owner create two transaction
        # - appcall: optin
        # - owner to escrow asset_transfer or payment
        Assert(is_two_tx),
        Assert(is_creator),

        # valid payment
        Cond(
            [Gtxn[1].type_enum() == TxnType.Payment, optinpayment],
            [Gtxn[1].type_enum() == TxnType.AssetTransfer, start]
        ),

        Return(Int(1))
    ])

    # "Args: optin"
    optin = Seq([
        # need owner create two transaction
        # - appcall: optin
        # - escrow to escrow optin
        Assert(is_two_tx),
        Assert(is_creator),

        # valid optin
        Assert(And(
            Gtxn[1].type_enum() == TxnType.AssetTransfer,
            Gtxn[1].asset_amount() == Int(0),
        )),

        Return(Int(1))
    ])

    # "Args: withdraw {secret}"
    withdraw = Seq([
        Assert(Not(is_time_out)),
        # This call only can be recipient
        Assert(is_recipient),
        Assert(Keccak256(Txn.application_args[1]) == App.globalGet(Bytes("Hash"))),

        # Valid Asset
        Assert(Gtxn[1].type_enum() == TxnType.AssetTransfer),
        Assert(Gtxn[1].asset_sender() == App.globalGet(Bytes("Escrow"))),
        Assert(Gtxn[1].asset_receiver() == App.globalGet(Bytes("Recipient"))),
        Assert(Gtxn[1].asset_amount() == App.globalGet(Bytes("amount"))),

        # update secret
        App.globalPut(Bytes("Secret"), Txn.application_args[1]),
        Return(Int(1)),
    ])

    # "Args: refund"
    refund = Seq([
        # need owner create two transaction
        # - appcall: optin
        # - owner to escrow asset_transfer or payment
        Assert(is_time_out),
        Assert(is_two_tx),
        Assert(is_creator),

        # Valid Asset
        Assert(Gtxn[1].type_enum() == TxnType.AssetTransfer),
        Assert(Gtxn[1].asset_receiver() == App.globalGet(Bytes("Owner"))),
        Assert(Gtxn[1].asset_sender() == App.globalGet(Bytes("Escrow"))),

        Return(Int(1))
    ])

    program = Cond(
        [Txn.application_id() == Int(0), onCreation],
        [Txn.on_completion() == OnComplete.DeleteApplication, Return(is_creator)],
        [Txn.on_completion() == OnComplete.UpdateApplication, Return(is_creator)],
        [Txn.application_args[0] == Bytes("update"), update],
        [Txn.application_args[0] == Bytes("fund"), fund],
        [Txn.application_args[0] == Bytes("optin"), optin],
        [Txn.application_args[0] == Bytes("withdraw"), withdraw],
        [Txn.application_args[0] == Bytes("refund"), refund],
    )

    return program


def escrow(app_id):
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
    compiled = compileTeal(escrow(123456), Mode.Application, version=4)
    f.write(compiled)

with open('clear.teal', 'w') as f:
    compiled = compileTeal(clear_program(), Mode.Application, version=4)
    f.write(compiled)
