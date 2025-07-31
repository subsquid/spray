use super::item_filter::{ItemFilter, Relations};
use super::selected_items::SelectedItems;
use crate::data::{Instruction, TransactionData};
use crate::query::util::parse_hex;
use crate::query::InstructionRequest;
use std::collections::HashSet;


#[derive(Debug, Default)]
pub struct InstructionRelations {
    pub transaction_balances: bool,
    pub transaction_token_balances: bool,
    pub transaction_instructions: bool,
    pub inner_instructions: bool,
    pub parent_instructions: bool,
    pub logs: bool,
}


impl Relations for InstructionRelations {
    fn include(&mut self, other: &Self) {
        self.transaction_balances |= other.transaction_balances;
        self.transaction_token_balances |= other.transaction_token_balances;
        self.transaction_instructions |= other.transaction_instructions;
        self.inner_instructions |= other.inner_instructions;
        self.parent_instructions |= other.parent_instructions;
        self.logs |= other.logs;
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
                sel.include_all_instructions |= rel.transaction_instructions;
                
                if rel.inner_instructions && !sel.include_all_instructions {
                    Self::eval_inner_instructions(sel, tx, i);
                }

                if rel.parent_instructions && !sel.include_all_instructions {
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
            set.contains(&ins.account_dict[ins.program_id as usize])
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
                set.contains(&ins.account_dict[*i as usize])
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
                        set.contains(&ins.account_dict[*i as usize])
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

    filter.relations_mut().transaction_balances = req.transaction_balances;
    filter.relations_mut().transaction_token_balances = req.transaction_token_balances;
    filter.relations_mut().transaction_instructions = req.transaction_instructions;
    filter.relations_mut().inner_instructions = req.inner_instructions;
    filter.relations_mut().parent_instructions = req.parent_instructions;
    filter.relations_mut().logs = req.logs;

    Some(filter)
}