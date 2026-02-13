use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckupRecord {
    pub id: String,
    pub checkup_date: String,
    pub status: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    /// 关联的文件数量（查询时填充）
    pub file_count: Option<i32>,
    /// 关联的项目名称列表（查询时填充）
    pub project_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordInput {
    pub checkup_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordInput {
    pub id: String,
    pub checkup_date: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
}

/// 查询全部检查记录（倒序）
#[tauri::command]
pub fn list_records(db: State<Database>) -> Result<Vec<CheckupRecord>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.checkup_date, r.status, r.notes, r.created_at, r.updated_at,
                    (SELECT COUNT(*) FROM checkup_files WHERE record_id = r.id) as file_count
             FROM checkup_records r
             ORDER BY r.checkup_date DESC, r.created_at DESC"
        )
        .map_err(|e| format!("查询检查记录失败: {}", e))?;

    let records = stmt
        .query_map([], |row| {
            let record_id: String = row.get(0)?;
            Ok((record_id.clone(), CheckupRecord {
                id: record_id,
                checkup_date: row.get(1)?,
                status: row.get(2)?,
                notes: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                file_count: Some(row.get(6)?),
                project_names: None,
            }))
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析失败: {}", e))?;

    // 为每条记录查询关联的项目名称
    let mut result: Vec<CheckupRecord> = Vec::new();
    for (record_id, mut record) in records {
        let mut pstmt = conn
            .prepare(
                "SELECT DISTINCT p.name FROM checkup_files f
                 JOIN checkup_projects p ON f.project_id = p.id
                 WHERE f.record_id = ?1"
            )
            .map_err(|e| format!("查询项目名称失败: {}", e))?;
        let names: Vec<String> = pstmt
            .query_map([&record_id], |row| row.get(0))
            .map_err(|e| format!("查询项目名称失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        record.project_names = Some(names);
        result.push(record);
    }

    Ok(result)
}

/// 创建检查记录
#[tauri::command]
pub fn create_record(input: CreateRecordInput, db: State<Database>) -> Result<CheckupRecord, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let notes = input.notes.unwrap_or_default();

    conn.execute(
        "INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at)
         VALUES (?1, ?2, 'pending_upload', ?3, ?4, ?5)",
        rusqlite::params![id, input.checkup_date, notes, now, now],
    )
    .map_err(|e| format!("创建检查记录失败: {}", e))?;

    Ok(CheckupRecord {
        id,
        checkup_date: input.checkup_date,
        status: "pending_upload".to_string(),
        notes,
        created_at: now.clone(),
        updated_at: now,
        file_count: Some(0),
        project_names: Some(vec![]),
    })
}

/// 更新检查记录
#[tauri::command]
pub fn update_record(input: UpdateRecordInput, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    let now = chrono::Local::now().to_rfc3339();

    let existing = conn.query_row(
        "SELECT checkup_date, notes, status FROM checkup_records WHERE id = ?1",
        [&input.id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
    ).map_err(|e| format!("记录不存在: {}", e))?;

    let date = input.checkup_date.unwrap_or(existing.0);
    let notes = input.notes.unwrap_or(existing.1);
    let status = input.status.unwrap_or(existing.2);

    conn.execute(
        "UPDATE checkup_records SET checkup_date=?1, notes=?2, status=?3, updated_at=?4 WHERE id=?5",
        rusqlite::params![date, notes, status, now, input.id],
    )
    .map_err(|e| format!("更新记录失败: {}", e))?;

    Ok(true)
}

/// 删除检查记录（级联删除关联数据）
#[tauri::command]
pub fn delete_record(id: String, db: State<Database>) -> Result<bool, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 级联删除：indicator_values -> ocr_results -> ai_analyses -> checkup_files -> checkup_records
    conn.execute("DELETE FROM indicator_values WHERE record_id = ?1", [&id])
        .map_err(|e| format!("删除指标值失败: {}", e))?;
    conn.execute("DELETE FROM ocr_results WHERE record_id = ?1", [&id])
        .map_err(|e| format!("删除OCR结果失败: {}", e))?;
    conn.execute("DELETE FROM ai_analyses WHERE record_id = ?1", [&id])
        .map_err(|e| format!("删除AI分析失败: {}", e))?;
    conn.execute("DELETE FROM checkup_files WHERE record_id = ?1", [&id])
        .map_err(|e| format!("删除文件记录失败: {}", e))?;
    conn.execute("DELETE FROM checkup_records WHERE id = ?1", [&id])
        .map_err(|e| format!("删除检查记录失败: {}", e))?;

    Ok(true)
}

use crate::commands::ocr::OcrParsedItem;

#[derive(Debug, Serialize, Clone)]
pub struct HistoryTimelineItem {
    pub id: String,
    pub checkup_date: String,
    pub status: String,
    pub notes: String,
    pub abnormal_items: Vec<AbnormalItem>,
    pub ai_analysis: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AbnormalItem {
    pub project_name: String,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: String,
}

#[tauri::command]
pub fn get_history_timeline(limit: Option<i64>, offset: Option<i64>, db: State<Database>) -> Result<Vec<HistoryTimelineItem>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
    
    let limit_val = limit.unwrap_or(10);
    let offset_val = offset.unwrap_or(0);

    // 1. 获取分页检查记录
    let mut stmt = conn.prepare(
        "SELECT id, checkup_date, status, notes 
         FROM checkup_records 
         ORDER BY checkup_date DESC, created_at DESC
         LIMIT ?1 OFFSET ?2"
    ).map_err(|e| format!("查询记录失败: {}", e))?;
    
    let records = stmt.query_map([limit_val, offset_val], |row| {
        Ok((
            row.get::<_, String>(0)?, // id
            row.get::<_, String>(1)?, // date
            row.get::<_, String>(2)?, // status
            row.get::<_, String>(3)?, // notes
        ))
    }).map_err(|e| format!("查询失败: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("解析失败: {}", e))?;

    let mut result = Vec::new();

    for (id, date, status, notes) in records {
        // 2. 获取该记录的异常指标 (通过 OCR 结果)
        let mut abnormal_items = Vec::new();
        let mut ocr_stmt = conn.prepare(
             "SELECT o.parsed_items, p.name 
              FROM ocr_results o
              LEFT JOIN checkup_projects p ON o.project_id = p.id
              WHERE o.record_id = ?1 AND o.status = 'success'"
        ).map_err(|e| format!("查询OCR失败: {}", e))?;
        
        let ocr_rows = ocr_stmt.query_map([&id], |row| {
             Ok((row.get::<_, String>(0)?, row.get::<_, String>(1).unwrap_or_default()))
        }).map_err(|e| format!("查询OCR失败: {}", e))?;
        
        for row in ocr_rows {
             if let Ok((json, project_name)) = row {
                 if let Ok(items) = serde_json::from_str::<Vec<OcrParsedItem>>(&json) {
                     for item in items {
                         if item.is_abnormal {
                             abnormal_items.push(AbnormalItem {
                                 project_name: project_name.clone(),
                                 name: item.name,
                                 value: item.value,
                                 unit: item.unit,
                                 reference_range: item.reference_range,
                             });
                         }
                     }
                 }
             }
        }

        // 3. 获取最新的 AI 分析结果
        let ai_analysis: Option<String> = conn.query_row(
            "SELECT response_content FROM ai_analyses WHERE record_id = ?1 AND status = 'success' ORDER BY created_at DESC LIMIT 1",
            [&id],
            |row| row.get(0)
        ).ok();

        result.push(HistoryTimelineItem {
            id,
            checkup_date: date,
            status,
            notes,
            abnormal_items,
            ai_analysis,
        });
    }

    Ok(result)
}
/// 获取单条检查记录详情
#[tauri::command]
pub fn get_record(id: String, db: State<Database>) -> Result<CheckupRecord, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    let mut record = conn.query_row(
        "SELECT id, checkup_date, status, notes, created_at, updated_at,
                (SELECT COUNT(*) FROM checkup_files WHERE record_id = ?1) as file_count
         FROM checkup_records WHERE id = ?1",
        [&id],
        |row| {
            Ok(CheckupRecord {
                id: row.get(0)?,
                checkup_date: row.get(1)?,
                status: row.get(2)?,
                notes: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                file_count: Some(row.get(6)?),
                project_names: None,
            })
        },
    ).map_err(|e| format!("记录不存在: {}", e))?;

    // 查询关联项目名称
    let mut pstmt = conn
        .prepare(
            "SELECT DISTINCT p.name FROM checkup_files f
             JOIN checkup_projects p ON f.project_id = p.id
             WHERE f.record_id = ?1"
        )
        .map_err(|e| format!("查询失败: {}", e))?;
    let names: Vec<String> = pstmt
        .query_map([&id], |row| row.get(0))
        .map_err(|e| format!("查询失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();
    record.project_names = Some(names);

    Ok(record)
}

#[tauri::command]
pub fn get_or_create_today_record(db: State<Database>) -> Result<CheckupRecord, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    // 1. 尝试查找今天的记录
    let existing_id: Option<String> = {
        let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
        let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;
        conn.query_row(
            "SELECT id FROM checkup_records WHERE checkup_date = ?1 LIMIT 1",
            [&today],
            |row| row.get(0),
        ).ok()
    };

    if let Some(id) = existing_id {
        return get_record(id, db);
    }

    // 2. 不存在则创建
    let input = CreateRecordInput {
        checkup_date: today,
        notes: None,
    };
    create_record(input, db)
}
