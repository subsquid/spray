use super::item_filter::{ItemFilter, Relations};
use super::relation_mask::relation_mask;
use super::selected_items::SelectedItems;
use crate::data::{Instruction, TransactionData};
use crate::query::util::parse_hex;
use crate::query::InstructionRequest;
use std::collections::HashSet;


relation_mask! {
    InstructionRelations {
        transaction_balances,
        transaction_token_balances,
        transaction_instructions,
        inner_instructions,
        parent_instructions,
        logs,
    }
}


pub type PreparedInstructionRequest = ItemFilter<Instruction, InstructionRelations>;


pub struct InstructionFilter {
    requests: Vec<PreparedInstructionRequest>
}


impl InstructionFilter {
    pub fn new(requests: Vec<PreparedInstructionRequest>) -> Self {
        Self {
            requests
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn eval(&self, sel: &mut SelectedItems, tx: &TransactionData) {
        for (i, ins) in tx.instructions.iter().enumerate() {
            if let Some(rel) = ItemFilter::or(&self.requests, ins) {
                sel.instructions[i] = true;
                sel.include_all_instructions |= rel.has_transaction_instructions();
                sel.include_all_balances |= rel.has_transaction_balances();
                sel.include_all_token_balances |= rel.has_transaction_token_balances();
                
                if !sel.include_all_instructions && rel.has_inner_instructions() {
                    Self::eval_inner_instructions(sel, tx, i);
                }

                if !sel.include_all_instructions && rel.has_parent_instructions() {
                    Self::eval_parent_instructions(sel, tx, i);
                }
            }
        }
    }

    fn eval_inner_instructions(
        sel: &mut SelectedItems,
        tx: &TransactionData,
        instruction_index: usize
    ) {
        let this = &tx.instructions[instruction_index].instruction_address;
        for i in instruction_index+1..tx.instructions.len() {
            let other = &tx.instructions[i].instruction_address;
            if this.len() < other.len() && this == &other[..this.len()] {
                sel.instructions[i] = true;
            } else {
                return;
            }
        }
    }

    fn eval_parent_instructions(
        sel: &mut SelectedItems,
        tx: &TransactionData,
        instruction_index: usize
    ) {
        let mut len = tx.instructions[instruction_index].instruction_address.len() - 1;
        if len == 0 {
            return;
        }
        for i in (0..instruction_index).rev() {
            if len == tx.instructions[i].instruction_address.len() {
                sel.instructions[i] = true;
                len -= 1;
                if len == 0 {
                    return;
                }
            }
        }
    }
}


pub fn compile_instruction_request(req: InstructionRequest) -> Option<PreparedInstructionRequest> {
    let mut filter = PreparedInstructionRequest::default();

    if let Some(list) = req.program_id {
        if list.is_empty() {
            return None
        }
        let set: HashSet<_> = list.into_iter().collect();
        filter.add(move |ins| {
            set.contains(&ins.account_list[ins.program_id as usize])
        });
    }

    if let Some(list) = req.discriminator {
        let list: Vec<_> = list.into_iter().filter_map(|s| parse_hex(&s)).collect();

        if list.is_empty() {
            return None
        }

        if !list.iter().any(|d| d.is_empty()) {
            filter.add(move |ins| {
                list.iter().any(|d| {
                    ins.binary_data.get(..d.len()) == Some(d)
                })
            });
        }
    }

    if let Some(list) = req.mentions_account {
        if list.is_empty() {
            return None
        }

        let set: HashSet<_> = list.into_iter().collect();

        filter.add(move |ins| {
            ins.accounts.iter().any(|i| {
                set.contains(&ins.account_list[*i as usize])
            })
        })
    }

    macro_rules! acc {
        ($i:literal, $name:ident) => {
            if let Some(list) = req.$name {
                if list.is_empty() {
                    return None
                }
                let set: HashSet<_> = list.into_iter().collect();
                filter.add(move |ins| {
                    ins.accounts.get($i).map_or(false, |i| {
                        set.contains(&ins.account_list[*i as usize])
                    })
                })
            }
        };
    }
    acc!(0, a0);
    acc!(1, a1);
    acc!(2, a2);
    acc!(3, a3);
    acc!(4, a4);
    acc!(5, a5);
    acc!(6, a6);
    acc!(7, a7);
    acc!(8, a8);
    acc!(9, a9);
    acc!(10, a10);
    acc!(11, a11);
    acc!(12, a12);
    acc!(13, a13);
    acc!(14, a14);
    acc!(15, a15);

    if let Some(is_committed) = req.is_committed {
        filter.add(move |ins| ins.is_committed == is_committed)
    }

    filter.relations_mut().set_transaction_balances(req.transaction_balances);
    filter.relations_mut().set_transaction_token_balances(req.transaction_token_balances);
    filter.relations_mut().set_transaction_instructions(req.transaction_instructions);
    filter.relations_mut().set_inner_instructions(req.inner_instructions);
    filter.relations_mut().set_parent_instructions(req.parent_instructions);
    filter.relations_mut().set_logs(req.logs);

    Some(filter)
}