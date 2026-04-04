## 1. Form Query Model

- [x] 1.1 Extend the form query layer to return local and inherited invocation-relevant fields with explicit owner metadata.
- [x] 1.2 Define ordering/grouping rules so local fields remain primary and inherited fields can be grouped by owning ancestor.

## 2. Form Rendering

- [x] 2.1 Render inherited owner sections or equivalent provenance cues in the active form panel.
- [x] 2.2 Update inherited field badges and selected-field helper copy to identify owner path and truthful edit scope.

## 3. Verification

- [x] 3.1 Add tests proving descendant forms expose inherited invocation-relevant options that appear in the preview.
- [x] 3.2 Add tests covering multi-owner grouping and inherited helper copy semantics.
