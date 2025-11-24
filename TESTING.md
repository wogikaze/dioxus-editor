# Manual Testing Guide

## Purpose
This guide helps verify the text display change implementation.

## What Changed
Previously, each line displayed BOTH:
1. Rendered text view
2. Input field

Now, each line displays ONLY ONE:
- **Focused line**: Input field for editing
- **Non-focused lines**: Rendered text view

## How to Test

### Setup
1. Run `dx serve` or build for web
2. Open the application in a browser

### Test Cases

#### 1. Initial Display
- **Expected**: Only the first line (line 0) should show an input field
- **Expected**: All other lines should show rendered text only
- **Verify**: No double display of text (rendered + input) on any line

#### 2. Clicking on Lines
- **Action**: Click on line 2 (the third line)
- **Expected**: Line 2 now shows an input field
- **Expected**: Previously focused line (line 0) now shows only rendered text
- **Expected**: Cursor appears in the clicked position

#### 3. Arrow Key Navigation - Down
- **Setup**: Focus on line 0 (first line)
- **Action**: Press ArrowDown key
- **Expected**: Focus moves to line 1 (second line)
- **Expected**: Line 1 now shows input field
- **Expected**: Line 0 now shows rendered text only
- **Action**: Press ArrowDown again
- **Expected**: Focus moves to line 2
- **Action**: Continue pressing ArrowDown until last line
- **Expected**: After reaching last line, pressing ArrowDown keeps focus on last line (doesn't crash or scroll)

#### 4. Arrow Key Navigation - Up
- **Setup**: Focus on last line
- **Action**: Press ArrowUp key
- **Expected**: Focus moves to previous line
- **Expected**: Previously focused line shows rendered text only
- **Action**: Continue pressing ArrowUp until first line
- **Expected**: After reaching first line, pressing ArrowUp keeps focus on first line (doesn't crash)

#### 5. Column Position Preservation
- **Setup**: Focus on line 1 (second line: " 子トピック")
- **Action**: Click to position cursor at column 5 (after the 5th character)
- **Action**: Press ArrowDown
- **Expected**: Cursor moves to line 2, column position should be preserved if possible
- **Expected**: If line 2 is shorter, cursor should be at end of line 2

#### 6. Editing Works
- **Setup**: Focus any line
- **Action**: Type some text
- **Expected**: Text appears in input field
- **Expected**: After pressing ArrowDown/ArrowUp, the edited text is preserved
- **Expected**: When returning to the edited line, it shows the edited content

#### 7. Other Keyboard Shortcuts Still Work
- **Action**: Press Tab (should indent)
- **Action**: Press Enter (should create new line)
- **Action**: Press Backspace at line start (should merge with previous line)
- **Expected**: All existing functionality continues to work

### Visual Verification
- Lines should not "jump" or flicker when moving focus
- Transition between rendered and input views should be smooth
- Line numbers and indentation should remain consistent
- No visual artifacts or layout issues

## Known Issues to Watch For
- Focus not moving when arrow keys pressed
- Multiple input fields visible at once
- Rendered text and input both showing on same line
- Cursor position lost when moving between lines
- App crashes when reaching first/last line

## Browser Testing
Test in at least one modern browser:
- Chrome/Edge
- Firefox
- Safari (if available)
