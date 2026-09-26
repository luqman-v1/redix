import { describe, expect, test } from "bun:test";
import {
  SPECIAL_KEYS,
  closeOtherTabs,
  closeTab,
  openConsoleTab,
  openKeyTab,
  openPubSubTab,
  type Tab,
} from "../src/lib/stores/tabs";

const keyTab = (key: string, type: string | null = "string", ttl: number | null = -1): Tab => ({
  key,
  type,
  ttl,
});

describe("special view tabs", () => {
  test("opening a special tab appends and focuses it", () => {
    const result = openConsoleTab([], 0);
    expect(result.tabs).toHaveLength(1);
    expect(result.tabs[0].key).toBe(SPECIAL_KEYS.console);
    expect(result.activeIndex).toBe(0);
  });

  test("reopening an already open special tab focuses it instead of duplicating", () => {
    const tabs = [keyTab("user:1"), keyTab(SPECIAL_KEYS.console)];
    const result = openConsoleTab(tabs, 0);
    expect(result.tabs).toHaveLength(2);
    expect(result.activeIndex).toBe(1);
  });

  test("special tabs are distinct and tracked separately", () => {
    const withConsole = openConsoleTab([], 0);
    const withPubSub = openPubSubTab(withConsole.tabs, withConsole.activeIndex);
    expect(withPubSub.tabs.map((t) => t.key)).toEqual([
      SPECIAL_KEYS.console,
      SPECIAL_KEYS.pubsub,
    ]);
  });
});

describe("openKeyTab", () => {
  test("a new key is appended and focused", () => {
    const result = openKeyTab([keyTab("a")], 0, "b", "hash", 60);
    expect(result.tabs).toHaveLength(2);
    expect(result.tabs[1]).toEqual({ key: "b", type: "hash", ttl: 60 });
    expect(result.activeIndex).toBe(1);
  });

  test("reopening a key refreshes type and ttl in place", () => {
    const tabs = [keyTab("a", "string", -1), keyTab("b")];
    const result = openKeyTab(tabs, 0, "a", "list", 120);
    expect(result.tabs).toHaveLength(2);
    expect(result.tabs[0]).toEqual({ key: "a", type: "list", ttl: 120 });
    expect(result.activeIndex).toBe(0);
  });

  test("reselecting a key does not disturb the other tabs", () => {
    const tabs = [keyTab("a", "string", 10), keyTab("b", "set", 20)];
    const result = openKeyTab(tabs, 1, "a", "zset", 30);
    expect(result.tabs[1]).toEqual({ key: "b", type: "set", ttl: 20 });
  });
});

describe("closeTab", () => {
  test("closing the last tab clamps the index to zero", () => {
    const result = closeTab([keyTab("a")], 0, 0);
    expect(result.tabs).toHaveLength(0);
    expect(result.activeIndex).toBe(0);
  });

  test("closing the active tab moves focus to the new last tab", () => {
    const tabs = [keyTab("a"), keyTab("b"), keyTab("c")];
    const result = closeTab(tabs, 2, 2);
    expect(result.tabs.map((t) => t.key)).toEqual(["a", "b"]);
    expect(result.activeIndex).toBe(1);
  });

  test("closing a tab before the active one shifts focus left", () => {
    const tabs = [keyTab("a"), keyTab("b"), keyTab("c")];
    const result = closeTab(tabs, 2, 0);
    expect(result.tabs.map((t) => t.key)).toEqual(["b", "c"]);
    expect(result.activeIndex).toBe(1);
  });

  test("closing a tab after the active one keeps focus", () => {
    const tabs = [keyTab("a"), keyTab("b"), keyTab("c")];
    const result = closeTab(tabs, 0, 2);
    expect(result.tabs.map((t) => t.key)).toEqual(["a", "b"]);
    expect(result.activeIndex).toBe(0);
  });

  test("closing never mutates the input array", () => {
    const tabs = [keyTab("a"), keyTab("b")];
    closeTab(tabs, 0, 0);
    expect(tabs).toHaveLength(2);
  });
});

describe("closeOtherTabs", () => {
  test("keeps only the requested tab and focuses it", () => {
    const tabs = [keyTab("a"), keyTab("b"), keyTab("c")];
    const result = closeOtherTabs(tabs, 1);
    expect(result.tabs.map((t) => t.key)).toEqual(["b"]);
    expect(result.activeIndex).toBe(0);
  });
});
