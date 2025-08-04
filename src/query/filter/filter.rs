use super::balance::BalanceFilter;
use super::instruction::InstructionFilter;
use super::token_balance::TokenBalanceFilter;
use super::transaction::TransactionFilter;
use crate::data::TransactionData;
use crate::query::filter::selected_items::SelectedItems;
use crate::query::SolanaQuery;


pub struct Filter {
    transaction: TransactionFilter,
    instruction: InstructionFilter,
    balance: BalanceFilter,
    token_balance: TokenBalanceFilter,
}


impl Filter {
    pub fn compile(query: SolanaQuery) -> Self {
        Self {
            transaction: TransactionFilter::new(query.transactions),
            instruction: InstructionFilter::new(query.instructions),
            balance: BalanceFilter::new(query.balances),
            token_balance: TokenBalanceFilter::new(query.token_balances)
        }
    }
    
    pub fn eval(&self, tx: &TransactionData) -> SelectedItems {
        let mut sel = SelectedItems::new_for_transaction(tx);
        
        if self.transaction.is_non_trivial() {
            self.transaction.eval(&mut sel, tx)
        }
        
        if self.instruction.is_non_trivial() {
            self.instruction.eval(&mut sel, tx)
        }
        
        if self.balance.is_non_trivial() {
            self.balance.eval(&mut sel, tx)
        }
        
        if self.token_balance.is_non_trivial() {
            self.token_balance.eval(&mut sel, tx)
        }
        
        sel
    }
}