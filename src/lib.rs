mod pb;

use pb::sf::solana::transfers::v1::{Transfer, Transfers};
use substreams_solana::pb::sol;

fn extract_transfer(
    instruction: &sol::v1::CompiledInstruction,
    account_keys: &[Vec<u8>],
    transfers: &mut Vec<Transfer>,
) {
    let program_id = match account_keys.get(instruction.program_id_index as usize) {
        Some(key) => key,
        None => return,
    };

    // The System Program public key is 32 zero bytes.
    if program_id.as_slice() != &[0u8; 32] {
        return;
    }

    // Native SOL Transfer instruction: 4-byte discriminator (2)
    // followed by an 8-byte little-endian lamport amount.
    if instruction.data.len() < 12
        || instruction.data[0..4] != [2, 0, 0, 0]
    {
        return;
    }

    let from_index = match instruction.accounts.get(0) {
        Some(index) => *index as usize,
        None => return,
    };

    let to_index = match instruction.accounts.get(1) {
        Some(index) => *index as usize,
        None => return,
    };

    let from_key = match account_keys.get(from_index) {
        Some(key) => key,
        None => return,
    };

    let to_key = match account_keys.get(to_index) {
        Some(key) => key,
        None => return,
    };

    let amount = u64::from_le_bytes([
        instruction.data[4],
        instruction.data[5],
        instruction.data[6],
        instruction.data[7],
        instruction.data[8],
        instruction.data[9],
        instruction.data[10],
        instruction.data[11],
    ]);

    transfers.push(Transfer {
        from: bs58::encode(from_key).into_string(),
        to: bs58::encode(to_key).into_string(),
        amount,
    });
}

#[substreams::handlers::map]
fn map_transfers(block: sol::v1::Block) -> Result<Transfers, substreams::errors::Error> {
    let mut transfers = Vec::new();

    for confirmed in &block.transactions {
        // Ignore failed transactions.
        if confirmed.meta.as_ref().map_or(false, |meta| meta.err.is_some()) {
            continue;
        }

        let transaction = match confirmed.transaction.as_ref() {
            Some(transaction) => transaction,
            None => continue,
        };

        let message = match transaction.message.as_ref() {
            Some(message) => message,
            None => continue,
        };

        // Top-level instructions.
        for instruction in &message.instructions {
            extract_transfer(instruction, &message.account_keys, &mut transfers);
        }

        // Inner instructions (CPI).
        if let Some(meta) = confirmed.meta.as_ref() {
            for inner_group in &meta.inner_instructions {
                for instruction in &inner_group.instructions {
                    extract_transfer(instruction, &message.account_keys, &mut transfers);
                }
            }
        }
    }

    Ok(Transfers { transfers })
}
