use crate::{CandidateBlock, Hashable, MinerError, Transaction};

/// Return true when a hash starts with the configured difficulty prefix.
pub fn hash_meets_difficulty(hash: &str, difficulty_prefix: &str) -> Result<bool, MinerError> {
    // Steps:
    // 1. Reject a prefix containing non-ASCII-hex characters with `InvalidDifficulty`.
    // 2. Compare using lowercase text so `A` and `a` are treated the same.
    // 3. Return whether `hash` starts with the normalized prefix.
    if !difficulty_prefix
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(MinerError::InvalidDifficulty);
    }

    Ok(hash
        .to_ascii_lowercase()
        .starts_with(&difficulty_prefix.to_ascii_lowercase()))
}

/// Calculate a simple merkle root from transaction hashes.
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<String, MinerError> {
    // Steps:
    // 1. Reject an empty list with `MinerError::EmptyCandidate`.
    // 2. Start with each transaction's `hash_hex()`.
    // 3. Pair hashes left-to-right and hash the concatenated pair.
    // 4. If a level has an odd count, duplicate the final hash.
    // 5. Return the final remaining hash.
    //todo!()
    if transactions.is_empty() {
        return Err(MinerError::EmptyCandidate);
    }
    let mut hashes: Vec<String> = transactions.iter().map(Transaction::hash_hex).collect();
    while hashes.len() > 1 {
        let mut next_hashes = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            let left = &hashes[i];
            let right = if i + 1 < hashes.len() {
                &hashes[i + 1]
            } else {
                left
            };
            next_hashes.push(sha256::digest(format!("{}{}", left, right)));
        }
        hashes = next_hashes;
    }
    Ok(hashes[0].clone())
}

/// Build deterministic candidate hash material for a nonce.
///
/// Use exactly:
/// `candidate:<previous_hash>|height:<height>|merkle:<merkle>|time:<timestamp>|nonce:<nonce>|txs:<txid>;...`
pub fn candidate_hash_material(
    candidate: &CandidateBlock,
    nonce: u64,
) -> Result<String, MinerError> {
    // Steps:
    // 1. Calculate the merkle root for `candidate.transactions`.
    // 2. Start the string with previous hash, height, merkle root, timestamp, and nonce.
    // 3. Append every transaction id followed by `;`.
    // 4. Return the final string.
    let merkle_root = calculate_merkle_root(&candidate.transactions)?;
    let material = format!(
        "candidate:{}|height:{}|merkle:{}|time:{}|nonce:{}|txs:{}",
        candidate.previous_block_hash,
        candidate.height,
        merkle_root,
        candidate.timestamp,
        nonce,
        candidate
            .transactions
            .iter()
                .map(|transaction| format!("{};", transaction.txid))
                .collect::<String>()
    );
    Ok(material)
}

/// Hash a candidate block at one nonce.
pub fn hash_candidate(candidate: &CandidateBlock, nonce: u64) -> Result<String, MinerError> {
    // Steps:
    // 1. Build candidate hash material with `candidate_hash_material`.
    // 2. Return `sha256::digest(material)`.
    let material = candidate_hash_material(candidate, nonce)?;
    Ok(sha256::digest(material))
}
