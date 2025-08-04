use super::item_filter::ItemFilter;
use super::relation_mask::relation_mask;
use super::selected_items::SelectedItems;
use crate::data::TransactionData;
use crate::query::TransactionRequest;
use std::collections::HashSet;


relation_mask! {
    TransactionRelations {
        instructions,
        logs,
        balances,
        token_balances,
    }
}


pub type PreparedTransactionRequest = ItemFilter<TransactionData, TransactionRelations>;


pub struct TransactionFilter {
    requests: Vec<PreparedTransactionRequest>
}


impl TransactionFilter {
    pub fn new(requests: Vec<TransactionRequest>) -> Self {
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
        let Some(rel) = ItemFilter::or(&self.requests, tx) else {
            return
        };
        sel.transaction = true;
        sel.instructions.add_all(rel.has_instructions());
        sel.balances.add_all(rel.has_balances());
        sel.token_balances.add_all(rel.has_token_balances());
    }
}


fn compile_request(req: TransactionRequest) -> Option<PreparedTransactionRequest> {
    let mut filter = PreparedTransactionRequest::default();

    if let Some(list) = req.fee_payer {
        if list.is_empty() {
            return None
        }
        let set: HashSet<_> = list.into_iter().collect();
        filter.add(move |tx| {
            tx.accounts.get(0).map_or(false, |a| set.contains(a))
        })
    }

    if let Some(list) = req.mentions_account {
        if list.is_empty() {
            return None
        }
        let set: HashSet<_> = list.into_iter().collect();
        filter.add(move |tx| {
            tx.accounts.iter().any(|a| set.contains(a))
        })
    }

    filter.relations_mut().set_instructions(req.instructions);
    filter.relations_mut().set_logs(req.logs);
    filter.relations_mut().set_balances(req.balances);
    filter.relations_mut().set_token_balances(req.token_balances);

    Some(filter)
}