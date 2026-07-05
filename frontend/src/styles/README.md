# Styles

We aim to incorporate design decisions in CSS classes because:

- Less JS. Style objects ship in the bundle and are re-created per
  render. Classes are parsed once.
- Avoids inline styles drift.
- Pseudo-states and media queries need CSS. `:hover`, `:focus`,
  `:focus-within`, and responsive rules cannot be expressed inline.
- Theming flows through the stylesheet. Classes reference the custom
  properties in `theme.css`, so light/dark just works without JS.

Inline styles are still fine, and preferred for:

- One-off positional tweaks: `margin`, `maxWidth`, `gap`, flex arrangements
  used once, and residual overrides next to a class e.g.
  `className="table-cell" style={{ fontWeight: 500 }}`.
- Data-driven values e.g. badge colors picked from a status map.
  Only the shape should belong in the class.

## Rule of thumb

The third time you type the same style object, it's a class.

## References

- <https://legacy.reactjs.org/docs/faq-styling.html#are-inline-styles-bad>
- <https://simonadcock.com/are-inline-styles-faster-than-atomic-css/>
