#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{PubkeyMatch, RuleSetV2},
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn test_pubkey_match_v2() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let target = Keypair::new();

    let rule =
        PubkeyMatch::serialize(PayloadKey::Destination.to_string(), target.pubkey()).unwrap();

    // Create a RuleSet.
    let rule_set = RuleSetV2::serialize(
        context.payer.pubkey(),
        "test rule_set",
        &[Operation::Transfer {
            scenario: utils::TransferScenario::Holder,
        }
        .to_string()],
        &[&rule],
    )
    .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain_serialized!(&mut context, rule_set, "test rule_set".to_string())
            .await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition with WRONG Pubkey.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(Keypair::new().pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::PubkeyMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store the payload of data to validate against the rule definition with CORRECT Pubkey.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(target.pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
