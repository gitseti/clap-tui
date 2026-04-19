## ADDED Requirements

### Requirement: Section hierarchy remains stable while content is clipped
The TUI SHALL preserve section hierarchy cues when long forms are scrolled or clipped without resurrecting offscreen section headings above still-visible rows from the same section.

#### Scenario: Section heading scrolls fully offscreen while section rows remain visible
- **WHEN** the heading row for a section has scrolled above the visible form viewport but one or more rows from that section are still visible
- **THEN** the heading remains offscreen
- **AND** the viewport does not redraw that heading at the top edge until the next section boundary actually becomes visible

#### Scenario: Later section boundary enters the viewport
- **WHEN** scrolling advances far enough that a later section boundary becomes visible in the viewport
- **THEN** the later section heading appears once at that boundary
- **AND** clipped rows from an earlier section do not cause the earlier heading to reappear
