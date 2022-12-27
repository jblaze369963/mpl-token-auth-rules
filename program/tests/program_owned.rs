#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{Rule, RuleSet},
};
use solana_program_test::tokio;
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};
use utils::{
    assert_rule_set_error, create_rule_set_on_chain, process_failing_validate_ix,
    process_passing_validate_ix, program_test, Operation, PayloadKey,
};

#[tokio::test]
async fn program_owned() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.  The target must be owned by the program ID specified in the Rule.
    let rule = Rule::ProgramOwned {
        program: mpl_token_metadata::id(),
        field: PayloadKey::Target.to_string(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set.add(Operation::Transfer.to_string(), rule).unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create an account owned by token-metadata to simulate a Token-Owned Escrow account.
    let fake_token_metadata_owned_escrow = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &fake_token_metadata_owned_escrow.pubkey(),
            rent.minimum_balance(0),
            0,
            &mpl_token_metadata::id(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_token_metadata_owned_escrow],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition.
    // In this case the Target will be used to look up the `AccountInfo`
    // and see who the owner is.  Here we put in the WRONG Pubkey.
    let wrong_rule_account = Keypair::new();
    let payload = Payload::from([(
        PayloadKey::Target.to_string(),
        PayloadType::Pubkey(wrong_rule_account.pubkey()),
    )]);

    // We also pass the WRONG account as an additional rule account.
    // It will be found by the Rule but will not be the owner.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            wrong_rule_account.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::ProgramOwnedCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // This time put the CORRECT Pubkey into the Payload and the validate instruction.
    let payload = Payload::from([(
        PayloadKey::Target.to_string(),
        PayloadType::Pubkey(fake_token_metadata_owned_escrow.pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            fake_token_metadata_owned_escrow.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;
}
