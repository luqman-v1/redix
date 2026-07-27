<script lang="ts">
  import { beautify } from "$lib/utils/beautifier";
  import { highlightSyntax } from "$lib/utils/highlighter";
  import { detectFormat } from "$lib/utils/format-detector";

  interface Props {
    value: string;
    compact?: boolean;
  }

  let { value, compact = true }: Props = $props();

  let formatted = $derived.by(() => {
    try {
      if (compact) {
        const format = detectFormat(value);
        let out = value;
        if (format === "json") {
          out = JSON.stringify(JSON.parse(value));
        } else if (format === "xml") {
          out = value.replace(/>\s+</g, '><').trim();
        }
        return { formatted: out, format };
      } else {
        return beautify(value);
      }
    } catch {
      return { formatted: value, format: "text" };
    }
  });

  let highlighted = $derived(highlightSyntax(formatted.formatted, formatted.format));
</script>

<div class="syntax-val" class:compact><code>{@html highlighted}</code></div>

<style>
  .syntax-val {
    font-family: "JetBrains Mono", "Fira Code", monospace;
    white-space: pre-wrap;
    word-break: break-all;
    font-size: 0.75rem;
    line-height: 1.5;
  }
  
  /* Syntax Highlighting Themes */
  :global(.hl-key) { color: #9cdcfe; }
  :global(.hl-string) { color: #ce9178; }
  :global(.hl-number) { color: #b5cea8; }
  :global(.hl-boolean) { color: #569cd6; }
  :global(.hl-null) { color: #569cd6; font-style: italic; }
  :global(.hl-tag) { color: #569cd6; }
  :global(.hl-attr) { color: #9cdcfe; }

  :global(.light .hl-key) { color: #1d4ed8; }
  :global(.light .hl-string) { color: #b91c1c; }
  :global(.light .hl-number) { color: #15803d; }
  :global(.light .hl-boolean) { color: #0369a1; }
  :global(.light .hl-null) { color: #0369a1; }
  :global(.light .hl-tag) { color: #991b1b; }
  :global(.light .hl-attr) { color: #e11d48; }
</style>
