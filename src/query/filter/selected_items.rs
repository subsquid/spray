use crate::data::TransactionData;


#[derive(Debug)]
pub struct SelectedItems {
    pub transaction: bool,
    pub instructions: Vec<bool>,
    pub balances: Vec<bool>,
    pub token_balances: Vec<bool>,
    pub include_all_instructions: bool,
    pub include_all_balances: bool,
    pub include_all_token_balances: bool
}


impl SelectedItems {
    pub fn new_for_transaction(tx: &TransactionData) -> Self {
        Self {
            transaction: false,
            instructions: vec![false; tx.instructions.len()],
            balances: vec![false; tx.balances.len()],
            token_balances: vec![false; tx.token_balances.len()],
            include_all_instructions: false,
            include_all_balances: false,
            include_all_token_balances: false
        }
    }
}