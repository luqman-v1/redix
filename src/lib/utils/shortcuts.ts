export interface Shortcut {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  action: () => void;
  description: string;
}

export function registerShortcuts(shortcuts: Shortcut[]) {
  function handler(e: KeyboardEvent) {
    for (const s of shortcuts) {
      const ctrlMatch = s.ctrl ? (e.ctrlKey || e.metaKey) : true;
      const shiftMatch = s.shift ? e.shiftKey : true;
      if (e.key === s.key && ctrlMatch && shiftMatch) {
        e.preventDefault();
        s.action();
        return;
      }
    }
  }
  window.addEventListener("keydown", handler);
  return () => window.removeEventListener("keydown", handler);
}
