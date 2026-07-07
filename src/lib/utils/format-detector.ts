export type DataFormat = "json" | "xml" | "binary" | "text";

const BINARY_PATTERN = /[\x00-\x08\x0E-\x1F]/;

export function detectFormat(value: string): DataFormat {
  if (isBinary(value)) return "binary";

  const trimmed = value.trim();
  if (
    (trimmed.startsWith("{") || trimmed.startsWith("[")) &&
    isValidJson(trimmed)
  ) {
    return "json";
  }

  if (trimmed.startsWith("<") && trimmed.endsWith(">")) return "xml";

  return "text";
}

export function isBinary(value: string): boolean {
  return BINARY_PATTERN.test(value);
}

function isValidJson(value: string): boolean {
  try {
    JSON.parse(value);
    return true;
  } catch {
    return false;
  }
}
