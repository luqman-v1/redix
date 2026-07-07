import { detectFormat, type DataFormat } from "./format-detector";

export function beautify(value: string): {
  formatted: string;
  format: DataFormat;
} {
  const format = detectFormat(value);

  switch (format) {
    case "json":
      return { formatted: JSON.stringify(JSON.parse(value), null, 2), format };
    case "xml":
      return { formatted: formatXml(value), format };
    case "binary":
      return { formatted: toHex(value), format };
    default:
      return { formatted: value, format };
  }
}

function formatXml(xml: string): string {
  let indent = 0;
  const lines: string[] = [];

  // split on >< boundaries, preserving empty tags and closing tags
  const tokens = xml.replace(/>\s*</g, ">\n<").split("\n");

  for (const token of tokens) {
    const trimmed = token.trim();
    if (!trimmed) continue;

    // closing tag: dedent before printing
    if (trimmed.startsWith("</")) {
      indent = Math.max(0, indent - 1);
    }

    lines.push("  ".repeat(indent) + trimmed);

    // opening tag (not self-closing, not closing, not declaration): indent after
    if (
      trimmed.startsWith("<") &&
      !trimmed.startsWith("</") &&
      !trimmed.startsWith("<?") &&
      !trimmed.endsWith("/>")
    ) {
      indent++;
    }
  }

  return lines.join("\n");
}

export function toHex(value: string): string {
  return Array.from(value)
    .map((ch) => ch.charCodeAt(0).toString(16).padStart(2, "0"))
    .join(" ");
}
