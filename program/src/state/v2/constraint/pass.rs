use solana_program::msg;

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, HEADER_SECTION},
    state::{Header, RuleResult},
};

/// Constraint representing an operation that always succeeds.
pub struct Pass;

impl<'a> Pass {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(_bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        Ok(Self {})
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize() -> Result<Vec<u8>, RuleSetError> {
        let mut data = Vec::with_capacity(HEADER_SECTION);
        // Header
        Header::serialize(ConstraintType::Pass, 0, &mut data);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Pass {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Pass
    }

    fn validate(
        &self,
        _accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        _payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Pass");
        RuleResult::Success(self.constraint_type().to_error())
    }
}
