struct CMinHash {
  sigma: Vec<usize>,
  pi: Vec<usize>,
  k: usize,
}

impl CMinHash {
  pub fn new(sigma: Vec<usize>, pi: Vec<usize>, k: usize) -> Self {
    CMinHash { sigma, pi, k }
  }

  pub fn compute(&self, data: &Vec<bool>) -> Vec<usize> {
    let data_permuted = self.apply_permutation(data, &self.sigma);

    let mut hashes = Vec::with_capacity(self.k);
    for k_index in 0..self.k {
      let pi_shifted = self.circulant_shift(k_index);
      let hash_value = data_permuted
        .iter()
        .enumerate()
        .filter_map(
          |(i, &value)| if value { Some(pi_shifted[i]) } else { None },
        )
        .min()
        .unwrap_or(usize::MAX);

      hashes.push(hash_value);
    }

    hashes
  }

  pub fn apply_permutation(
    &self,
    data: &Vec<bool>,
    permutation: &Vec<usize>,
  ) -> Vec<bool> {
    let mut permuted_data = vec![false; data.len()];
    for (i, &position) in permutation.iter().enumerate() {
      if position < data.len() {
        permuted_data[position] = data[i];
      }
    }
    permuted_data
  }

  pub fn circulant_shift(&self, shift: usize) -> Vec<usize> {
    let d = self.pi.len();
    let mut shifted_pi = vec![0; d];

    for (i, &value) in self.pi.iter().enumerate() {
      // Calculate new position with circular wrapping
      let new_position = (i + shift) % d;
      shifted_pi[new_position] = value;
    }

    shifted_pi
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_apply_permutation() {
    let cminhash = CMinHash::new(vec![2, 0, 1], vec![], 0); // pi is irrelevant here
    let data = vec![true, false, true];
    let permuted_data = cminhash.apply_permutation(&data, &cminhash.sigma);
    assert_eq!(permuted_data, vec![false, true, true]);
  }

  #[test]
  fn test_circulant_shift() {
    let cminhash = CMinHash::new(vec![], vec![1, 2, 0], 0); // sigma is irrelevant here
    let shifted_pi = cminhash.circulant_shift(1);
    assert_eq!(shifted_pi, vec![0, 1, 2]);
  }

  #[test]
  fn test_compute_basic() {
    let cminhash = CMinHash::new(vec![0, 2, 1], vec![1, 0, 2], 3);
    let data = vec![true, false, true];
    let hashes = cminhash.compute(&data);
    let expected_hashes = vec![0, 1, 0];
    assert_eq!(hashes, expected_hashes);
  }

  fn jaccard_similarity(v1: &Vec<bool>, v2: &Vec<bool>) -> f64 {
    let intersection = v1.iter().zip(v2).filter(|(&a, &b)| a && b).count();
    let union = v1.iter().zip(v2).filter(|(&a, &b)| a || b).count();
    intersection as f64 / union as f64
  }

  #[test]
  fn test_conformance_with_theoretical_expectations() {
    let cminhash = CMinHash::new(vec![2, 0, 1, 3], vec![1, 3, 0, 2], 4);

    let data1 = vec![true, false, true, false];
    let data2 = vec![true, true, false, false];
    let actual_similarity = jaccard_similarity(&data1, &data2);

    let hashes1 = cminhash.compute(&data1);
    let hashes2 = cminhash.compute(&data2);

    let matching_hashes = hashes1
      .iter()
      .zip(hashes2.iter())
      .filter(|(&a, &b)| a == b)
      .count();
    let estimated_similarity = matching_hashes as f64 / cminhash.k as f64;

    assert!((actual_similarity - estimated_similarity).abs() < 0.2, "The estimated Jaccard similarity does not match the actual similarity closely enough.");
  }

  fn are_similar_hashes(
    hashes1: &Vec<usize>,
    hashes2: &Vec<usize>,
    threshold: f64,
  ) -> bool {
    let matching_hashes = hashes1
      .iter()
      .zip(hashes2)
      .filter(|(&a, &b)| a == b)
      .count();
    let similarity_ratio = matching_hashes as f64 / hashes1.len() as f64;
    similarity_ratio >= threshold
  }

  #[test]
  fn test_deduplication_with_cminhash() {
    let cminhash = CMinHash::new(vec![2, 0, 1, 3, 4], vec![1, 3, 0, 4, 2], 5);

    let entry1 = vec![true, false, true, false, true];
    let entry2 = entry1.clone(); // Duplicate of entry1
    let entry3 = vec![true, true, false, false, true]; // Similar to entry1 and entry2, but not identical
    let entry4 = vec![false, false, false, false, false]; // Significantly different from the others

    let hashes1 = cminhash.compute(&entry1);
    let hashes2 = cminhash.compute(&entry2);
    let hashes3 = cminhash.compute(&entry3);
    let hashes4 = cminhash.compute(&entry4);

    assert!(
      are_similar_hashes(&hashes1, &hashes2, 0.8),
      "Entry1 and Entry2 should be identified as duplicates"
    );
    assert!(
      are_similar_hashes(&hashes1, &hashes3, 0.4),
      "Entry1 and Entry3 should be identified as near-duplicates"
    );
    assert!(
      !are_similar_hashes(&hashes1, &hashes4, 0.8),
      "Entry1 and Entry4 should not be identified as similar"
    );
  }
}
