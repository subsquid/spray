use super::filter::SelectedItems;
use super::{BlockFieldSelection, FieldSelection};
use crate::data::{BlockData, TransactionData};
use crate::json_builder::{safe_prop, JsonBuilder};


pub fn render_transaction_message(fields: &FieldSelection, tx: &TransactionData, sel: &SelectedItems) -> String {
    let mut json = JsonBuilder::new();
    json.begin_object();

    safe_prop!(json, "type", json.safe_str("transaction"));
    safe_prop!(json, "slot", json.number(tx.slot));
    safe_prop!(json, "transactionIndex", json.number(tx.transaction_index));

    if sel.transaction {
        safe_prop!(json, "transaction", {
            let fields = &fields.transaction;
            let transaction_index = tx.transaction_index;
            let tx = &tx.transaction;
            json.begin_object();
            if fields.transaction_index {
                safe_prop!(json, "transactionIndex", json.number(transaction_index));
            }
            if fields.version {
                safe_prop!(json, "version", json.value(&tx.version));
            }
            if fields.err {
                safe_prop!(json, "err", {
                    if let Some(err) = tx.err.as_ref() {
                        json.raw(err)
                    } else {
                        json.null()
                    }
                });
            }
            if fields.signatures {
                safe_prop!(json, "signatures", json.raw(&tx.signatures));
            }
            json.end_object();
        });
    }

    if !sel.instructions.is_empty() {
        safe_prop!(json, "instructions", {
            json.begin_array();
            sel.instructions.for_each_selected(|i| {
                let ins = &tx.instructions[i];
                let fields = &fields.instruction;
                json.begin_object();
                if fields.transaction_index {
                     safe_prop!(json, "transactionIndex", json.number(tx.transaction_index));
                }
                if fields.instruction_address {
                    safe_prop!(json, "instructionAddress", json.value(&ins.instruction_address));
                }
                if fields.program_id {
                    safe_prop!(json, "programId", json.safe_str(&tx.accounts[ins.program_id as usize]));
                }
                if fields.accounts {
                    safe_prop!(json, "accounts", {
                        json.begin_array();
                        for i in ins.accounts.iter().copied() {
                            json.safe_str(&tx.accounts[i as usize]);
                            json.comma();
                        }
                        json.end_array();
                    });
                }
                if fields.data {
                    safe_prop!(json, "data", json.safe_str(&ins.data));
                }
                if fields.is_committed {
                    safe_prop!(json, "isCommitted", json.boolean(ins.is_committed));
                }
                json.end_object();
                json.comma();
            });
            json.end_array();
        });
    }

    json.end_object();
    json.into_string()
}


pub fn render_block_message(fields: &BlockFieldSelection, block: &BlockData) -> String {
    let mut json = JsonBuilder::new();
    json.begin_object();
    
    safe_prop!(json, "type", json.safe_str("block"));
    safe_prop!(json, "slot", json.number(block.slot));
    
    if fields.number 
        || fields.hash 
        || fields.parent_number 
        || fields.parent_hash 
        || fields.height 
        || fields.timestamp 
    {
        safe_prop!(json, "header", {
            json.begin_object();
            if fields.number {
                safe_prop!(json, "number", json.number(block.slot));
            }
            if fields.hash {
                safe_prop!(json, "hash", json.safe_str(&block.hash));
            }
            if fields.parent_number {
                safe_prop!(json, "parentNumber", json.number(block.parent_slot));
            }
            if fields.parent_hash {
                safe_prop!(json, "parentHash", json.safe_str(&block.parent_hash));
            }
            if fields.height {
                safe_prop!(json, "height", {
                    if let Some(height) = block.height {
                        json.number(height)
                    } else {
                        json.null()
                    }
                });
            }
            if fields.timestamp {
                safe_prop!(json, "timestamp", json.number(block.timestamp));
            }
            json.end_object();
        });
    }

    json.end_object();
    json.into_string()
}