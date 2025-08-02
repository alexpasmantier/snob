use rustc_hash::FxHashMap;
use snob_lib::utils::{is_test_file, merge_hashmaps};

#[test]
fn test_is_test_file_basic() {
    // Test positive cases
    assert!(is_test_file("test_module.py"));
    assert!(is_test_file("module_test.py"));
    assert!(is_test_file("/path/to/test_something.py"));
    assert!(is_test_file("/path/to/something_test.py"));

    // Test negative cases
    assert!(!is_test_file("module.py"));
    assert!(!is_test_file("main.py"));
    assert!(!is_test_file("testmodule.py"));
    assert!(!is_test_file("moduletest.py"));
    assert!(!is_test_file("test.py"));
    assert!(!is_test_file(""));
}

#[test]
fn test_merge_hashmaps_basic() {
    let mut map1 = FxHashMap::default();
    map1.insert("a".to_string(), vec![1, 2]);

    let mut map2 = FxHashMap::default();
    map2.insert("a".to_string(), vec![3, 4]);
    map2.insert("b".to_string(), vec![5]);

    let mut maps = vec![map1, map2];
    let result = merge_hashmaps(&mut maps);

    assert!(result.contains_key("a"));
    assert!(result.contains_key("b"));

    let a_values = result.get("a").unwrap();
    assert!(a_values.contains(&1));
    assert!(a_values.contains(&2));
    assert!(a_values.contains(&3));
    assert!(a_values.contains(&4));

    let b_values = result.get("b").unwrap();
    assert_eq!(b_values, &vec![5]);
}

#[test]
fn test_merge_hashmaps_empty() {
    let mut empty_maps: Vec<FxHashMap<String, Vec<i32>>> = vec![];
    let result = merge_hashmaps(&mut empty_maps);
    assert!(result.is_empty());
}
