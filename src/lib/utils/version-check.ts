const RELEASES_ENDPOINT =
  "https://api.github.com/repos/luqmannulhakim/redix/releases/latest";
const CACHE_KEY = "redix_update_check";
const CACHE_TTL_MS = 6 * 60 * 60 * 1000;

export interface UpdateInfo {
  current: string;
  latest: string;
  url: string;
}

interface ReleasePayload {
  tag_name?: string;
  html_url?: string;
  draft?: boolean;
  prerelease?: boolean;
}

function parseVersion(v: string): [number, number, number] | null {
  const match = v.trim().replace(/^v/i, "").match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) return null;
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

/// Returns -1 when a < b, 1 when a > b, 0 when equal. Returns null when either
/// side is not a recognisable x.y.z version.
export function compareVersions(a: string, b: string): number | null {
  const left = parseVersion(a);
  const right = parseVersion(b);
  if (!left || !right) return null;
  for (let i = 0; i < 3; i++) {
    if (left[i] !== right[i]) return left[i] < right[i] ? -1 : 1;
  }
  return 0;
}

interface CachedProbe {
  latest: string;
  url: string;
  checkedAt: number;
}

function readCache(): CachedProbe | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<CachedProbe>;
    if (typeof parsed?.latest !== "string" || typeof parsed?.checkedAt !== "number") {
      return null;
    }
    return { latest: parsed.latest, url: parsed.url ?? "", checkedAt: parsed.checkedAt };
  } catch {
    return null;
  }
}

/// Resolves to the newer release when the latest tag is ahead of `current`,
/// otherwise null. Any failure (offline, rate limit, malformed payload)
/// resolves to null so the app is never blocked by a failed check.
export async function checkForUpdate(
  current: string,
  now: number = Date.now(),
): Promise<UpdateInfo | null> {
  const cached = readCache();
  const fresh = cached !== null && now - cached.checkedAt < CACHE_TTL_MS;
  if (cached && fresh) {
    return toUpdateInfo(current, cached);
  }

  let probe: CachedProbe = { latest: current, url: "", checkedAt: now };
  try {
    const res = await fetch(RELEASES_ENDPOINT, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (res.ok) {
      const payload = (await res.json()) as ReleasePayload;
      const tag = payload.tag_name ?? "";
      if (tag && payload.html_url && !payload.draft && !payload.prerelease) {
        probe = { latest: tag, url: payload.html_url, checkedAt: now };
      }
    }
  } catch {
    // Fall through: the empty url marks this probe as unsuccessful.
  }

  // Cache every probe, successful or not, so a failing or rate-limited
  // endpoint is not retried on every launch.
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify(probe));
  } catch {
    /* storage unavailable; skip caching */
  }

  return toUpdateInfo(current, probe);
}

function toUpdateInfo(current: string, probe: CachedProbe): UpdateInfo | null {
  if (!probe.url) return null;
  // An update exists only when the published tag is ahead of what is running.
  if (compareVersions(current, probe.latest) !== -1) return null;
  // Tags arrive as "v0.9.0"; the UI renders "v{latest}", so drop the prefix.
  return { current, latest: probe.latest.replace(/^v/i, ""), url: probe.url };
}
