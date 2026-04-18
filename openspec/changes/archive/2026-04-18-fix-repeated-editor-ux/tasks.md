## 1. Repeated Row Geometry

- [x] 1.1 Update repeated-row layout and hit-test geometry so any row with visible controls reserves an external right-side gutter, with lone remove buttons centered in that gutter.
- [x] 1.2 Keep repeated-value fields on the row-based render path when the input area is height-clipped, so visible rows and controls do not collapse into a merged paragraph block.
- [x] 1.3 Add render and mouse interaction tests covering middle-row remove placement, last-row add/remove placement, and partially clipped repeated editors.

## 2. Repeated Row Navigation

- [x] 2.1 Adjust repeated-editor keyboard handling so `Up` and `Down` move between repeated rows when possible and fall through to previous or next form-field navigation at the first and last rows.
- [x] 2.2 Add reducer or controller tests proving first-row `Up` and last-row `Down` move to adjacent visible fields without regressing existing in-editor row traversal.
