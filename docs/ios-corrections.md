# iOS Corrections

Localization fixes needed in `Peach/Resources/Localizable.xcstrings`.

## 1. Comparison Controls help text — incorrect timing claim

**Key:** `"After both notes have played, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard. You can't answer while the notes are still playing."`

**Problem:** Says buttons activate "after both notes have played" and "you can't answer while the notes are still playing." In reality, the buttons become active as soon as the second (target) note starts playing — the user can answer while it's still sounding.

**Fix (EN):** "Once the second note starts playing, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard."

**Fix (DE):** "Sobald der zweite Ton zu spielen beginnt, werden die Tasten **Höher** und **Tiefer** aktiv. Tippe auf die Taste, die zu dem passt, was du gehört hast."

## 2. Chart help body texts — missing trailing periods

**Problem:** Several chart help body strings are complete sentences but omit the trailing period. This may be intentional for SwiftUI tooltip styling, but for consistency with proper punctuation the help modal versions should have periods.

**Affected keys:**
- `"This chart shows how your pitch perception is developing over time"`
- `"The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress"`
- `"The shaded area around the line shows how consistent you are — a narrower band means more reliable results"`
- `"The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper"`
- `"The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right"`

**Fix:** Add trailing period to each.
