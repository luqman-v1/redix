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
