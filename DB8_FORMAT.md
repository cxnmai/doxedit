# DB8 File Format

`.db8` files are plain UTF-8 text files. The raw file is the source of truth.

Each raw line maps to one rendered paragraph or line in the editor. Blank lines are preserved.

## Inline Styles

Styled text uses bracketed spans:

```text
[style style: text]
```

Styles are composable. A span can use one or many styles:

```text
Normal text [bold underline: important phrase] continues.
[pocket bold: Spending DA] [hat: Uniqueness] [cite small: Smith 24]
```

Supported style tokens:

- `pocket`
- `hat`
- `block`
- `tag`
- `cite`
- `bold`
- `underline`
- `small` or `shrunk`
- `highlight`

Unknown style tokens are ignored so older renderers can still display the text.

## Examples

```text
[pocket bold: Spending DA]
[hat: Uniqueness] The economy is resilient now.
[cite small underline: Smith 24] Growth is stable and inflation is cooling.
This is normal text with [highlight bold: a key phrase].
```

## Design Rules

- Keep documents line-oriented.
- Prefer short, explicit style names over punctuation-heavy syntax.
- Use raw mode when editing source syntax directly.
- Use render mode to preview the styled document.
