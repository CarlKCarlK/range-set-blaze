mod tests;

use std::cmp::max;
use std::collections::BTreeMap;

pub fn fmt(items: &BTreeMap<u128, u128>) -> String {
    let mut result = String::new();
    for (start, length) in items {
        if !result.is_empty() {
            result.push(',');
        }
        result.push_str(&format!("{}..{}", start, start + length));
    }
    result
}

pub fn b_d_cmk(items: &mut BTreeMap<u128, u128>, start: u128, end: u128) {
    internal_add(items, start, end - start);
}
pub fn internal_add(items: &mut BTreeMap<u128, u128>, start: u128, length: u128) {
    let end = start + length;
    assert!(start < end); // !!!cmk check that length is not zero
                          // !!! cmk would be nice to have a partition_point function that returns two iterators
    let mut before = items.range_mut(..=start);
    // println!("before {:?}", before.collect::<Vec<_>>());
    if let Some((start_x, length_x)) = before.next() {
        let start_x2 = *start_x;
        println!("start_x {start_x:?}, length_x {length_x:?}");
        let end_x: u128 = start_x + *length_x;
        if end_x < start {
            insert(items, start, end);
        } else if end > end_x {
            *length_x = end - start_x;
            delete_extra(items, start_x2, end);
        } else {
            // do nothing
        }
    } else {
        insert(items, start, end);
    }
}

fn delete_extra(items: &mut BTreeMap<u128, u128>, start: u128, end: u128) {
    let mut after = items.range_mut(start..);
    let (start2, length2) = after.next().unwrap(); // !!! cmk assert that there is a next
    assert!(start == *start2 && start2 + *length2 == end); // !!! cmk real assert
                                                           // !!!cmk would be nice to have a delete_range function
    let mut new_end = end;
    let delete_list = after
        .map_while(|(start3, length3)| {
            if *start3 <= end {
                new_end = max(new_end, start3 + *length3);
                Some(*start3)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    println!("delete_list {delete_list:?}");
    if new_end > end {
        *length2 = new_end - start;
    }
    for start in delete_list {
        items.remove(&start);
    }
}
fn insert(items: &mut BTreeMap<u128, u128>, start: u128, end: u128) {
    let was_there = items.insert(start, end - start);
    assert!(was_there.is_none());
    // !!!cmk real assert
    delete_extra(items, start, end);
}

// !!!cmk can I use a Rust range?
// !!!cmk allow negatives and any size

struct RangeSetInt {
    _items: BTreeMap<u128, u128>, // !!!cmk usize?
                                  // !!!cmk underscore?
}

impl RangeSetInt {
    fn new() -> RangeSetInt {
        RangeSetInt {
            _items: BTreeMap::new(),
        }
    }

    fn clear(&mut self) {
        self._items.clear();
    }

    // !!!cmk keep this in a field
    fn len(&self) -> u128 {
        self._items.values().sum()
    }

    // fn _internal_add(&mut self, start: u128, length: u128) {
    //     // !!!cmk put this shortcut back?
    //     // if self._items.len() == 0 {
    //     //     self._items.insert(start, length);
    //     //     return;
    //     // }

    //     // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    //     // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    //     // !!!cmk rename index to "range"
    //     let range = self._items.range(..start);
    //     let mut peekable_forward = range.clone().peekable();
    //     let peek_forward = peekable_forward.peek();
    //     let mut peekable_backwards = range.rev().peekable();
    //     let peek_backwards = peekable_backwards.peek();
    //     if let Some(peek_forward) = peek_forward {
    //         let mut peek_forward = *peek_forward;
    //         if *peek_forward.0 == start {
    //             if length > *peek_forward.1 {
    //                 peek_forward.1 = &length;
    //                 // previous_range = peek_forward;
    //                 // peek_forward = peekable_forward.next(); // index should point to the following range for the remainder of this method
    //                 todo!()
    //             } else {
    //                 todo!();
    //             }
    //         }
    //     } else {
    //         println!("self._items.insert(start, length);");
    //         if let Some(previous_range) = peek_backwards {
    //             // nothing
    //         } else {
    //             return;
    //         }
    //     }

    //     todo!();
    //     //             return;
    //     //         }
    //     //     } else if index == 0 {
    //     //         self._items.insert(index, RangeX { start, length });
    //     //         previous_index = index;
    //     //         index += 1 // index_of_miss should point to the following range for the remainder of this method
    //     //     } else {
    //     //         previous_index = index - 1;
    //     //         let previous_range: &mut RangeX = &mut self._items[previous_index];

    //     //         if previous_range.end() >= start {
    //     //             let new_length = start + length - previous_range.start;
    //     //             if new_length <= previous_range.length {
    //     //                 return;
    //     //             } else {
    //     //                 previous_range.length = new_length;
    //     //             }
    //     //         } else {
    //     //             // after previous range, not contiguous with previous range
    //     //             self._items.insert(index, RangeX { start, length });
    //     //             previous_index = index;
    //     //             index += 1;
    //     //         }
    //     //     }
    //     // }

    //     // let previous_range: &RangeX = &self._items[previous_index];
    //     // let previous_end = previous_range.end();
    //     // while index < self._items.len() {
    //     //     let range: &RangeX = &self._items[index];
    //     //     if previous_end < range.start {
    //     //         break;
    //     //     }
    //     //     let range_end = range.end();
    //     //     if previous_end < range_end {
    //     //         self._items[previous_index].length = range_end - previous_range.start;
    //     //         index += 1;
    //     //         break;
    //     //     }
    //     //     index += 1;
    //     // }
    //     // self._items.drain(previous_index + 1..index);
    // }
}
