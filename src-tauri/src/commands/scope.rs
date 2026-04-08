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
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM chat_conversations
                    WHERE id = ?1 AND owner_user_id = ?2 AND member_id = ?3
                )",
                rusqlite::params![conversation_id, owner_user_id, member_id],
                |row| row.get::<_, i32>(0),
            )
            .map_err(|e| format!("查询会话失败: {}", e))?
            == 1;

        if exists {
            return Ok(conversation_id);
        }

        let now = chrono::Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chat_conversations (id, owner_user_id, member_id, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, '默认会话', ?4, ?5)",
            rusqlite::params![conversation_id, owner_user_id, member_id, now, now],
        )
        .map_err(|e| format!("创建会话失败: {}", e))?;

        return Ok(conversation_id);
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
