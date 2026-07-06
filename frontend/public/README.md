# Public

Files here are copied verbatim into the build output and served by URL from
the site root. Assets belong here because:

- They are referenced by URL: the favicon link in `index.html`, `<img src>`
  in the sidebar and loading spinner, a future `robots.txt`.
- They carry their own colors and do not need to track the theme.

Them-tracking UI glyphs do not belong here. an `<img src>` SVG cannot
inherit `currentColor`, so it would ignore hover states and light/dark. Those
live in `src/components/icons/`.

Everything in this folder ships in the production build and is publicly
served, including this file.

## Rule of thumb

If a URL points at it, it goes here. If CSS needs to reach inside it, it is
a component.

## References

- <https://vite.dev/guide/assets#the-public-directory>
