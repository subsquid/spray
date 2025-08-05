use anyhow::ensure;
use crate::query::util::{field_selection, item_field_selection, request};
use serde::{Deserialize, Serialize};


field_selection! {
    block: BlockFieldSelection,
    transaction: TransactionFieldSelection,
    instruction: InstructionFieldSelection,
    balance: BalanceFieldSelection,
    token_balance: TokenBalanceFieldSelection,
}


item_field_selection! {
    BlockFieldSelection {
        number,
        hash,
        parent_number,
        parent_hash,
        height,
        timestamp,
    }

    TransactionFieldSelection {
        transaction_index,
        version,
        account_keys,
        address_table_lookups,
        num_readonly_signed_accounts,
        num_readonly_unsigned_accounts,
        num_required_signatures,
        recent_blockhash,
        signatures,
        err,
        fee,
        compute_units_consumed,
        loaded_addresses,
        fee_payer,
        has_dropped_log_messages,
    }

    InstructionFieldSelection {
        transaction_index,
        instruction_address,
        program_id,
        accounts,
        data,
        d1,
        d2,
        d4,
        d8,
        error,
        compute_units_consumed,
        is_committed,
        has_dropped_log_messages,
    }
    
    BalanceFieldSelection {
        transaction_index,
        account,
        pre,
        post,
    }
    
    TokenBalanceFieldSelection {
        transaction_index,
        account,
        pre_mint,
        post_mint,
        pre_decimals,
        post_decimals,
        pre_program_id,
        post_program_id,
        pre_owner,
        post_owner,
        pre_amount,
        post_amount,
    }
}


pub type Base58Bytes = String;
pub type Bytes = String;


request! {
    pub struct TransactionRequest {
        pub fee_payer: Option<Vec<Base58Bytes>>,
        pub mentions_account: Option<Vec<Base58Bytes>>,
        pub instructions: bool,
        pub logs: bool,
        pub balances: bool,
        pub token_balances: bool,
    }

    pub struct InstructionRequest {
        pub program_id: Option<Vec<Base58Bytes>>,
        pub discriminator: Option<Vec<Bytes>>,
        pub d1: Option<Vec<Bytes>>,
        pub d2: Option<Vec<Bytes>>,
        pub d4: Option<Vec<Bytes>>,
        pub d8: Option<Vec<Bytes>>,
        pub mentions_account: Option<Vec<Base58Bytes>>,
        pub a0: Option<Vec<Base58Bytes>>,
        pub a1: Option<Vec<Base58Bytes>>,
        pub a2: Option<Vec<Base58Bytes>>,
        pub a3: Option<Vec<Base58Bytes>>,
        pub a4: Option<Vec<Base58Bytes>>,
        pub a5: Option<Vec<Base58Bytes>>,
        pub a6: Option<Vec<Base58Bytes>>,
        pub a7: Option<Vec<Base58Bytes>>,
        pub a8: Option<Vec<Base58Bytes>>,
        pub a9: Option<Vec<Base58Bytes>>,
        pub a10: Option<Vec<Base58Bytes>>,
        pub a11: Option<Vec<Base58Bytes>>,
        pub a12: Option<Vec<Base58Bytes>>,
        pub a13: Option<Vec<Base58Bytes>>,
        pub a14: Option<Vec<Base58Bytes>>,
        pub a15: Option<Vec<Base58Bytes>>,
        pub is_committed: Option<bool>,
        pub transaction: bool,
        pub transaction_balances: bool,
        pub transaction_token_balances: bool,
        pub transaction_instructions: bool,
        pub inner_instructions: bool,
        pub parent_instructions: bool,
        pub logs: bool,
    }

    pub struct TokenBalanceRequest {
        pub account: Option<Vec<Base58Bytes>>,
        pub pre_mint: Option<Vec<Base58Bytes>>,
        pub post_mint: Option<Vec<Base58Bytes>>,
        pub pre_program_id: Option<Vec<Base58Bytes>>,
        pub post_program_id: Option<Vec<Base58Bytes>>,
        pub pre_owner: Option<Vec<Base58Bytes>>,
        pub post_owner: Option<Vec<Base58Bytes>>,
        pub transaction: bool,
        pub transaction_instructions: bool,
    }

    pub struct BalanceRequest {
        pub account: Option<Vec<Base58Bytes>>,
        pub transaction: bool,
        pub transaction_instructions: bool,
    }

    pub struct SolanaQuery {
        pub fields: FieldSelection,
        pub include_all_blocks: bool,
        pub transactions: Vec<TransactionRequest>,
        pub instructions: Vec<InstructionRequest>,
        pub balances: Vec<BalanceRequest>,
        pub token_balances: Vec<TokenBalanceRequest>,
    }
}


impl SolanaQuery {
    pub fn validate(&self) -> anyhow::Result<()> {
        let num_items = self.transactions.len() 
            + self.instructions.len() 
            + self.balances.len() 
            + self.token_balances.len();

        ensure!(
            num_items <= 100,
            "query contains {} item requests, but only 100 is allowed",
            num_items
        );

        Ok(())
    }
}