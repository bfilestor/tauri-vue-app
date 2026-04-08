use crate::db::{read_active_conversation_id, read_active_member_scope, MemberScope};
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemberScopeInput {
    pub owner_user_id: Option<String>,
    pub member_id: Option<String>,
    pub member_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChatScopeInput {
    pub owner_user_id: Option<String>,
    pub member_id: Option<String>,
    pub member_name: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMemberScope {
    pub owner_user_id: String,
    pub member_id: String,
    pub member_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedChatScope {
    pub owner_user_id: String,
    pub member_id: String,
    pub member_name: String,
    pub conversation_id: String,
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub fn resolve_member_scope(
    conn: &Connection,
    scope: Option<MemberScopeInput>,
) -> Result<ResolvedMemberScope, String> {
    let explicit_owner_user_id = normalize_optional_text(
        scope
            .as_ref()
            .and_then(|item| item.owner_user_id.clone()),
    );
    let explicit_member_id = normalize_optional_text(
        scope
            .as_ref()
            .and_then(|item| item.member_id.clone()),
    );
    let explicit_member_name = normalize_optional_text(
        scope
            .as_ref()
            .and_then(|item| item.member_name.clone()),
    )
    .unwrap_or_default();

    match (explicit_owner_user_id, explicit_member_id) {
        (Some(owner_user_id), Some(member_id)) => Ok(ResolvedMemberScope {
            owner_user_id,
            member_id,
            member_name: explicit_member_name,
        }),
        _ => {
            let fallback_scope = read_active_member_scope(conn)
                .map_err(|e| format!("读取当前成员上下文失败: {}", e))?;

            let MemberScope {
                owner_user_id,
                member_id,
            } = fallback_scope.ok_or_else(|| "当前用户或成员上下文缺失".to_string())?;

            Ok(ResolvedMemberScope {
                owner_user_id,
                member_id,
                member_name: explicit_member_name,
            })
        }
    }
}

pub fn resolve_chat_scope(
    conn: &Connection,
    scope: Option<ChatScopeInput>,
) -> Result<ResolvedChatScope, String> {
    let resolved_member_scope = resolve_member_scope(
        conn,
        scope.as_ref().map(|item| MemberScopeInput {
            owner_user_id: item.owner_user_id.clone(),
            member_id: item.member_id.clone(),
            member_name: item.member_name.clone(),
        }),
    )?;

    let explicit_conversation_id = normalize_optional_text(
        scope
            .as_ref()
            .and_then(|item| item.conversation_id.clone()),
    );
    let fallback_conversation_id = read_active_conversation_id(conn)
        .map_err(|e| format!("读取当前会话上下文失败: {}", e))?;

    let conversation_id = ensure_member_conversation(
        conn,
        &resolved_member_scope.owner_user_id,
        &resolved_member_scope.member_id,
        explicit_conversation_id.or(fallback_conversation_id),
    )?;

    Ok(ResolvedChatScope {
        owner_user_id: resolved_member_scope.owner_user_id,
        member_id: resolved_member_scope.member_id,
        member_name: resolved_member_scope.member_name,
        conversation_id,
    })
}

pub fn ensure_member_conversation(
    conn: &Connection,
    owner_user_id: &str,
    member_id: &str,
    preferred_conversation_id: Option<String>,
) -> Result<String, String> {
    if let Some(conversation_id) = normalize_optional_text(preferred_conversation_id) {
        let existing_binding = conn
            .query_row(
                "SELECT owner_user_id, member_id
                 FROM chat_conversations
                 WHERE id = ?1",
                rusqlite::params![&conversation_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|e| format!("查询会话失败: {}", e))?;

        match existing_binding {
            Some((existing_owner_user_id, existing_member_id))
                if existing_owner_user_id == owner_user_id && existing_member_id == member_id =>
            {
                return Ok(conversation_id);
            }
            Some(_) => {}
            None => {
                let now = chrono::Local::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO chat_conversations (id, owner_user_id, member_id, title, created_at, updated_at)
                     VALUES (?1, ?2, ?3, '默认会话', ?4, ?5)",
                    rusqlite::params![conversation_id, owner_user_id, member_id, now, now],
                )
                .map_err(|e| format!("创建会话失败: {}", e))?;

                return Ok(conversation_id);
            }
        }
    }

    let existing_id = conn
        .query_row(
            "SELECT id
             FROM chat_conversations
             WHERE owner_user_id = ?1 AND member_id = ?2
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
            rusqlite::params![owner_user_id, member_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| format!("查询会话失败: {}", e))?;

    if let Some(conversation_id) = existing_id {
        return Ok(conversation_id);
    }

    let conversation_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO chat_conversations (id, owner_user_id, member_id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, '默认会话', ?4, ?5)",
        rusqlite::params![conversation_id, owner_user_id, member_id, now, now],
    )
    .map_err(|e| format!("创建会话失败: {}", e))?;

    Ok(conversation_id)
}

pub fn touch_conversation(conn: &Connection, conversation_id: &str) -> Result<(), String> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE chat_conversations SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, conversation_id],
    )
    .map_err(|e| format!("更新会话时间失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        CONFIG_KEY_ACTIVE_CONVERSATION_ID, CONFIG_KEY_ACTIVE_MEMBER_ID,
        CONFIG_KEY_ACTIVE_OWNER_USER_ID,
    };
    use rusqlite::Result;

    fn create_test_conn() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "
            CREATE TABLE system_config (
                id TEXT PRIMARY KEY,
                config_key TEXT NOT NULL UNIQUE,
                config_value TEXT DEFAULT '',
                updated_at TEXT NOT NULL
            );

            CREATE TABLE chat_conversations (
                id TEXT PRIMARY KEY,
                owner_user_id TEXT NOT NULL,
                member_id TEXT NOT NULL,
                title TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(conn)
    }

    fn insert_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO system_config (id, config_key, config_value, updated_at)
             VALUES (?1, ?2, ?3, '2026-04-08T16:30:00+08:00')",
            rusqlite::params![format!("cfg-{key}"), key, value],
        )?;
        Ok(())
    }

    fn insert_conversation(
        conn: &Connection,
        id: &str,
        owner_user_id: &str,
        member_id: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO chat_conversations (id, owner_user_id, member_id, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, '默认会话', '2026-04-08T16:30:00+08:00', '2026-04-08T16:30:00+08:00')",
            rusqlite::params![id, owner_user_id, member_id],
        )?;
        Ok(())
    }

    fn count_member_conversations(
        conn: &Connection,
        owner_user_id: &str,
        member_id: &str,
    ) -> Result<i64> {
        conn.query_row(
            "SELECT COUNT(*)
             FROM chat_conversations
             WHERE owner_user_id = ?1 AND member_id = ?2",
            rusqlite::params![owner_user_id, member_id],
            |row| row.get(0),
        )
    }

    #[test]
    fn resolve_member_scope_prefers_explicit_values_over_fallback_context() -> Result<()> {
        let conn = create_test_conn()?;
        insert_config(&conn, CONFIG_KEY_ACTIVE_OWNER_USER_ID, "1001")?;
        insert_config(&conn, CONFIG_KEY_ACTIVE_MEMBER_ID, "2001")?;

        let resolved = resolve_member_scope(
            &conn,
            Some(MemberScopeInput {
                owner_user_id: Some("3001".to_string()),
                member_id: Some("4001".to_string()),
                member_name: Some("本人".to_string()),
            }),
        )
        .expect("explicit scope should win");

        assert_eq!(
            resolved,
            ResolvedMemberScope {
                owner_user_id: "3001".to_string(),
                member_id: "4001".to_string(),
                member_name: "本人".to_string(),
            }
        );

        Ok(())
    }

    #[test]
    fn resolve_chat_scope_falls_back_to_active_context_and_reuses_member_conversation() -> Result<()> {
        let conn = create_test_conn()?;
        insert_config(&conn, CONFIG_KEY_ACTIVE_OWNER_USER_ID, "1001")?;
        insert_config(&conn, CONFIG_KEY_ACTIVE_MEMBER_ID, "2001")?;
        insert_conversation(&conn, "conv-2001", "1001", "2001")?;
        insert_config(&conn, CONFIG_KEY_ACTIVE_CONVERSATION_ID, "conv-2001")?;

        let resolved = resolve_chat_scope(&conn, None).expect("fallback scope should resolve");

        assert_eq!(resolved.owner_user_id, "1001");
        assert_eq!(resolved.member_id, "2001");
        assert_eq!(resolved.conversation_id, "conv-2001");
        assert_eq!(count_member_conversations(&conn, "1001", "2001")?, 1);

        Ok(())
    }

    #[test]
    fn resolve_chat_scope_creates_independent_default_conversations_per_member() -> Result<()> {
        let conn = create_test_conn()?;

        let first = resolve_chat_scope(
            &conn,
            Some(ChatScopeInput {
                owner_user_id: Some("1001".to_string()),
                member_id: Some("2001".to_string()),
                member_name: Some("本人".to_string()),
                conversation_id: None,
            }),
        )
        .expect("first member scope should resolve");
        let second = resolve_chat_scope(
            &conn,
            Some(ChatScopeInput {
                owner_user_id: Some("1001".to_string()),
                member_id: Some("2002".to_string()),
                member_name: Some("母亲".to_string()),
                conversation_id: None,
            }),
        )
        .expect("second member scope should resolve");
        let first_again = resolve_chat_scope(
            &conn,
            Some(ChatScopeInput {
                owner_user_id: Some("1001".to_string()),
                member_id: Some("2001".to_string()),
                member_name: Some("本人".to_string()),
                conversation_id: None,
            }),
        )
        .expect("first member conversation should be reused");

        assert_ne!(first.conversation_id, second.conversation_id);
        assert_eq!(first_again.conversation_id, first.conversation_id);
        assert_eq!(count_member_conversations(&conn, "1001", "2001")?, 1);
        assert_eq!(count_member_conversations(&conn, "1001", "2002")?, 1);

        Ok(())
    }

    #[test]
    fn resolve_chat_scope_rejects_cross_member_conversation_reuse() -> Result<()> {
        let conn = create_test_conn()?;
        insert_conversation(&conn, "conv-shared", "1001", "2001")?;

        let resolved = resolve_chat_scope(
            &conn,
            Some(ChatScopeInput {
                owner_user_id: Some("1001".to_string()),
                member_id: Some("2002".to_string()),
                member_name: Some("父亲".to_string()),
                conversation_id: Some("conv-shared".to_string()),
            }),
        )
        .expect("foreign conversation id should not break current member scope");

        assert_ne!(resolved.conversation_id, "conv-shared");
        assert_eq!(count_member_conversations(&conn, "1001", "2001")?, 1);
        assert_eq!(count_member_conversations(&conn, "1001", "2002")?, 1);

        Ok(())
    }
}
