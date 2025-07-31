use crate::data::TransactionData;


#[derive(Debug)]
pub struct SelectedItems {
    pub transaction: bool,
    pub instructions: Vec<bool>,
    pub include_all_instructions: bool
}


impl SelectedItems {
    pub fn new_for_transaction(tx: &TransactionData) -> Self {
        Self {
            transaction: false,
            instructions: vec![false; tx.instructions.len()],
            include_all_instructions: false
        }
    }
}