//! Array built-in methods.

/// Array prototype methods (ES6+).
pub struct ArrayPrototype;

impl ArrayPrototype {
    // ES6 methods

    /// Array.prototype.find - Find the first element matching a predicate.
    pub fn find<T, F>(arr: &[T], predicate: F) -> Option<&T>
    where
        F: Fn(&T, usize, &[T]) -> bool,
    {
        for (i, item) in arr.iter().enumerate() {
            if predicate(item, i, arr) {
                return Some(item);
            }
        }
        None
    }

    /// Array.prototype.findIndex - Find the index of the first matching element.
    pub fn find_index<T, F>(arr: &[T], predicate: F) -> Option<usize>
    where
        F: Fn(&T, usize, &[T]) -> bool,
    {
        for (i, item) in arr.iter().enumerate() {
            if predicate(item, i, arr) {
                return Some(i);
            }
        }
        None
    }

    /// Array.prototype.fill - Fill an array with a value.
    pub fn fill<T: Clone>(arr: &mut [T], value: T, start: Option<isize>, end: Option<isize>) {
        let len = arr.len() as isize;
        let start = Self::normalize_index(start.unwrap_or(0), len);
        let end = Self::normalize_index(end.unwrap_or(len), len);

        for item in arr.iter_mut().take(end).skip(start) {
            *item = value.clone();
        }
    }

    /// Array.prototype.copyWithin - Copy elements within the array.
    pub fn copy_within<T: Clone>(
        arr: &mut [T],
        target: isize,
        start: Option<isize>,
        end: Option<isize>,
    ) {
        let len = arr.len() as isize;
        let target = Self::normalize_index(target, len);
        let start = Self::normalize_index(start.unwrap_or(0), len);
        let end = Self::normalize_index(end.unwrap_or(len), len);

        if start >= end || target >= arr.len() {
            return;
        }

        let count = (end - start).min(arr.len() - target);

        // Copy to temp to handle overlapping regions
        let temp: Vec<T> = arr[start..start + count].to_vec();
        for (i, item) in temp.into_iter().enumerate() {
            arr[target + i] = item;
        }
    }

    // ES2016 methods

    /// Array.prototype.includes - Check if an array includes an element.
    pub fn includes<T: PartialEq>(
        arr: &[T],
        search_element: &T,
        from_index: Option<isize>,
    ) -> bool {
        let len = arr.len() as isize;
        let from = Self::normalize_index(from_index.unwrap_or(0), len);

        for item in arr.iter().skip(from) {
            if item == search_element {
                return true;
            }
        }
        false
    }

    // ES2019 methods

    /// Array.prototype.flat - Flatten a nested array.
    pub fn flat<T: Clone>(arr: &[Vec<T>], depth: usize) -> Vec<T> {
        if depth == 0 {
            return arr.iter().flatten().cloned().collect();
        }
        arr.iter().flatten().cloned().collect()
    }

    // ES2022 methods

    /// Array.prototype.at - Get element at index (supports negative indices).
    pub fn at<T>(arr: &[T], index: isize) -> Option<&T> {
        let len = arr.len() as isize;
        let normalized = if index >= 0 { index } else { len + index };
        if normalized >= 0 && normalized < len {
            Some(&arr[normalized as usize])
        } else {
            None
        }
    }

    /// Array.prototype.findLast - Find the last element matching a predicate.
    pub fn find_last<T, F>(arr: &[T], predicate: F) -> Option<&T>
    where
        F: Fn(&T, usize, &[T]) -> bool,
    {
        for (i, item) in arr.iter().enumerate().rev() {
            if predicate(item, i, arr) {
                return Some(item);
            }
        }
        None
    }

    /// Array.prototype.findLastIndex - Find the index of the last matching element.
    pub fn find_last_index<T, F>(arr: &[T], predicate: F) -> Option<usize>
    where
        F: Fn(&T, usize, &[T]) -> bool,
    {
        for (i, item) in arr.iter().enumerate().rev() {
            if predicate(item, i, arr) {
                return Some(i);
            }
        }
        None
    }

    // Helper methods

    fn normalize_index(index: isize, len: isize) -> usize {
        if index < 0 {
            (len + index).max(0) as usize
        } else {
            (index.min(len)) as usize
        }
    }
}

/// Array static methods (ES6+).
pub struct ArrayStatic;

impl ArrayStatic {
    /// Array.from - Create an array from an iterable or array-like object.
    pub fn from<T: Clone>(iterable: &[T]) -> Vec<T> {
        iterable.to_vec()
    }

    /// Array.of - Create an array from arguments.
    pub fn of<T>(items: Vec<T>) -> Vec<T> {
        items
    }

    /// Array.isArray - Check if a value is an array.
    pub fn is_array<T>(_value: &T) -> bool {
        // In a real implementation, this would check the internal [[Class]]
        true // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = ArrayPrototype::find(&arr, |&x, _, _| x > 3);
        assert_eq!(result, Some(&4));
    }

    #[test]
    fn test_find_index() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = ArrayPrototype::find_index(&arr, |&x, _, _| x > 3);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_includes() {
        let arr = vec![1, 2, 3, 4, 5];
        assert!(ArrayPrototype::includes(&arr, &3, None));
        assert!(!ArrayPrototype::includes(&arr, &6, None));
    }

    #[test]
    fn test_at() {
        let arr = vec![1, 2, 3, 4, 5];
        assert_eq!(ArrayPrototype::at(&arr, 0), Some(&1));
        assert_eq!(ArrayPrototype::at(&arr, -1), Some(&5));
        assert_eq!(ArrayPrototype::at(&arr, 10), None);
    }

    #[test]
    fn test_find_last() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = ArrayPrototype::find_last(&arr, |&x, _, _| x < 4);
        assert_eq!(result, Some(&3));
    }
}
