# Icons

We keep UI glyphs as inline SVG components because:

- They inherit text color. `stroke="currentColor"` picks up hover
  transitions and light/dark theming with zero extra assets.
- State swaps are JSX, not fetches e.g. the eye/eye-off toggle.
- No icon library dependency. Each icon is a couple of paths.

Conventions:

- One file per icon, built on `IconBase.tsx` (24-unit viewBox, 2px stroke,
  round caps, 17px default), exported from `index.tsx`.
- Icons stay `aria-hidden`, the control using them carries the accessible
  name.
- Import from the barrel: `import { EyeIcon } from "../icons"`.

URL-addressed standalone assets (logo, favicon, social images) live in
`public/` instead - an `<img src>` SVG cannot inherit `currentColor`.

## References

- <https://feathericons.com/>
- <https://developer.mozilla.org/en-US/docs/Web/CSS/color_value#currentcolor_keyword>
