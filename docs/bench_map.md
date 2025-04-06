
Possible new candidates:

Crate Ranges Element Type Set Operations? Map Support? Notes
intervaltree Possibly overlapping Ord ❌ ✅ Maps values to overlapping intervals. Query for intervals that overlap a point or range.
intervaltree-rs Possibly overlapping Ord ❌ ✅ Faster, immutable implementation.
range_map Disjoint Ord ❌ ✅ Duplicate of rangemap; older/inactive.
time_range_map Disjoint DateTime ❌ ✅ Focused on time intervals, similar to rangemap.
range_dict Disjoint Ord ❌ ✅ Thin wrapper over BTreeMap<RangeInclusive<T>, V>. No set ops.
interval_map Disjoint or overlapping Ord ❌ ✅ Claims to support both with linear search; experimental.
interval-collections Disjoint Ord ✅ ✅ Aims for high performance and correctness.
crunchy-interval-map Overlapp
