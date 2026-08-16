//! Teams and analysis sharing (read access via team membership).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TeamStore {
    memory: Arc<Mutex<MemoryTeams>>,
    postgres: Option<PgPool>,
}

#[derive(Default)]
struct MemoryTeams {
    teams: HashMap<Uuid, Team>,
    /// team_id -> (user_id -> role)
    members: HashMap<Uuid, HashMap<Uuid, String>>,
    /// analysis_id -> team_ids
    shares: HashMap<Uuid, HashSet<Uuid>>,
    /// email cache for member listing (user_id -> email)
    emails: HashMap<Uuid, String>,
}

impl TeamStore {
    pub fn new(postgres: Option<PgPool>) -> Self {
        Self {
            memory: Arc::new(Mutex::new(MemoryTeams::default())),
            postgres,
        }
    }

    pub async fn create_team(&self, name: &str, creator_id: Uuid) -> ApiResult<Team> {
        let name = name.trim();
        if name.is_empty() || name.len() > 120 {
            return Err(ApiError::bad_request("team name must be 1–120 characters"));
        }
        let team = Team {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_by: creator_id,
            created_at: Utc::now(),
        };
        if let Some(pool) = &self.postgres {
            sqlx::query(
                "INSERT INTO teams (id, name, created_by, created_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(team.id)
            .bind(&team.name)
            .bind(team.created_by)
            .bind(team.created_at)
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            sqlx::query(
                "INSERT INTO team_members (team_id, user_id, role, created_at) VALUES ($1, $2, 'owner', $3)",
            )
            .bind(team.id)
            .bind(creator_id)
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        } else {
            let mut mem = self.memory.lock().await;
            mem.teams.insert(team.id, team.clone());
            let mut members = HashMap::new();
            members.insert(creator_id, "owner".to_string());
            mem.members.insert(team.id, members);
        }
        Ok(team)
    }

    pub async fn list_teams_for_user(&self, user_id: Uuid) -> ApiResult<Vec<Team>> {
        if let Some(pool) = &self.postgres {
            let rows = sqlx::query_as::<_, TeamRow>(
                r#"
                SELECT t.id, t.name, t.created_by, t.created_at
                FROM teams t
                INNER JOIN team_members m ON m.team_id = t.id
                WHERE m.user_id = $1
                ORDER BY t.created_at DESC
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(rows.into_iter().map(Team::from).collect());
        }
        let mem = self.memory.lock().await;
        let mut teams: Vec<Team> = mem
            .members
            .iter()
            .filter(|(_, members)| members.contains_key(&user_id))
            .filter_map(|(id, _)| mem.teams.get(id).cloned())
            .collect();
        teams.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(teams)
    }

    pub async fn add_member(
        &self,
        team_id: Uuid,
        actor_id: Uuid,
        member_user_id: Uuid,
        member_email: &str,
        role: &str,
    ) -> ApiResult<TeamMember> {
        self.require_member(team_id, actor_id).await?;
        if let Some(pool) = &self.postgres {
            sqlx::query(
                r#"
                INSERT INTO team_members (team_id, user_id, role, created_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (team_id, user_id) DO UPDATE SET role = EXCLUDED.role
                "#,
            )
            .bind(team_id)
            .bind(member_user_id)
            .bind(role)
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        } else {
            let mut mem = self.memory.lock().await;
            if !mem.teams.contains_key(&team_id) {
                return Err(ApiError::not_found("team not found"));
            }
            mem.members
                .entry(team_id)
                .or_default()
                .insert(member_user_id, role.to_string());
            mem.emails
                .insert(member_user_id, member_email.to_string());
        }
        Ok(TeamMember {
            team_id,
            user_id: member_user_id,
            email: member_email.to_string(),
            role: role.to_string(),
            created_at: Utc::now(),
        })
    }

    pub async fn list_members(&self, team_id: Uuid, actor_id: Uuid) -> ApiResult<Vec<TeamMember>> {
        self.require_member(team_id, actor_id).await?;
        if let Some(pool) = &self.postgres {
            let rows = sqlx::query_as::<_, MemberRow>(
                r#"
                SELECT m.team_id, m.user_id, u.email, m.role, m.created_at
                FROM team_members m
                INNER JOIN users u ON u.id = m.user_id
                WHERE m.team_id = $1
                ORDER BY m.created_at ASC
                "#,
            )
            .bind(team_id)
            .fetch_all(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(rows
                .into_iter()
                .map(|r| TeamMember {
                    team_id: r.team_id,
                    user_id: r.user_id,
                    email: r.email,
                    role: r.role,
                    created_at: r.created_at,
                })
                .collect());
        }
        let mem = self.memory.lock().await;
        let Some(members) = mem.members.get(&team_id) else {
            return Err(ApiError::not_found("team not found"));
        };
        Ok(members
            .iter()
            .map(|(uid, role)| TeamMember {
                team_id,
                user_id: *uid,
                email: mem
                    .emails
                    .get(uid)
                    .cloned()
                    .unwrap_or_else(|| uid.to_string()),
                role: role.clone(),
                created_at: Utc::now(),
            })
            .collect())
    }

    pub async fn share_analysis(
        &self,
        analysis_id: Uuid,
        team_id: Uuid,
        shared_by: Uuid,
    ) -> ApiResult<()> {
        self.require_member(team_id, shared_by).await?;
        if let Some(pool) = &self.postgres {
            sqlx::query(
                r#"
                INSERT INTO analysis_shares (analysis_id, team_id, shared_by, created_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (analysis_id, team_id) DO NOTHING
                "#,
            )
            .bind(analysis_id)
            .bind(team_id)
            .bind(shared_by)
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        } else {
            let mut mem = self.memory.lock().await;
            if !mem.teams.contains_key(&team_id) {
                return Err(ApiError::not_found("team not found"));
            }
            mem.shares.entry(analysis_id).or_default().insert(team_id);
        }
        Ok(())
    }

    pub async fn list_shared_analysis_ids(
        &self,
        team_id: Uuid,
        actor_id: Uuid,
    ) -> ApiResult<Vec<Uuid>> {
        self.require_member(team_id, actor_id).await?;
        if let Some(pool) = &self.postgres {
            let rows = sqlx::query_scalar::<_, Uuid>(
                "SELECT analysis_id FROM analysis_shares WHERE team_id = $1 ORDER BY created_at DESC",
            )
            .bind(team_id)
            .fetch_all(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(rows);
        }
        let mem = self.memory.lock().await;
        Ok(mem
            .shares
            .iter()
            .filter(|(_, teams)| teams.contains(&team_id))
            .map(|(id, _)| *id)
            .collect())
    }

    pub async fn analysis_ids_visible_to(&self, user_id: Uuid) -> ApiResult<HashSet<Uuid>> {
        if let Some(pool) = &self.postgres {
            let rows = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT s.analysis_id
                FROM analysis_shares s
                INNER JOIN team_members m ON m.team_id = s.team_id
                WHERE m.user_id = $1
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(rows.into_iter().collect());
        }
        let mem = self.memory.lock().await;
        let team_ids: HashSet<Uuid> = mem
            .members
            .iter()
            .filter(|(_, members)| members.contains_key(&user_id))
            .map(|(id, _)| *id)
            .collect();
        Ok(mem
            .shares
            .iter()
            .filter(|(_, teams)| teams.iter().any(|t| team_ids.contains(t)))
            .map(|(id, _)| *id)
            .collect())
    }

    pub async fn remember_email(&self, user_id: Uuid, email: &str) {
        if self.postgres.is_some() {
            return;
        }
        let mut mem = self.memory.lock().await;
        mem.emails.insert(user_id, email.to_string());
    }

    async fn require_member(&self, team_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        if let Some(pool) = &self.postgres {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            )
            .bind(team_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            if !exists {
                // Distinguish missing team vs not a member
                let team_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM teams WHERE id = $1)",
                )
                .bind(team_id)
                .fetch_one(pool)
                .await
                .map_err(|err| ApiError::internal(err.to_string()))?;
                if !team_exists {
                    return Err(ApiError::not_found("team not found"));
                }
                return Err(ApiError::forbidden("not a team member"));
            }
            return Ok(());
        }
        let mem = self.memory.lock().await;
        if !mem.teams.contains_key(&team_id) {
            return Err(ApiError::not_found("team not found"));
        }
        if mem
            .members
            .get(&team_id)
            .is_some_and(|m| m.contains_key(&user_id))
        {
            Ok(())
        } else {
            Err(ApiError::forbidden("not a team member"))
        }
    }
}

#[derive(sqlx::FromRow)]
struct TeamRow {
    id: Uuid,
    name: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            created_by: row.created_by,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    team_id: Uuid,
    user_id: Uuid,
    email: String,
    role: String,
    created_at: DateTime<Utc>,
}
