export interface Tab {
  key: string;
  type: string | null;
  ttl: number | null;
}

export const SPECIAL_KEYS = {
  console: "__REDIS_CONSOLE__",
  pubsub: "__PUBSUB__",
  memory: "__MEMORY_ANALYZER__",
  slowlog: "__SLOW_LOG__",
} as const;

function openSpecial(tabs: Tab[], activeIndex: number, key: string): { tabs: Tab[]; activeIndex: number } {
  const idx = tabs.findIndex((t) => t.key === key);
  if (idx === -1) {
    return { tabs: [...tabs, { key, type: null, ttl: null }], activeIndex: tabs.length };
  }
  return { tabs, activeIndex: idx };
}

export function openConsoleTab(tabs: Tab[], activeIndex: number) {
  return openSpecial(tabs, activeIndex, SPECIAL_KEYS.console);
}

export function openPubSubTab(tabs: Tab[], activeIndex: number) {
  return openSpecial(tabs, activeIndex, SPECIAL_KEYS.pubsub);
}

export function openMemoryAnalyzerTab(tabs: Tab[], activeIndex: number) {
  return openSpecial(tabs, activeIndex, SPECIAL_KEYS.memory);
}

export function openSlowLogTab(tabs: Tab[], activeIndex: number) {
  return openSpecial(tabs, activeIndex, SPECIAL_KEYS.slowlog);
}

export function openKeyTab(
  tabs: Tab[],
  activeIndex: number,
  key: string,
  type: string,
  ttl: number,
): { tabs: Tab[]; activeIndex: number } {
  const idx = tabs.findIndex((t) => t.key === key);
  if (idx >= 0) {
    const next = tabs.map((t, i) => (i === idx ? { ...t, type, ttl } : t));
    return { tabs: next, activeIndex: idx };
  }
  return { tabs: [...tabs, { key, type, ttl }], activeIndex: tabs.length };
}

export function closeTab(tabs: Tab[], activeIndex: number, index: number): { tabs: Tab[]; activeIndex: number } {
  const next = tabs.filter((_, i) => i !== index);
  let nextActive = activeIndex;
  if (activeIndex >= next.length) {
    nextActive = Math.max(0, next.length - 1);
  } else if (activeIndex > index) {
    nextActive = activeIndex - 1;
  }
  return { tabs: next, activeIndex: nextActive };
}

export function closeAllTabs(): { tabs: Tab[]; activeIndex: number } {
  return { tabs: [], activeIndex: 0 };
}

export function closeOtherTabs(tabs: Tab[], index: number): { tabs: Tab[]; activeIndex: number } {
  return { tabs: [tabs[index]], activeIndex: 0 };
}
