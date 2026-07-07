# Phase 0: Project Scaffold

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bootstrap Tauri v2 + Svelte + TailwindCSS project that launches on macOS/Windows/Linux.

**Architecture:** Tauri v2 backend (Rust) with Svelte frontend. TailwindCSS for styling with CSS custom properties for theme tokens.

**Tech Stack:** Tauri v2, Svelte 5, TailwindCSS v4, Vite, TypeScript

## Global Constraints
- Node.js 20+, Rust stable (1.77+)
- pnpm as package manager
- All files under 800 lines, functions under 50 lines

---

## Task 1: Init Tauri v2 + Svelte Project

**Files:**
- Create: `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`
- Create: `src/`, `src-tauri/`
- Create: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`
- Create: `src/app.html`, `src/routes/+page.svelte`, `src/app.css`

**Steps:**

- [ ] **Step 1: Scaffold project**

```bash
pnpm create tauri-app redix --template svelte-ts
cd redix
```

- [ ] **Step 2: Install dependencies**

```bash
pnpm install
```

- [ ] **Step 3: Verify dev starts**

```bash
pnpm tauri dev
```

Expected: Blank Svelte app in Tauri window.

- [ ] **Step 4: Commit**

```bash
git init && git add . && git commit -m "feat: init tauri v2 + svelte project"
```

---

## Task 2: TailwindCSS + Theme Tokens

**Files:**
- Modify: `vite.config.ts` (add tailwindcss plugin)
- Modify: `src/app.css` (theme tokens)

**Interfaces:**
- Produces: CSS custom properties `--color-bg`, `--color-fg`, `--color-surface`, `--color-border`, `--color-accent`, `--color-muted`, `--color-success`, `--color-error`, `--color-warning`
- Produces: `.dark` and `.light` class on `<html>`

**Steps:**

- [ ] **Step 1: Install TailwindCSS**

```bash
pnpm add -D tailwindcss @tailwindcss/vite
```

- [ ] **Step 2: Add plugin to vite config**

```ts
// vite.config.ts
import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
});
```

- [ ] **Step 3: Define theme tokens**

```css
/* src/app.css */
@import "tailwindcss";

:root {
  --color-bg: #1e1e2e;
  --color-fg: #cdd6f4;
  --color-surface: #313244;
  --color-border: #45475a;
  --color-accent: #89b4fa;
  --color-muted: #6c7086;
  --color-success: #a6e3a1;
  --color-error: #f38ba8;
  --color-warning: #fab387;
}

.light {
  --color-bg: #eff1f5;
  --color-fg: #4c4f69;
  --color-surface: #ccd0da;
  --color-border: #bcc0cc;
  --color-accent: #1e66f5;
  --color-muted: #8c8fa1;
  --color-success: #40a02b;
  --color-error: #d20f39;
  --color-warning: #df8e1d;
}

body {
  background-color: var(--color-bg);
  color: var(--color-fg);
  margin: 0;
  font-family: system-ui, -apple-system, sans-serif;
}
```

- [ ] **Step 4: Verify**

```bash
pnpm tauri dev
```

Expected: Dark background with correct token colors.

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: tailwindcss setup with dark/light theme tokens"
```

---

## Task 3: Theme Toggle + Persistence

**Files:**
- Create: `src/lib/stores/theme.ts`
- Create: `src/lib/components/ThemeToggle.svelte`
- Modify: `src/routes/+page.svelte` (add toggle)

**Interfaces:**
- Produces: `theme` store — `Writable<'dark' | 'light'>`
- Produces: `toggleTheme()` function
- Produces: `<ThemeToggle />` component

**Steps:**

- [ ] **Step 1: Write theme store**

```ts
// src/lib/stores/theme.ts
import { writable } from "svelte/store";

type Theme = "dark" | "light";

function createThemeStore() {
  const stored = (typeof localStorage !== "undefined"
    ? localStorage.getItem("theme")
    : null) as Theme | null;
  const initial: Theme = stored ?? "dark";
  const { subscribe, set, update } = writable<Theme>(initial);

  function apply(theme: Theme) {
    document.documentElement.classList.remove("dark", "light");
    document.documentElement.classList.add(theme);
    localStorage.setItem("theme", theme);
  }

  apply(initial);

  return {
    subscribe,
    toggle() {
      update((t) => {
        const next: Theme = t === "dark" ? "light" : "dark";
        apply(next);
        return next;
      });
    },
    set(theme: Theme) {
      apply(theme);
      set(theme);
    },
  };
}

export const theme = createThemeStore();
```

- [ ] **Step 2: Write ThemeToggle component**

```svelte
<!-- src/lib/components/ThemeToggle.svelte -->
<script lang="ts">
  import { theme } from "$lib/stores/theme";
</script>

<button
  onclick={() => theme.toggle()}
  class="rounded-md p-2 hover:opacity-80 transition-opacity cursor-pointer"
  style:background-color="var(--color-surface)"
  style:color="var(--color-fg)"
  aria-label="Toggle theme"
>
  {#if $theme === "dark"}🌙{:else}☀️{/if}
</button>
```

- [ ] **Step 3: Add to page**

```svelte
<!-- src/routes/+page.svelte -->
<script lang="ts">
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
</script>

<div class="min-h-screen p-8" style:background-color="var(--color-bg)" style:color="var(--color-fg)">
  <div class="flex justify-end">
    <ThemeToggle />
  </div>
  <h1 class="text-2xl font-bold mt-4">Redix</h1>
  <p style:color="var(--color-muted)">Redis GUI Client</p>
</div>
```

- [ ] **Step 4: Verify**

Click toggle → switches dark/light. Restart → theme persists.

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: theme toggle with localStorage persistence"
```

---

## Task 4: Three-Panel Layout Shell

**Files:**
- Create: `src/lib/components/Panel.svelte`
- Create: `src/lib/components/Layout.svelte`
- Modify: `src/routes/+page.svelte` (use layout)

**Interfaces:**
- Produces: `<Layout>` with 3 named slots: `sidebar`, `main`, `bottom`
- Produces: `<Panel>` — resizable panel with drag handle
- Props: `direction: 'horizontal' | 'vertical'`, `initialSize: number`, `minSize: number`, `maxSize: number`

**Steps:**

- [ ] **Step 1: Write Panel component**

```svelte
<!-- src/lib/components/Panel.svelte -->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    initialSize = 250,
    minSize = 150,
    maxSize = 500,
    direction = "horizontal",
    children,
  }: {
    initialSize?: number;
    minSize?: number;
    maxSize?: number;
    direction?: "horizontal" | "vertical";
    children: Snippet;
  } = $props();

  let size = $state(initialSize);
  let dragging = $state(false);

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    const pos = direction === "horizontal" ? e.clientX : e.clientY;
    size = Math.min(maxSize, Math.max(minSize, pos));
  }

  function onPointerUp() {
    dragging = false;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="relative flex shrink-0 overflow-hidden"
  class:flex-row={direction === "horizontal"}
  class:flex-col={direction === "vertical"}
  style={direction === "horizontal"
    ? `width: ${size}px`
    : `height: ${size}px`}
  style:background-color="var(--color-bg)"
>
  <div class="flex-1 overflow-auto">
    {@render children()}
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="shrink-0 hover:opacity-80 transition-opacity"
    class:w-1={direction === "horizontal"}
    class:h-1={direction === "vertical"}
    class:cursor-col-resize={direction === "horizontal"}
    class:cursor-row-resize={direction === "vertical"}
    style:background-color="var(--color-border)"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    role="separator"
    tabindex="0"
  ></div>
</div>
```

- [ ] **Step 2: Write Layout component**

```svelte
<!-- src/lib/components/Layout.svelte -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Panel from "./Panel.svelte";

  let { sidebar, main, bottom }: {
    sidebar: Snippet;
    main: Snippet;
    bottom: Snippet;
  } = $props();
</script>

<div class="h-screen flex flex-col overflow-hidden" style:background-color="var(--color-bg)">
  <div class="flex flex-1 overflow-hidden">
    <Panel initialSize={260} minSize={180} maxSize={400} direction="horizontal">
      {@render sidebar()}
    </Panel>
    <div class="flex-1 overflow-auto" style:background-color="var(--color-bg)">
      {@render main()}
    </div>
  </div>
  <Panel initialSize={200} minSize={100} maxSize={500} direction="vertical">
    {@render bottom()}
  </Panel>
</div>
```

- [ ] **Step 3: Wire into page**

```svelte
<!-- src/routes/+page.svelte -->
<script lang="ts">
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import Layout from "$lib/components/Layout.svelte";
</script>

<Layout>
  {#snippet sidebar()}
    <div class="p-4">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-bold" style:color="var(--color-fg)">Connections</h2>
        <ThemeToggle />
      </div>
      <p class="text-sm" style:color="var(--color-muted)">No connections yet</p>
    </div>
  {/snippet}

  {#snippet main()}
    <div class="p-4">
      <p style:color="var(--color-muted)">Select a connection to browse keys</p>
    </div>
  {/snippet}

  {#snippet bottom()}
    <div class="p-4">
      <p class="text-sm" style:color="var(--color-muted)">Command console</p>
    </div>
  {/snippet}
</Layout>
```

- [ ] **Step 4: Verify**

Three panels visible. Drag dividers to resize. Min/max respected.

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: three-panel resizable layout shell"
```
