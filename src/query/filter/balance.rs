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
    pub fn new(requests: Vec<BalanceRequest>) -> Self {
        let requests = requests
            .into_iter()
            .filter_map(compile_request)
            .collect();
        
        Self {
            requests
        }
    }

    pub fn is_non_trivial(&self) -> bool {
        !self.requests.is_empty()
    }

    pub fn eval(&self, sel: &mut SelectedItems, tx: &TransactionData) {
        for (i, b) in tx.balances.iter().enumerate() {
            if let Some(rel) = ItemFilter::or(&self.requests, b) {
                sel.balances.add(i);
                sel.transaction |= rel.has_transaction();
                sel.instructions.add_all(rel.has_transaction_instructions());
            }
        }
    }
}


fn compile_request(req: BalanceRequest) -> Option<PreparedBalanceRequest> {
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