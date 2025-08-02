use super::balance::BalanceRelations;
use super::item_filter::ItemFilter;
use super::selected_items::SelectedItems;
use crate::data::{TokenBalance, TransactionData};
use crate::query::TokenBalanceRequest;
use std::collections::HashSet;


pub type PreparedTokenBalanceRequest = ItemFilter<TokenBalance, BalanceRelations>;


pub struct TokenBalanceFilter {
    requests: Vec<PreparedTokenBalanceRequest>
}


impl TokenBalanceFilter {
    pub fn new(requests: Vec<PreparedTokenBalanceRequest>) -> Self {
        Self {
            requests
        }
    }

    pub fn eval(&self, sel: &mut SelectedItems, tx: &TransactionData) {
        for (i, b) in tx.token_balances.iter().enumerate() {
            if let Some(rel) = ItemFilter::or(&self.requests, b) {
                sel.balances[i] = true;
                sel.transaction |= rel.has_transaction();
                sel.include_all_instructions |= rel.has_transaction_instructions();
            }
        }
    }
}


pub fn compile_token_balance_request(req: TokenBalanceRequest) -> Option<PreparedTokenBalanceRequest> {
    let mut filter = PreparedTokenBalanceRequest::default();

    if let Some(list) = req.account {
        if list.is_empty() {
            return None
        }
        let set: HashSet<_> = list.into_iter().collect();
        filter.add(move |b| set.contains(&b.account));
    }

    macro_rules! in_opt_list {
        ($name:ident) => {
            if let Some(list) = req.$name {
                if list.is_empty() {
                    return None
                }
                let set: HashSet<_> = list.into_iter().collect();
                filter.add(move |b| b.$name.as_ref().map_or(false, |v| set.contains(v)));
            }
        };
    }
    in_opt_list!(pre_mint);
    in_opt_list!(post_mint);
    in_opt_list!(pre_program_id);
    in_opt_list!(post_program_id);
    in_opt_list!(pre_owner);
    in_opt_list!(post_owner);

    filter.relations_mut().set_transaction(req.transaction);
    filter.relations_mut().set_transaction_instructions(req.transaction_instructions);
    
    Some(filter)
}