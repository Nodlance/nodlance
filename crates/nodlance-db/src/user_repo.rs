//! PostgreSQL implementation of UserRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{DomainError, NewUser, Permission, Role, User, UserRepository};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    password_hash: String,
    display_name: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            display_name: row.display_name,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(FromRow)]
struct RoleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
        }
    }
}

impl UserRepository for PgUserRepository {
    async fn create(&self, new_user: &NewUser, password_hash: &str) -> Result<User, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, username, password_hash, display_name, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, now(), now())
            RETURNING id, username, password_hash, display_name, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&new_user.username)
        .bind(password_hash)
        .bind(new_user.display_name.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create user"))?;

        Ok(row.into())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, display_name, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get user by id"))?;

        Ok(row.map(Into::into))
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, display_name, is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get user by username"))?;

        Ok(row.map(Into::into))
    }

    async fn list(&self) -> Result<Vec<User>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, display_name, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list users"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, DomainError> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT r.id, r.name, r.description
            FROM roles r
            INNER JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            ORDER BY r.name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get user roles"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>, DomainError> {
        #[derive(FromRow)]
        struct PermRow {
            name: String,
        }

        let rows = sqlx::query_as::<_, PermRow>(
            r#"
            SELECT DISTINCT p.name
            FROM permissions p
            INNER JOIN role_permissions rp ON rp.permission_id = p.id
            INNER JOIN user_roles ur ON ur.role_id = rp.role_id
            WHERE ur.user_id = $1
            ORDER BY p.name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get user permissions"))?;

        rows.into_iter()
            .map(|r| {
                r.name.parse::<Permission>().map_err(|err| {
                    DomainError::Internal(format!("invalid permission in db: {err}"))
                })
            })
            .collect()
    }

    async fn assign_role(&self, user_id: Uuid, role_name: &str) -> Result<(), DomainError> {
        let role = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description
            FROM roles
            WHERE name = $1
            "#,
        )
        .bind(role_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "find role by name"))?
        .ok_or_else(|| DomainError::NotFound(format!("role not found: {role_name}")))?;

        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(role.id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "assign role"))?;

        Ok(())
    }
}
