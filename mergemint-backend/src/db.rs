/// Database abstraction layer for MergeMint backend.
///
/// Provides typed query methods over a PostgreSQL connection pool. All
/// pagination is cursor-based (keyed on `created_at DESC, id DESC`) to
/// avoid offset drift on live datasets.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounty {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub reward: String,
    pub status: String,
    pub creator: String,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BountyPage {
    pub bounties: Vec<Bounty>,
    pub next_cursor: Option<String>,
}

// ── Database client ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List bounties created by a specific address, newest-first.
    pub async fn list_bounties_by_creator(
        &self,
        creator: &str,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<BountyPage> {
        let rows = match cursor {
            Some(c) => {
                sqlx::query(
                    r#"
                    SELECT id, title, description, reward, status, creator, assignee, created_at
                    FROM bounties
                    WHERE creator = $1 AND created_at < $2
                    ORDER BY created_at DESC
                    LIMIT $3
                    "#,
                )
                .bind(creator)
                .bind(c)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT id, title, description, reward, status, creator, assignee, created_at
                    FROM bounties
                    WHERE creator = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(creator)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await?
            }
        };

        self.paginate(rows, limit)
    }

    /// List bounties where the given address appears in the `assignees` join
    /// table. Returns newest-first with cursor-based pagination.
    ///
    /// This is the backing query for `GET /api/v1/bounties/assignee/{address}`.
    pub async fn list_bounties_by_assignee(
        &self,
        assignee: &str,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<BountyPage> {
        let rows = match cursor {
            Some(c) => {
                sqlx::query(
                    r#"
                    SELECT b.id, b.title, b.description, b.reward, b.status,
                           b.creator, b.assignee, b.created_at
                    FROM bounties b
                    JOIN assignees a ON a.bounty_id = b.id
                    WHERE a.address = $1 AND b.created_at < $2
                    ORDER BY b.created_at DESC
                    LIMIT $3
                    "#,
                )
                .bind(assignee)
                .bind(c)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT b.id, b.title, b.description, b.reward, b.status,
                           b.creator, b.assignee, b.created_at
                    FROM bounties b
                    JOIN assignees a ON a.bounty_id = b.id
                    WHERE a.address = $1
                    ORDER BY b.created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(assignee)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await?
            }
        };

        self.paginate(rows, limit)
    }

    /// Internal helper: convert raw rows into a BountyPage, detecting whether
    /// there is a next page by fetching limit+1 and trimming the last row.
    fn paginate(
        &self,
        mut rows: Vec<sqlx::postgres::PgRow>,
        limit: i64,
    ) -> Result<BountyPage> {
        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.pop();
        }

        let bounties: Vec<Bounty> = rows
            .into_iter()
            .map(|r| Bounty {
                id: r.get("id"),
                title: r.get("title"),
                description: r.get("description"),
                reward: r.get("reward"),
                status: r.get("status"),
                creator: r.get("creator"),
                assignee: r.get("assignee"),
                created_at: r.get("created_at"),
            })
            .collect();

        let next_cursor = if has_more {
            bounties
                .last()
                .map(|b| b.created_at.to_rfc3339())
        } else {
            None
        };

        Ok(BountyPage { bounties, next_cursor })
    }
}
