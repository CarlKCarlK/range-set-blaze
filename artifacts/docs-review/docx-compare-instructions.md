# RangeSetBlaze documentation comparison

- **Original (OLD):** `rangesetblaze-docs-old-aec75cd6.docx`
  - Commit: `aec75cd6d58ba8898c7393f456027123b85adbf5`
- **Revised (NEW):** `rangesetblaze-docs-new-1623c9cd.docx`
  - Commit: `1623c9cde9f10e32bd7596ae965dbf1048b92eb0`

In Microsoft Word, choose **Review → Compare → Compare...**. Select the OLD file as
**Original document** and the NEW file as **Revised document**, then choose **OK**.

The intended review covers three related documentation changes: the replacement of the
experimental range-or-gap API with the public gap APIs (`range_at`, `range_or_gap_at`, and
`fill_gaps` across sets, maps, and sorted-disjoint streams); the transition of `f32`/`f64`
support from experimental feature-gated documentation to built-in support; and the new
nightly experimental cursor-based insertion feature. The NEW build enables
`cursor_nightly_experimental`, but the feature currently selects only private implementation
code and has no corresponding rendered public Rustdoc prose or public item. Private-item
documentation was intentionally excluded.

The documents use the same curated section order. They omit Rustdoc navigation, source
controls, generated page furniture, auto-trait and blanket implementations, dependency
documentation, timestamps, and local build paths.
