use std::collections::HashMap;

pub fn merge_hashmaps<K, V, A>(hashmaps: &mut [HashMap<K, V>]) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone + Extend<A> + IntoIterator<Item = A>,
{
    if hashmaps.is_empty() {
        return HashMap::new();
    }
    hashmaps
        .iter_mut()
        .reduce(|acc, hashmap| {
            for (k, v) in hashmap.iter() {
                acc.entry(k.clone())
                    .and_modify(|acc_v| {
                        acc_v.extend(v.clone());
                    })
                    .or_insert_with(|| v.clone());
            }
            acc
        })
        .unwrap()
        .clone()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_merge_hashmaps() {
        let mut hm1 = HashMap::new();
        hm1.insert("a", vec![1, 2]);
        hm1.insert("b", vec![2, 3]);

        let mut hm2 = HashMap::new();
        hm2.insert("a", vec![1, 2, 4]);
        hm2.insert("b", vec![]);

        let mut hm3 = HashMap::new();
        hm3.insert("a", vec![1, 2]);
        hm3.insert("c", vec![7, 8, 9]);

        let mut hashmaps = vec![hm1, hm2, hm3];
        let merged = merge_hashmaps(&mut hashmaps);

        let mut expected = HashMap::new();
        expected.insert("a", vec![1, 2, 1, 2, 4, 1, 2]);
        expected.insert("b", vec![2, 3]);
        expected.insert("c", vec![7, 8, 9]);

        assert_eq!(merged, expected);
    }
}
