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
            let accounts = &tx.accounts;
            let tx = &tx.transaction;
            json.begin_object();
            if fields.transaction_index {
                safe_prop!(json, "transactionIndex", json.number(transaction_index));
            }
            if fields.version {
                safe_prop!(json, "version", json.value(&tx.version));
            }
            if fields.account_keys {
                safe_prop!(json, "accountKeys", json.array(0..tx.account_keys, |json, i| {
                    json.safe_str(&accounts[i])
                }));
            }
            if fields.address_table_lookups {
                safe_prop!(json, "addressTableLookups", json.raw(&tx.address_table_lookups));
            }
            if fields.num_required_signatures {
                safe_prop!(json, "numRequiredSignatures", json.number(tx.num_required_signatures));
            }
            if fields.num_readonly_signed_accounts {
                safe_prop!(json, "numReadonlySignedAccounts", json.number(tx.num_readonly_signed_accounts));
            }
            if fields.num_readonly_unsigned_accounts {
                safe_prop!(json, "numReadonlyUnsignedAccounts", json.number(tx.num_readonly_unsigned_accounts));
            }
            if fields.recent_blockhash {
                safe_prop!(json, "recentBlockhash", json.safe_str(&tx.recent_blockhash));
            }
            if fields.signatures {
                safe_prop!(json, "signatures", json.raw(&tx.signatures));
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
            if fields.fee {
                safe_prop!(json, "fee", json.number_str(tx.fee));
            }
            if fields.compute_units_consumed {
                safe_prop!(json, "computeUnitsConsumed", {
                    if let Some(val) = tx.compute_units_consumed {
                        json.number_str(val)
                    } else {
                        json.null()
                    }
                });
            }
            if fields.loaded_addresses {
                safe_prop!(json, "loadedAddresses", json.raw(&tx.loaded_addresses));
            }
            if fields.fee_payer {
                safe_prop!(json, "feePayer", {
                    if let Some(acc) = accounts.get(0) {
                        json.safe_str(acc)
                    } else {
                        json.null()
                    }
                });
            }
            if fields.has_dropped_log_messages {
                safe_prop!(json, "hasDroppedLogMessages", json.raw("true"));
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
                if fields.d1 {
                    safe_prop!(json, "d1", {
                        if let Some(bytes) = ins.binary_data.get(..1) {
                            json.binary(bytes)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.d2 {
                    safe_prop!(json, "d2", {
                        if let Some(bytes) = ins.binary_data.get(..2) {
                            json.binary(bytes)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.d4 {
                    safe_prop!(json, "d4", {
                        if let Some(bytes) = ins.binary_data.get(..4) {
                            json.binary(bytes)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.d8 {
                    safe_prop!(json, "d8", {
                        if let Some(bytes) = ins.binary_data.get(..8) {
                            json.binary(bytes)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.error {
                    safe_prop!(json, "error", {
                        if let Some(err) = ins.error.as_ref() {
                            json.str(err)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.compute_units_consumed {
                    safe_prop!(json, "computeUnitsConsumed", json.null());
                }
                if fields.is_committed {
                    safe_prop!(json, "isCommitted", json.boolean(ins.is_committed));
                }
                if fields.has_dropped_log_messages {
                    safe_prop!(json, "hasDroppedLogMessages", json.raw("true"));
                }
                json.end_object();
                json.comma();
            });
            json.end_array();
        });
    }

    if !sel.balances.is_empty() {
        safe_prop!(json, "balances", {
            json.begin_array();
            sel.balances.for_each_selected(|i| {
                let fields = &fields.balance;
                let b = &tx.balances[i];
                json.begin_object();
                if fields.transaction_index {
                     safe_prop!(json, "transactionIndex", json.number(tx.transaction_index));
                }
                if fields.account {
                    safe_prop!(json, "account", json.safe_str(&b.account));
                }
                if fields.pre {
                    safe_prop!(json, "pre", json.number_str(b.pre));
                }
                if fields.post {
                    safe_prop!(json, "post", json.number_str(b.post));
                }
                json.end_object();
                json.comma();
            });
            json.end_array()
        });
    }

    if !sel.token_balances.is_empty() {
        safe_prop!(json, "tokenBalances", {
            json.begin_array();
            sel.token_balances.for_each_selected(|i| {
                let b = &tx.token_balances[i];
                let fields = &fields.token_balance;
                json.begin_object();
                if fields.transaction_index {
                     safe_prop!(json, "transactionIndex", json.number(tx.transaction_index));
                }
                if fields.account {
                    safe_prop!(json, "account", json.safe_str(&b.account));
                }
                macro_rules! account {
                    ($camel:literal, $prop:ident) => {
                        if fields.$prop {
                            safe_prop!(json, $camel, {
                                if let Some(acc) = b.$prop.as_ref() {
                                    json.safe_str(acc)
                                } else {
                                    json.null()
                                }
                            });
                        }
                    };
                }
                account!("preMint", pre_mint);
                account!("postMint", post_mint);
                account!("preProgramId", pre_program_id);
                account!("postProgramId", post_program_id);
                account!("preOwner", pre_owner);
                account!("postOwner", post_owner);

                if fields.pre_decimals {
                    safe_prop!(json, "preDecimals", {
                        if let Some(val) = b.pre_decimals {
                            json.number(val)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.post_decimals {
                    safe_prop!(json, "postDecimals", {
                        if let Some(val) = b.post_decimals {
                            json.number(val)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.pre_amount {
                    safe_prop!(json, "preAmount", {
                        if let Some(val) = b.pre_amount.as_ref() {
                            json.safe_str(val)
                        } else {
                            json.null()
                        }
                    });
                }
                if fields.post_amount {
                    safe_prop!(json, "postAmount", {
                        if let Some(val) = b.post_amount.as_ref() {
                            json.safe_str(val)
                        } else {
                            json.null()
                        }
                    });
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