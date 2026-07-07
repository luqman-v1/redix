# Task 16: Format Detection and Beautifier Utility

## Status: DONE

## Commit
`7b5a0f4` - feat: format detection and beautifier utility

## Files Created

- `src/lib/utils/format-detector.ts` - `detectFormat()` and `isBinary()` functions
- `src/lib/utils/beautifier.ts` - `beautify()`, `formatXml()`, and `toHex()` functions

## Details

- `detectFormat`: checks binary (non-printable chars), JSON (parseable `{`/`[`), XML (`<...>`), else text
- `beautify`: returns `{ formatted, format }` - JSON gets `stringify` with 2-space indent, XML gets simple tag indentation, binary gets hex dump, text passthrough
- `formatXml`: splits on `><` boundaries, indents open tags, dedents close tags, 2-space indent
- `toHex`: each char to 2-digit hex, space-separated

No tests needed per task spec - validated through UI in Task 17.
