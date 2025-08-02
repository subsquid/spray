use super::item_filter::ItemFilter;
use super::relation_mask::relation_mask;
use super::selected_items::SelectedItems;
use crate::data::{Balance, TransactionData};
use crate::query::BalanceRequest;
use std::collections::HashSet;


relation_mask! {
    BalanceRelations {
        transaction,
        transaction_instructions,
    }
}


pub type PreparedBalanceRequest = ItemFilter<Balance, BalanceRelations>;


pub struct BalanceFilter {
    requests: Vec<PreparedBalanceRequest>
}


impl BalanceFilter {
    pub fn new(requests: Vec<PreparedBalanceRequest>) -> Self {
        Self {
            requests
        }
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn eval(&self, sel: &mut SelectedItems, tx: &TransactionData) {
        for (i, b) in tx.balances.iter().enumerate() {
            if let Some(rel) = ItemFilter::or(&self.requests, b) {
                sel.balances[i] = true;
                sel.transaction |= rel.has_transaction();
                sel.include_all_instructions |= rel.has_transaction_instructions();
            }
        }
    }
}


pub fn compile_balance_request(req: BalanceRequest) -> Option<PreparedBalanceRequest> {
    let mut filter = PreparedBalanceRequest::default();

    if let Some(list) = req.account {
        if list.is_empty() {
            return None
        }
        let set: HashSet<_> = list.into_iter().collect();
        filter.add(move |b| set.contains(&b.account));
    }

    filter.relations_mut().set_transaction(req.transaction);
    filter.relations_mut().set_transaction_instructions(req.transaction_instructions);
    
    Some(filter)
}