use borsh::BorshSerialize;
use solana_program::{msg, program_error::ProgramError};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, RuleV2, HEADER_SECTION, U64_BYTES},
    state::{try_from_bytes, RuleResult},
};

/// Constraint representing a group OR, where at least one rule contained must pass.
pub struct Any<'a> {
    /// The number of rules contained under Any.
    pub size: &'a u64,
    /// The vector of Rules contained under Any.
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> Any<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let size = try_from_bytes::<u64>(0, U64_BYTES, bytes)?;

        let mut rules = Vec::with_capacity(*size as usize);
        let mut offset = U64_BYTES;

        for _ in 0..*size {
            let rule = RuleV2::from_bytes(&bytes[offset..])?;
            offset += rule.length();
            rules.push(rule);
        }

        Ok(Self { size, rules })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(rules: &[&[u8]]) -> std::io::Result<Vec<u8>> {
        let length = (U64_BYTES
            + rules
                .iter()
                .map(|v| v.len())
                .reduce(|accum, item| accum + item)
                .ok_or(RuleSetError::DataIsEmpty)
                .unwrap()) as u32;

        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::Any as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - size
        let size = rules.len() as u64;
        BorshSerialize::serialize(&size, &mut data)?;
        // - rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Any<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Any
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Any");

        let mut last_failure: Option<ProgramError> = None;
        let mut last_error: Option<ProgramError> = None;

        for rule in &self.rules {
            let result = rule.validate(
                accounts,
                payload,
                update_rule_state,
                rule_set_state_pda,
                rule_authority,
            );

            match result {
                RuleResult::Success(_) => return result,
                RuleResult::Failure(err) => last_failure = Some(err),
                RuleResult::Error(err) => last_error = Some(err),
            }
        }

        // Return failure if and only if all rules failed.  Use the last failure.
        if let Some(err) = last_failure {
            RuleResult::Failure(err)
        } else if let Some(err) = last_error {
            // Return invalid if and only if all rules were invalid.  Use the last invalid.
            RuleResult::Error(err)
        } else {
            RuleResult::Error(RuleSetError::UnexpectedRuleSetFailure.into())
        }
    }
}
