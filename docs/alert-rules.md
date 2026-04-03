# Alert rules (`alerts.json`)

Alert rules live in a single JSON file. You can maintain them by hand, start from the **example file** in this repository, or use the in-app editor (**`E`**) which saves the same format.

## Where the file is stored

| Role | Path |
|------|------|
| **Canonical** (all new saves from the app) | `{config_dir}/portwatch/alerts.json` |
| **Legacy** (read if present and canonical is missing) | `~/.config/portwatch/alerts.json` |

`config_dir` comes from the [`dirs`](https://docs.rs/dirs/latest/dirs/fn.config_dir.html) crate, e.g.:

- **Linux:** `~/.config/portwatch/alerts.json`
- **macOS:** `~/Library/Application Support/portwatch/alerts.json`
- **Windows:** `%APPDATA%\portwatch\alerts.json` (typically under the user’s Roaming AppData)

Copy the example to the canonical path and adjust:

```bash
# Example (Linux/macOS-style); adjust for your platform’s `config_dir`.
mkdir -p "$(dirname ~/.config/portwatch/alerts.json 2>/dev/null || echo "$HOME/.config/portwatch")"
cp examples/alerts.example.json ~/.config/portwatch/alerts.json
```

On macOS with a fresh install, prefer copying into `~/Library/Application Support/portwatch/alerts.json` so it matches where the app saves.

## Top-level shape

```json
{
  "rules": [ /* array of rule objects */ ]
}
```

An empty file is valid: `{ "rules": [] }`.

## Rule object (`AlertRule`)

Every element of `rules` is an object with **all** of these keys:

| Field | Type | Meaning |
|-------|------|--------|
| `id` | string | Stable identifier for the rule (used for cooldown tracking). Any unique string is fine in hand-edited JSON; the TUI may regenerate this when you save from the editor. |
| `name` | string | Short human-readable label (shown in notifications / UI). |
| `condition` | object | See [Conditions](#conditions) below. |
| `enabled` | boolean | If `false`, the rule is ignored until enabled again. |
| `severity` | string | One of: `Info`, `Warning`, `Critical` (exact casing as in JSON). |
| `cooldown_seconds` | number | Minimum seconds between two firings of the **same** rule (`id`). Must be &gt; 0. |

## Conditions

Conditions use Serde’s **adjacent tagging**: a `type` string and a `params` object (or `null` for variants with no fields).

```json
"condition": {
  "type": "<VariantName>",
  "params": { /* variant-specific fields */ }
}
```

### `PortOpened`

Fires when the given **local** port was not present in the previous scan but is present now.

| `params` field | Type | Meaning |
|----------------|------|--------|
| `port` | integer | TCP/UDP port **0–65535**. |

### `PortClosed`

Fires when the given port was present in the previous scan but is gone now.

| `params` field | Type | Meaning |
|----------------|------|--------|
| `port` | integer | Port **0–65535**. |

### `PortRangeActivity`

Fires when a **new** listening/activity appears on a port inside `[start_port, end_port]` (compared to the previous scan).

| `params` field | Type | Meaning |
|----------------|------|--------|
| `start_port` | integer | Range start (inclusive). |
| `end_port` | integer | Range end (inclusive); must be ≥ `start_port`. |

### `ExternalConnection`

Fires when a row has a **remote** address whose string matches `ip_pattern`, subject to `exclude_private`.

| `params` field | Type | Meaning |
|----------------|------|--------|
| `ip_pattern` | string | [Rust regex](https://docs.rs/regex/latest/regex/) pattern matched against the remote IP string (IPv4 or IPv6). |
| `exclude_private` | boolean | If `true`, skip addresses considered private/loopback (see implementation in `evaluator.rs`). |

### `ProcessCpuThreshold`

Evaluated for processes tied to port rows; uses sampled CPU% from the backend.

| `params` field | Type | Meaning |
|----------------|------|--------|
| `process_pattern` | string | Regex matched against the **process name**. |
| `threshold_percent` | number | Fire when CPU **exceeds** this value (float). |

### `ProcessMemoryThreshold`

Same idea as CPU, using resident memory.

| `params` field | Type | Meaning |
|----------------|------|--------|
| `process_pattern` | string | Regex matched against the **process name**. |
| `threshold_mb` | integer | Fire when RSS **exceeds** this many **mebibytes** (1024×1024 bytes). |

### `UnknownProcessListening`

No parameters.

```json
"condition": {
  "type": "UnknownProcessListening",
  "params": null
}
```

**Note:** This variant is accepted in the config file and in the TUI, but **no alert is emitted for it yet** in the current evaluator (reserved for future use). You can keep it disabled or omit it until implemented.

## Example

See **[`examples/alerts.example.json`](../examples/alerts.example.json)** in the repository for a filled-out file with one rule per condition type.

## Validation

The repository test suite checks that `examples/alerts.example.json` parses successfully, so the example stays in sync with the schema.
