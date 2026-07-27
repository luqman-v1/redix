import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { toasts } from '$lib/stores/toasts';

export async function exportCsv(filename: string, headers: string[], rows: string[][]) {
  try {
    const defaultPath = `${filename}.csv`;
    const filePath = await save({
      defaultPath,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    });

    if (!filePath) return;

    // Build CSV string
    const escapeCsv = (str: string) => {
      const val = str === null || str === undefined ? '' : String(str);
      if (val.includes(',') || val.includes('\n') || val.includes('"')) {
        return `"${val.replace(/"/g, '""')}"`;
      }
      return val;
    };

    const lines = [
      headers.map(escapeCsv).join(','),
      ...rows.map(row => row.map(escapeCsv).join(','))
    ];

    const csvContent = lines.join('\n');
    await writeTextFile(filePath, csvContent);
    toasts.add(`Exported successfully to ${filePath}`, 'success');
  } catch (err) {
    console.error("Export failed", err);
    toasts.add("Failed to export: " + String(err), 'error');
  }
}

export async function importCsv(): Promise<string[][] | null> {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    
    const filePath = await open({
      filters: [{ name: 'CSV', extensions: ['csv'] }],
      multiple: false
    });
    
    if (!filePath || typeof filePath !== 'string') return null;

    const content = await readTextFile(filePath);
    return parseCsv(content);
  } catch (err) {
    console.error("Import failed", err);
    toasts.add("Failed to import: " + String(err), 'error');
    return null;
  }
}

export function parseCsv(text: string): string[][] {
  const result: string[][] = [];
  let currentLine: string[] = [];
  let currentVal = '';
  let inQuotes = false;

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    
    if (inQuotes) {
      if (char === '"') {
        if (i + 1 < text.length && text[i + 1] === '"') {
          currentVal += '"';
          i++; // skip escaped quote
        } else {
          inQuotes = false;
        }
      } else {
        currentVal += char;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
      } else if (char === ',') {
        currentLine.push(currentVal);
        currentVal = '';
      } else if (char === '\n' || char === '\r') {
        if (char === '\r' && i + 1 < text.length && text[i + 1] === '\n') {
          i++;
        }
        currentLine.push(currentVal);
        result.push(currentLine);
        currentLine = [];
        currentVal = '';
      } else {
        currentVal += char;
      }
    }
  }
  
  if (currentLine.length > 0 || currentVal !== '') {
    currentLine.push(currentVal);
    result.push(currentLine);
  }

  // Remove empty trailing lines
  while (result.length > 0 && result[result.length - 1].length === 1 && result[result.length - 1][0] === '') {
    result.pop();
  }

  return result;
}
