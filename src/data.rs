use serde::Serialize;
use std::sync::Arc;


pub type AccountIndex = u8;
pub type ItemIndex = usize;
pub type Base58Bytes = String;
pub type JsonString = String;


pub enum DataMessage {
    Block(BlockData),
    Transaction(TransactionData)
}


pub struct BlockData {
    pub slot: u64,
    pub hash: Base58Bytes,
    pub parent_slot: u64,
    pub parent_hash: Base58Bytes,
    pub height: Option<u64>,
    pub timestamp: i64
}


pub type AccountList = Arc<[Base58Bytes]>;


pub struct TransactionData {
    pub slot: u64,
    pub transaction_index: ItemIndex,
    pub transaction: Transaction,
    pub instructions: Vec<Instruction>,
    pub balances: Vec<Balance>,
    pub token_balances: Vec<TokenBalance>,
    pub accounts: AccountList
}


pub struct Transaction {
    pub version: TransactionVersion,
    pub account_keys: usize,
    pub address_table_lookups: JsonString,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
    pub num_required_signatures: u8,
    pub recent_blockhash: Base58Bytes,
    pub signatures: JsonString,
    pub err: Option<JsonString>,
    pub compute_units_consumed: Option<u64>,
    pub fee: u64,
    pub loaded_addresses: JsonString,
}


#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionVersion {
    Legacy,
    #[serde(untagged)]
    Other(u8),
}


pub struct Instruction {
    pub instruction_address: Vec<ItemIndex>,
    pub program_id: AccountIndex,
    pub accounts: Vec<AccountIndex>,
    pub data: Base58Bytes,
    pub binary_data: Vec<u8>,
    pub is_committed: bool,
    pub account_list: AccountList
}


#[derive(Default, Clone)]
pub struct TokenBalance {
    pub account: Base58Bytes,
    pub pre_mint: Option<Base58Bytes>,
    pub post_mint: Option<Base58Bytes>,
    pub pre_decimals: Option<u32>,
    pub post_decimals: Option<u32>,
    pub pre_program_id: Option<Base58Bytes>,
    pub post_program_id: Option<Base58Bytes>,
    pub pre_owner: Option<Base58Bytes>,
    pub post_owner: Option<Base58Bytes>,
    pub pre_amount: Option<String>,
    pub post_amount: Option<String>,
}


pub struct Balance {
    pub account: Base58Bytes,
    pub pre: u64,
    pub post: u64
}