use super::source::TransactionUpdate;
use crate::data::{AccountDict, Instruction, JsonString, Transaction, TransactionData, TransactionVersion};
use crate::geyser::solana::storage::confirmed_block::MessageAddressTableLookup;
use crate::json_builder::{safe_prop, JsonBuilder};


pub fn map_transaction(update: TransactionUpdate) -> TransactionData {
    let meta = update.meta;

    let transaction = Transaction {
        version: if update.versioned { TransactionVersion::Legacy } else { TransactionVersion::Other(0) },
        account_keys: update.account_keys.len(),
        address_table_lookups: render_address_table_lookups(&update.address_table_lookups),
        num_readonly_signed_accounts: update.header.num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: update.header.num_readonly_unsigned_accounts as u8,
        num_required_signatures: update.header.num_required_signatures as u8,
        recent_blockhash: bs58::encode(&update.recent_blockhash).into_string(),
        signatures: JsonBuilder::render(|json| json.base58_list(&update.signatures)),
        err: None,
        compute_units_consumed: meta.compute_units_consumed,
        fee: meta.fee,
        loaded_addresses: JsonBuilder::render(|json| {
            json.begin_object();
            safe_prop!(json, "writable", json.base58_list(&meta.loaded_writable_addresses));
            safe_prop!(json, "readonly", json.base58_list(&meta.loaded_readonly_addresses));
            json.end_object();
        }),
    };

    let accounts: AccountDict = {
        let len = update.account_keys.len() + meta.loaded_writable_addresses.len() + meta.loaded_readonly_addresses.len();
        let mut accounts = Vec::with_capacity(len);
        accounts.extend(update.account_keys.iter().map(|a| bs58::encode(a).into_string()));
        accounts.extend(meta.loaded_writable_addresses.iter().map(|a| bs58::encode(a).into_string()));
        accounts.extend(meta.loaded_readonly_addresses.iter().map(|a| bs58::encode(a).into_string()));
        accounts.into()
    };

    let instructions = {
        let len = update.instructions.len() + meta.inner_instructions.iter().map(|l| l.instructions.len()).sum::<usize>();
        let mut instructions: Vec<Instruction> = Vec::with_capacity(len);
        let mut address = Vec::with_capacity(5);

        // re-organize inner instructions
        let mut inner = vec![Vec::new(); update.instructions.len()];
        for item in meta.inner_instructions {
            let i = item.index as usize;
            if inner[i].is_empty() {
                inner[i] = item.instructions;
            } else {
                inner[i].extend(item.instructions);
            }
        }

        for (i, (ins, inner)) in update.instructions.into_iter().zip(inner).enumerate() {
            address.clear();
            address.push(i);

            instructions.push(Instruction {
                instruction_address: address.clone(),
                program_id: ins.program_id_index as u8,
                accounts: ins.accounts,
                data: bs58::encode(&ins.data).into_string(),
                binary_data: ins.data,
                account_dict: accounts.clone(),
                is_committed: transaction.err.is_none(),
            });

            for ins in inner {
                let stack_height = ins.stack_height.unwrap_or(2) as usize;
                assert!(stack_height > 1);

                address.truncate(stack_height);

                if address.len() == stack_height {
                    address[stack_height - 1] += 1;
                } else {
                    assert_eq!(address.len() + 1, stack_height);
                    address.push(0);
                }

                instructions.push(Instruction {
                    instruction_address: address.clone(),
                    program_id: ins.program_id_index as u8,
                    accounts: ins.accounts,
                    data: bs58::encode(&ins.data).into_string(),
                    binary_data: ins.data,
                    account_dict: accounts.clone(),
                    is_committed: transaction.err.is_none(),
                });
            }
        }
        instructions
    };

    TransactionData {
        slot: update.slot,
        transaction_index: update.index,
        transaction,
        instructions,
        accounts
    }
}


fn render_address_table_lookups(lookups: &[MessageAddressTableLookup]) -> JsonString {
    let mut json = JsonBuilder::new();
    json.begin_array();
    for lookup in lookups {
        json.begin_object();
        safe_prop!(json, "accountKey", json.base58(&lookup.account_key));
        safe_prop!(json, "readonlyIndexes", json.number_list(lookup.readonly_indexes.iter().copied()));
        safe_prop!(json, "writableIndexes", json.number_list(lookup.writable_indexes.iter().copied()));
        json.end_object();
        json.comma();
    }
    json.end_array();
    json.into_string()
}