# Visual Guide - Text Display Change

## Before (Old Behavior)

```
┌─────────────────────────────────────────┐
│ 1. アウトライン  [rendered text]        │ ← Line 0
│    アウトライン  [input field]          │
├─────────────────────────────────────────┤
│ 2.  子トピック  [rendered text]         │ ← Line 1  
│     子トピック  [input field]           │
├─────────────────────────────────────────┤
│ 3.   更に深いトピック  [rendered text]  │ ← Line 2
│      更に深いトピック  [input field]    │
├─────────────────────────────────────────┤
│ 4. 　全角スペース始まり [rendered text] │ ← Line 3
│    　全角スペース始まり [input field]   │
└─────────────────────────────────────────┘
```
**Problem**: Each line takes up 2 rows (微妙 - "not ideal")

## After (New Behavior)

### When Line 0 is Focused
```
┌─────────────────────────────────────────┐
│ 1. アウトライン  [input field █]        │ ← Line 0 (FOCUSED - shows input)
├─────────────────────────────────────────┤
│ 2.  子トピック  [rendered text only]    │ ← Line 1 (not focused)
├─────────────────────────────────────────┤
│ 3.   更に深いトピック  [rendered text]  │ ← Line 2 (not focused)
├─────────────────────────────────────────┤
│ 4. 　全角スペース始まり [rendered text] │ ← Line 3 (not focused)
└─────────────────────────────────────────┘
```

### After Pressing ArrowDown (Line 1 is Focused)
```
┌─────────────────────────────────────────┐
│ 1. アウトライン  [rendered text only]   │ ← Line 0 (not focused anymore)
├─────────────────────────────────────────┤
│ 2.  子トピック  [input field █]         │ ← Line 1 (FOCUSED - shows input)
├─────────────────────────────────────────┤
│ 3.   更に深いトピック  [rendered text]  │ ← Line 2 (not focused)
├─────────────────────────────────────────┤
│ 4. 　全角スペース始まり [rendered text] │ ← Line 3 (not focused)
└─────────────────────────────────────────┘
```

### After Clicking on Line 3
```
┌─────────────────────────────────────────┐
│ 1. アウトライン  [rendered text only]   │ ← Line 0 (not focused)
├─────────────────────────────────────────┤
│ 2.  子トピック  [rendered text only]    │ ← Line 1 (not focused anymore)
├─────────────────────────────────────────┤
│ 3.   更に深いトピック  [rendered text]  │ ← Line 2 (not focused)
├─────────────────────────────────────────┤
│ 4. 　全角スペース始まり [input field █] │ ← Line 3 (FOCUSED - shows input)
└─────────────────────────────────────────┘
```

## Key Improvements ✨

1. **Space Efficient**: Only 1 row per line (not 2)
2. **Clear Focus**: Immediately see which line is being edited
3. **Arrow Navigation**: Easy to move between lines with ↑↓
4. **Clean View**: Non-focused lines show only the formatted result

## Arrow Key Behavior

```
Press ↑ (ArrowUp)
─────────────────────
Line 3 focused → Line 2 focused → Line 1 focused → Line 0 focused → (stays at 0)

Press ↓ (ArrowDown)
─────────────────────
Line 0 focused → Line 1 focused → Line 2 focused → Line 3 focused → (stays at 3)
```

## Column Position Preservation

```
Line 1: "This is a long line of text"
        Cursor at position 10 ────────┘

Press ↓

Line 2: "Short"
        Cursor at position 5 (end) ───┘
        (clamped because line 2 is shorter)

Press ↑

Line 1: "This is a long line of text"
        Cursor at position 5 ──────────┘
        (attempted to preserve position 5, not original 10)
```

## Implementation Overview

```rust
// Check which line has focus
let is_focused = selection.read().focus.line == line_index;

// Render conditionally
if is_focused {
    // Show input for editing
    input { value: line.text, autofocus: true, ... }
} else {
    // Show rendered result (clickable to focus)
    div { 
        onclick: move |_| { set_focus(line_index) },
        {render_line(...)}
    }
}
```

## CSS Enhancement

```css
.line-render-container {
    cursor: text;          /* Shows text cursor on hover */
    transition: ...        /* Smooth hover effect */
}

.line-render-container:hover {
    background-color: ...  /* Visual feedback */
}
```

## Legend

- `[input field █]` - Input element with cursor
- `[rendered text only]` - Formatted/rendered display
- `(FOCUSED)` - Line that currently has focus
- `↑↓` - Arrow up/down keys
- `█` - Cursor position
