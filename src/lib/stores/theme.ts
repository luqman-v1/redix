type Theme = "dark" | "light";

const STORAGE_KEY = "redix-theme";

function getInitial(): Theme {
  if (typeof window === "undefined") return "dark";
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "light" || stored === "dark" ? stored : "dark";
}

function apply(theme: Theme) {
  document.documentElement.classList.remove("dark", "light");
  document.documentElement.classList.add(theme);
  localStorage.setItem(STORAGE_KEY, theme);
}

function createThemeStore() {
  let current = getInitial();
  apply(current);

  const subscribers = new Set<(value: Theme) => void>();

  function notify() {
    for (const fn of subscribers) fn(current);
  }

  return {
    subscribe(fn: (value: Theme) => void) {
      subscribers.add(fn);
      fn(current);
      return () => subscribers.delete(fn);
    },
    set(theme: Theme) {
      current = theme;
      apply(current);
      notify();
    },
    toggle() {
      current = current === "dark" ? "light" : "dark";
      apply(current);
      notify();
    },
  };
}

export const theme = createThemeStore();
