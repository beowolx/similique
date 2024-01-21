use std::hash::{Hash, Hasher};
use wyhash::WyHash;

#[derive(Debug)]
pub struct MinHash {
  num_perm: usize,
  signature: Vec<u64>,
}

impl MinHash {
  pub fn new(num_perm: usize) -> Self {
    MinHash {
      num_perm,
      signature: vec![u64::MAX; num_perm],
    }
  }

  pub fn update<T: Hash>(&mut self, item: &T) {
    let mut hasher = WyHash::with_seed(42);
    item.hash(&mut hasher);
    let hash = hasher.finish();

    // TODO: improve permutation function
    // check this paper: https://arxiv.org/pdf/2109.04595.pdf
    for i in 0..self.num_perm {
      let permuted_hash = (hash.wrapping_add(i as u64)) & u64::MAX;
      self.signature[i] = std::cmp::min(self.signature[i], permuted_hash);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_minhash() {
    let num_perm = 128;
    let minhash = MinHash::new(num_perm);

    assert_eq!(minhash.signature.len(), num_perm);
    assert!(minhash.signature.iter().all(|&x| x == u64::MAX));
  }

  #[test]
  fn test_update_minhash() {
    let mut minhash = MinHash::new(128);

    // Initial state: all values should be u64::MAX
    assert!(minhash.signature.iter().all(|&x| x == u64::MAX));

    // Update with an item
    minhash.update(&"test item");

    // After update: signature should be different
    assert!(minhash.signature.iter().any(|&x| x != u64::MAX));

    // Ensuring the signature length remains constant
    assert_eq!(minhash.signature.len(), 128);
  }

  #[test]
  fn test_update_minhash_multiple() {
    let mut minhash = MinHash::new(128);

    minhash.update(&"test item 1");
    let first_update_signature = minhash.signature.clone();

    minhash.update(&"test item 2");
    let second_update_signature = minhash.signature.clone();

    // Signatures should evolve with each update
    assert_ne!(first_update_signature, second_update_signature);
  }
}
