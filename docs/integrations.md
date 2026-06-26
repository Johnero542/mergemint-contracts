# GitHub App Webhook Delivery — Integration Design

## Overview

When a bounty is completed on-chain, MergeMint must take corresponding off-chain actions: commenting on the associated pull request, closing the linked GitHub issue, and crediting the contributor. This document defines the full pipeline from Stellar event to GitHub App API call, and is the shared reference for the API team, GitHub App team, and contract team.

---

## Pipeline Overview

```
Stellar Network
     │
     │  bounty_completed event
     ▼
Horizon Polling (MergeMint Indexer)
     │
     │  event parsed, bounty_id extracted
     ▼
Indexer Event Processor
     │
     │  look up GitHub PR / issue from bounty metadata
     ▼
GitHub App Webhook Delivery
     │
     │  POST /webhook  (idempotent, keyed on bounty_id)
     ▼
GitHub App
     │
     ├──▶ Comment on PR ("Bounty completed, reward paid")
     ├──▶ Close linked issue
     └──▶ Update project board (optional)
```

---

## Step 1 — Horizon Polling

The MergeMint API polls the Stellar Horizon `/transactions` or `/effects` endpoint for the contract address at a configurable interval (recommended: 5 seconds).

**Filter criteria:**
- `contract_id` = MergeMintContract address
- Event topic = `bounty_completed`

**Relevant event shape (from `src/events.rs`):**

```
topics: (Symbol("bounty_completed"), Address(contributor))
data:   BytesN<32>(bounty_id)
```

The indexer extracts `bounty_id` and `contributor` from each matched event.

---

## Step 2 — Indexer Event Processing

For each `bounty_completed` event the indexer:

1. Checks a **processed events table** (keyed on `bounty_id`) to skip already-handled events (idempotency — see below).
2. Looks up the bounty's associated GitHub PR/issue from the **bounty metadata table**, which was populated when the bounty was created via the off-chain API.
3. Constructs the webhook payload.
4. Delivers the payload to the GitHub App service endpoint.
5. Marks the event as processed in the processed events table.

---

## Step 3 — Webhook Payload

The indexer delivers an HTTP POST to the GitHub App's webhook endpoint.

**Endpoint:** `POST https://app.mergemint.io/webhooks/bounty-completed`

**Headers:**

```
Content-Type: application/json
X-MergeMint-Signature: <HMAC-SHA256 of body with shared secret>
```

**Payload shape:**

```json
{
  "event": "bounty_completed",
  "bounty_id": "<hex-encoded 32-byte bounty ID>",
  "contributor_address": "<Stellar address>",
  "reward_amount": 1000000000,
  "reward_token": "<Stellar token contract address>",
  "github": {
    "pr_number": 42,
    "issue_number": 17,
    "repo": "owner/repo"
  },
  "ledger_sequence": 123456,
  "timestamp": "2026-06-26T23:44:00Z"
}
```

**Field descriptions:**

| Field | Type | Description |
|---|---|---|
| `event` | string | Always `"bounty_completed"` |
| `bounty_id` | string (hex) | Unique on-chain bounty identifier (dedup key) |
| `contributor_address` | string | Stellar address of the rewarded contributor |
| `reward_amount` | integer | Amount in base token units (e.g., stroops) |
| `reward_token` | string | Token contract address |
| `github.pr_number` | integer | GitHub PR number linked to this bounty |
| `github.issue_number` | integer | GitHub issue number (nullable) |
| `github.repo` | string | `owner/repo` slug |
| `ledger_sequence` | integer | Stellar ledger at which the event was emitted |
| `timestamp` | ISO 8601 | Wall-clock time of the ledger close |

---

## Idempotency

Duplicate event delivery is expected (Horizon may return the same event on successive polls before the indexer advances its cursor). The system handles duplicates using `bounty_id` as the deduplication key:

1. Before delivering a webhook, the indexer queries: `SELECT 1 FROM processed_events WHERE bounty_id = $1`.
2. If a row exists, the event is skipped silently.
3. If no row exists, the webhook is delivered and the row is inserted **after a successful 2xx response** from the GitHub App.
4. The insertion is idempotent (upsert with `ON CONFLICT DO NOTHING`).

The GitHub App must also tolerate duplicate delivery by checking its own records for the `bounty_id` before performing GitHub API actions.

---

## Error Handling and Retries

### Indexer → GitHub App delivery failure

| Scenario | Behaviour |
|---|---|
| GitHub App returns 5xx | Retry with exponential back-off (initial: 5 s, max: 5 min, cap: 10 attempts) |
| GitHub App returns 4xx (excl. 409) | Log error, do **not** retry (client error, likely bad payload) |
| GitHub App returns 409 Conflict | Treat as success (already processed) |
| Network timeout | Retry as per 5xx policy |
| All retries exhausted | Move event to dead-letter queue; alert on-call |

The processed events row is **not** inserted until a 2xx is confirmed, ensuring at-least-once delivery semantics.

### GitHub App → GitHub API failure

| Scenario | Behaviour |
|---|---|
| GitHub API rate limit (403/429) | Respect `Retry-After` header; queue action |
| PR/issue not found (404) | Log warning, skip action, mark bounty processed |
| GitHub API 5xx | Retry up to 3 times with jitter |

---

## Security

- All webhook deliveries are signed with an HMAC-SHA256 signature using a shared secret (rotated per environment).
- The GitHub App verifies the `X-MergeMint-Signature` header before processing any payload.
- The shared secret is stored in the GitHub App's secrets manager (not in source code or environment variables in plaintext).

---

## Review Notes

This document should be reviewed and approved by:
- **API team** — polling interval, processed events table schema, retry queue implementation
- **GitHub App team** — webhook endpoint URL, signature verification, idempotency handling
- **Contract team** — event shape accuracy, `bounty_id` encoding
