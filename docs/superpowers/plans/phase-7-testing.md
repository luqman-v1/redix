# Phase 7: Testing & Optimization

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement task-by-task.

**Goal:** 80%+ test coverage, integration tests with Docker Redis, performance validation.

---

## Task 25: Rust Backend Unit Tests

**Files:**
- Modify: existing test modules in `src-tauri/src/config/` and `src-tauri/src/redis/`

**Coverage targets:**
- `ConnectionConfig` model — all fields, serialization roundtrip (already done in Task 5)
- `ConnectionStore` — encrypt/decrypt, CRUD operations (already done in Task 6)
- `RedisValue` — all variant methods (already done in Task 9)
- `buildTree` — namespace splitting, empty input, single key, deeply nested

**Steps:**

- [ ] **Step 1: Add tree builder edge case tests**

```rust
#[test]
fn test_build_tree_empty() {
    let result = build_tree(&[], ":");
    assert!(result.is_empty());
}

#[test]
fn test_build_tree_single_key() {
    let result = build_tree(&["mykey".to_string()], ":");
    assert_eq!(result.len(), 1);
    assert!(result[0].is_leaf);
}

#[test]
fn test_build_tree_nested() {
    let keys = vec![
        "user:1:name".to_string(),
        "user:1:email".to_string(),
        "user:2:name".to_string(),
    ];
    let result = build_tree(&keys, ":");
    assert_eq!(result.len(), 1); // "user" folder
    assert_eq!(result[0].children.len(), 2); // "1" and "2"
}
```

- [ ] **Step 2: Add format detector tests (frontend)**

```ts
// src/lib/utils/__tests__/format-detector.test.ts
import { describe, it, expect } from "vitest";
import { detectFormat } from "../format-detector";

describe("detectFormat", () => {
  it("detects JSON object", () => expect(detectFormat('{"a":1}')).toBe("json"));
  it("detects JSON array", () => expect(detectFormat('[1,2,3]')).toBe("json"));
  it("detects XML", () => expect(detectFormat("<root><a/></root>")).toBe("xml"));
  it("detects binary", () => expect(detectFormat("\x00\x01\x02")).toBe("binary"));
  it("defaults to text", () => expect(detectFormat("hello world")).toBe("text"));
  it("handles empty", () => expect(detectFormat("")).toBe("text"));
});
```

- [ ] **Step 3: Add beautifier tests**

```ts
// src/lib/utils/__tests__/beautifier.test.ts
import { describe, it, expect } from "vitest";
import { beautify } from "../beautifier";

describe("beautify", () => {
  it("pretty-prints JSON", () => {
    const { formatted, format } = beautify('{"a":1,"b":[2]}');
    expect(format).toBe("json");
    expect(formatted).toContain("\n");
  });
  it("hex for binary", () => {
    const { format } = beautify("\x00\x01");
    expect(format).toBe("binary");
  });
});
```

- [ ] **Step 4: Add tree builder tests (frontend)**

```ts
// src/lib/utils/__tests__/tree-builder.test.ts
import { describe, it, expect } from "vitest";
import { buildTree } from "../tree-builder";

describe("buildTree", () => {
  it("returns empty for no keys", () => {
    expect(buildTree([])).toEqual([]);
  });

  it("creates nested folders", () => {
    const tree = buildTree(["a:b:c", "a:b:d", "a:e"]);
    expect(tree).toHaveLength(1); // "a"
    expect(tree[0].children).toHaveLength(2); // "b", "e"
  });

  it("separates leaves from folders", () => {
    const tree = buildTree(["key1", "folder:child"]);
    const leaf = tree.find(n => n.name === "key1");
    const folder = tree.find(n => n.name === "folder");
    expect(leaf?.isLeaf).toBe(true);
    expect(folder?.isLeaf).toBe(false);
  });

  it("handles custom separator", () => {
    const tree = buildTree(["a.b.c"], ".");
    expect(tree).toHaveLength(1);
    expect(tree[0].name).toBe("a");
  });
});
```

- [ ] **Step 5: Run all tests**

```bash
cd src-tauri && cargo test
cd .. && pnpm vitest run
```

Expected: All PASS. Check coverage ≥ 80%.

- [ ] **Step 6: Commit**

```bash
git add . && git commit -m "test: add unit tests for backend and frontend utilities"
```

---

## Task 26: Redis Integration Tests

**Files:**
- Create: `src-tauri/tests/redis_integration.rs`
- Create: `docker-compose.test.yml`

**Steps:**

- [ ] **Step 1: Create test docker-compose**

```yaml
# docker-compose.test.yml
services:
  redis-standalone:
    image: redis:7-alpine
    ports: ["6399:6379"]

  redis-cluster:
    image: grokzen/redis-cluster:7.0.0
    ports:
      - "7000:7000"
      - "7001:7001"
      - "7002:7002"
```

- [ ] **Step 2: Write integration tests**

```rust
// src-tauri/tests/redis_integration.rs
// pongolong: requires Docker Redis running

use redix::redis::{StandaloneClient, RedisClient};
use redix::config::ConnectionConfig;

fn test_config() -> ConnectionConfig {
    let mut config = ConnectionConfig::new("test", "127.0.0.1", 6399);
    config
}

#[tokio::test]
async fn test_connect_and_ping() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();
    let result = client.ping().await.unwrap();
    assert_eq!(result.as_str(), Some("PONG"));
}

#[tokio::test]
async fn test_set_get_del() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();

    client.execute("SET", &["test:key", "hello"]).await.unwrap();
    let val = client.execute("GET", &["test:key"]).await.unwrap();
    assert_eq!(val.as_str(), Some("hello"));

    client.del(&["test:key"]).await.unwrap();
    let val = client.execute("GET", &["test:key"]).await.unwrap();
    assert!(val.is_nil());
}

#[tokio::test]
async fn test_scan_keys() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();

    // Setup
    for i in 0..5 {
        client.execute("SET", &["scan:test:{i}", "v"]).await.unwrap();
    }

    let (cursor, keys) = client.scan_keys(0, 100, "scan:test:*").await.unwrap();
    assert!(keys.len() >= 5);

    // Cleanup
    for key in &keys {
        client.del(&[key]).await.unwrap();
    }
}
```

- [ ] **Step 3: Run integration tests**

```bash
docker compose -f docker-compose.test.yml up -d
cd src-tauri && cargo test --test redis_integration
```

Expected: All integration tests PASS.

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "test: redis integration tests with docker compose"
```

---

## Task 27: Performance Optimization

**Files:**
- Modify: `src/lib/components/KeyTree.svelte` (virtual scrolling)

**Steps:**

- [ ] **Step 1: Add virtual scrolling to KeyTree**

Use `svelte-virtual-list` or custom virtual scroll for 10k+ keys.

```bash
pnpm add svelte-virtual-list
```

```svelte
<!-- Replace flat list in KeyTree with virtual list when keys > 1000 -->
{#if keys.length > 1000}
  <VirtualList items={flatNodes} height="600" itemHeight={28} let:item>
    <TreeNodeComponent node={item} {onselect} />
  </VirtualList>
{:else}
  {#each tree as node (node.name)}
    <TreeNodeComponent {node} {onselect} />
  {/each}
{/if}
```

- [ ] **Step 2: Lazy value loading**

Only fetch value when key is selected, not on tree load. Already implemented in Phase 4 via `$effect`.

- [ ] **Step 3: Connection pool tuning**

```rust
// In StandaloneClient, use connection-manager feature
// redis crate handles pooling internally with MultiplexedConnection
```

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "perf: virtual scrolling for large key lists"
```

---

## Task 28: E2E Smoke Test

**Files:**
- Create: `tests/e2e.spec.ts`

**Steps:**

- [ ] **Step 1: Write E2E test**

```ts
// tests/e2e.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Redix E2E", () => {
  test("app launches and shows layout", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await expect(page.locator("text=Connections")).toBeVisible();
    await expect(page.locator("text=No connections yet")).toBeVisible();
  });

  test("add connection flow", async ({ page }) => {
    await page.goto("http://localhost:1420");
    await page.click("text=+ Add");
    await page.fill('input[placeholder*="Name"]', "Local Redis");
    await page.fill('input[placeholder*="Host"]', "localhost");
    await page.fill('input[placeholder*="Port"]', "6379");
    await page.click("text=Add");
    await expect(page.locator("text=Local Redis")).toBeVisible();
  });

  test("theme toggle persists", async ({ page }) => {
    await page.goto("http://localhost:1420");
    const html = page.locator("html");
    await page.click('[aria-label="Toggle theme"]');
    await expect(html).toHaveClass(/light/);
    await page.reload();
    await expect(html).toHaveClass(/light/);
  });
});
```

- [ ] **Step 2: Run E2E tests**

```bash
pnpm tauri dev &
pnpm playwright test
```

- [ ] **Step 3: Commit**

```bash
git add . && git commit -m "test: e2e smoke tests for critical paths"
```
