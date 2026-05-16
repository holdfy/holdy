use sha2::{Digest, Sha256};

fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn to_32_bytes(leaf: &[u8]) -> [u8; 32] {
    if leaf.len() == 32 {
        let mut out = [0u8; 32];
        out.copy_from_slice(leaf);
        out
    } else {
        hash_bytes(leaf)
    }
}

/// MerkleTree: build root from leaf hashes. Leaves 32 bytes; internal = SHA256(left || right).
pub fn merkle_root(leaves: &[Vec<u8>]) -> Option<Vec<u8>> {
    if leaves.is_empty() {
        return None;
    }
    let mut cur: Vec<[u8; 32]> = leaves.iter().map(|l| to_32_bytes(l)).collect();
    while cur.len() > 1 {
        let mut next = Vec::with_capacity((cur.len() + 1) / 2);
        let mut i = 0;
        while i < cur.len() {
            let left = &cur[i];
            let right = if i + 1 < cur.len() { &cur[i + 1] } else { &cur[i] };
            let mut combined = left.to_vec();
            combined.extend_from_slice(right);
            next.push(hash_bytes(&combined));
            i += 2;
        }
        cur = next;
    }
    Some(cur[0].to_vec())
}

/// ProofForIndex: root and sibling path for leaf at leaf_index.
pub fn proof_for_index(leaves: &[Vec<u8>], leaf_index: usize) -> (Option<Vec<u8>>, Vec<Vec<u8>>) {
    if leaves.is_empty() || leaf_index >= leaves.len() {
        return (None, vec![]);
    }
    let mut cur: Vec<[u8; 32]> = leaves.iter().map(|l| to_32_bytes(l)).collect();
    let mut proof = Vec::new();
    let mut idx = leaf_index;

    while cur.len() > 1 {
        let mut next = Vec::with_capacity((cur.len() + 1) / 2);
        let mut i = 0;
        while i < cur.len() {
            let left = &cur[i];
            let right = if i + 1 < cur.len() { &cur[i + 1] } else { &cur[i] };
            let mut combined = left.to_vec();
            combined.extend_from_slice(right);
            next.push(hash_bytes(&combined));
            i += 2;
        }
        let sibling = if idx % 2 == 0 {
            if idx + 1 < cur.len() {
                cur[idx + 1].to_vec()
            } else {
                cur[idx].to_vec()
            }
        } else {
            cur[idx - 1].to_vec()
        };
        proof.push(sibling);
        idx /= 2;
        cur = next;
    }

    if cur.len() == 1 {
        (Some(cur[0].to_vec()), proof)
    } else {
        (None, proof)
    }
}
