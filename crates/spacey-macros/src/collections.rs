// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Collection creation macros.
//!
//! Provides ergonomic macros for creating standard library collections.

/// Creates a HashMap from key-value pairs.
///
/// # Example
///
/// ```
/// use spacey_macros::hashmap;
///
/// let map = hashmap! {
///     "key1" => "value1",
///     "key2" => "value2",
/// };
/// assert_eq!(map.get("key1"), Some(&"value1"));
/// ```
#[macro_export]
macro_rules! hashmap {
    () => {
        ::std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key, $value);)+
        map
    }};
}

/// Creates a HashSet from values.
///
/// # Example
///
/// ```
/// use spacey_macros::hashset;
///
/// let set = hashset! { 1, 2, 3 };
/// assert!(set.contains(&2));
/// ```
#[macro_export]
macro_rules! hashset {
    () => {
        ::std::collections::HashSet::new()
    };
    ($($value:expr),+ $(,)?) => {{
        let mut set = ::std::collections::HashSet::new();
        $(set.insert($value);)+
        set
    }};
}

/// Creates a BTreeMap from key-value pairs (sorted by key).
///
/// # Example
///
/// ```
/// use spacey_macros::btreemap;
///
/// let map = btreemap! {
///     "b" => 2,
///     "a" => 1,
///     "c" => 3,
/// };
/// let keys: Vec<_> = map.keys().collect();
/// assert_eq!(keys, vec![&"a", &"b", &"c"]);
/// ```
#[macro_export]
macro_rules! btreemap {
    () => {
        ::std::collections::BTreeMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::BTreeMap::new();
        $(map.insert($key, $value);)+
        map
    }};
}

/// Creates a BTreeSet from values (sorted).
///
/// # Example
///
/// ```
/// use spacey_macros::btreeset;
///
/// let set = btreeset! { 3, 1, 2 };
/// let values: Vec<_> = set.iter().collect();
/// assert_eq!(values, vec![&1, &2, &3]);
/// ```
#[macro_export]
macro_rules! btreeset {
    () => {
        ::std::collections::BTreeSet::new()
    };
    ($($value:expr),+ $(,)?) => {{
        let mut set = ::std::collections::BTreeSet::new();
        $(set.insert($value);)+
        set
    }};
}

/// Create a Vec with pre-allocated capacity.
///
/// # Example
///
/// ```
/// use spacey_macros::vec_with_capacity;
///
/// let items = vec_with_capacity!(100, 1, 2, 3);
/// assert!(items.capacity() >= 100);
/// assert_eq!(items.len(), 3);
/// ```
#[macro_export]
macro_rules! vec_with_capacity {
    ($capacity:expr) => {
        Vec::with_capacity($capacity)
    };
    ($capacity:expr, $($item:expr),+ $(,)?) => {{
        let mut v = Vec::with_capacity($capacity);
        $(v.push($item);)+
        v
    }};
}

/// Creates a VecDeque from values.
///
/// # Example
///
/// ```
/// use spacey_macros::vecdeque;
///
/// let deque = vecdeque![1, 2, 3];
/// assert_eq!(deque.front(), Some(&1));
/// assert_eq!(deque.back(), Some(&3));
/// ```
#[macro_export]
macro_rules! vecdeque {
    () => {
        ::std::collections::VecDeque::new()
    };
    ($($value:expr),+ $(,)?) => {{
        let mut deque = ::std::collections::VecDeque::new();
        $(deque.push_back($value);)+
        deque
    }};
}

/// Creates a LinkedList from values.
///
/// # Example
///
/// ```
/// use spacey_macros::linkedlist;
///
/// let list = linkedlist![1, 2, 3];
/// assert_eq!(list.front(), Some(&1));
/// ```
#[macro_export]
macro_rules! linkedlist {
    () => {
        ::std::collections::LinkedList::new()
    };
    ($($value:expr),+ $(,)?) => {{
        let mut list = ::std::collections::LinkedList::new();
        $(list.push_back($value);)+
        list
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hashmap() {
        let map = hashmap! {
            "a" => 1,
            "b" => 2,
        };
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }

    #[test]
    fn test_hashset() {
        let set = hashset! { 1, 2, 3 };
        assert!(set.contains(&1));
        assert!(!set.contains(&4));
    }

    #[test]
    fn test_btreemap() {
        let map = btreemap! {
            "b" => 2,
            "a" => 1,
        };
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec![&"a", &"b"]);
    }

    #[test]
    fn test_btreeset() {
        let set = btreeset! { 3, 1, 2 };
        let values: Vec<_> = set.iter().collect();
        assert_eq!(values, vec![&1, &2, &3]);
    }

    #[test]
    fn test_vec_with_capacity() {
        let v = vec_with_capacity!(100, 1, 2, 3);
        assert!(v.capacity() >= 100);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_vecdeque() {
        let deque = vecdeque![1, 2, 3];
        assert_eq!(deque.len(), 3);
        assert_eq!(deque.front(), Some(&1));
    }

    #[test]
    fn test_linkedlist() {
        let list = linkedlist![1, 2, 3];
        assert_eq!(list.len(), 3);
    }
}
