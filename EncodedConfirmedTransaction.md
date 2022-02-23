```
EncodedConfirmedTransaction
{
    slot: 3,
    transaction: EncodedTransactionWithStatusMeta
        { transaction: Json(UiTransaction
            {
                signatures: [
                "3TqHts8pT4GxGPNGAJRk5yWvDy2SzJds5T4sDfX56AJ5pYEMRQZsiVFSYEM4YtodCbu7MBUXwMwV2FtjEWpkVAaY"
            ],
                message: Raw(UiRawMessage
                    {
                        header: MessageHeader
                            {
                                num_required_signatures: 1,
                                num_readonly_signed_accounts: 0,
                                num_readonly_unsigned_accounts: 1
                },
                account_keys: [
                    "7QJxe5EJuSKBxie6W8GZiq3HuCdynaDcJHfusw8c7dUT",
                    "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM"
                ],
                recent_blockhash: "HuqGALzrZi3vZAu8Ao16C4p4STUnfMYGjeUXc6VTaLti",
                instructions: [UiCompiledInstruction
                            {
                                program_id_index: 1,
                                accounts: [
                            0
                        ],
                                data: "143NXVKNwtPXZrZSRpjm91FtKmGjVoD5Rp5Ev7GpnHm3w4hM6gYbrvJASxv5BFzZnfVzzD19mupmp6G2DSzzafhr9duR4gqNt1JWwCiaawRNeBjSvUChz3irQiB2iWb4vsmhb"
                    }
                ]
            })
        }),
        meta: Some(UiTransactionStatusMeta
                    {
                        err: None,
                        status: Ok(()),
                        fee: 5000,
                        pre_balances: [
                500000000000000000,
                1
            ],
                        post_balances: [
                499999999999995000,
                1
            ],
                        inner_instructions: Some([]),
                        log_messages: Some(
                            [
                "Program 4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM invoke [1]",
                "Program log: Entry point solana_keri with signer 7QJxe5EJuSKBxie6W8GZiq3HuCdynaDcJHfusw8c7dUT",
                "Program log: Processing DID:SOL:KERI Inception",
                "Program log: Valdated DID Reference {\"i\": \"did:sol:keri:ERbbDrguW5HFrAJ98xisQpdauOpTykaHlTNIO7N0BEbE\", \"ri\": \"did:keri:local_db\"}",
                "Program 4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM consumed 39932 of 1400000 compute units",
                "Program 4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM success"
            ]),
                        pre_token_balances: Some([]),
                        post_token_balances: Some([]),
                        rewards: Some([])
        })
    },
    block_time: Some(1645658017)
}
```