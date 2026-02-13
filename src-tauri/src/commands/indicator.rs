use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Indicator {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub unit: String,
    pub reference_range: String,
    pub sort_order: i32,
    pub is_core: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIndicatorInput {
    pub project_id: String,
    pub name: String,
    pub unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_core: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIndicatorInput {
    pub id: String,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_core: Option<bool>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub fn list_indicators(project_id: String, db: State<Database>) -> Result<Vec<Indicator>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, name, unit, reference_range, sort_order, is_core, created_at
             FROM indicators WHERE project_id = ?1 ORDER BY sort_order ASC, created_at ASC"
        )
        .map_err(|e| format!("查询指标失败: {}", e))?;

    let indicators = stmt
        .query_map([&project_id], |row| {
            Ok(Indicator {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                unit: row.get(3)?,
                reference_range: row.get(4)?,
                sort_order: row.get(5)?,
                is_core: row.get::<_, i32>(6)? == 1,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询指标失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析指标数据失败: {}", e))?;

    Ok(indicators)
}

#[tauri::command]
pub fn create_indicator(input: CreateIndicatorInput, db: State<Database>) -> Result<Indicator, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let unit = input.unit.unwrap_or_default();
    let reference_range = input.reference_range.unwrap_or_default();
    let is_core = input.is_core.unwrap_or(false);

    conn.execute(
        "INSERT INTO indicators (id, project_id, name, unit, reference_range, sort_order, is_core, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![id, input.project_id, input.name, unit, reference_range, is_core as i32, now],
    )
    .map_err(|e| format!("创建指标失败: {}", e))?;

    Ok(Indicator {
        id,
        project_id: input.project_id,
        name: input.name,
        unit,
        reference_range,
        sort_order: 0,
        is_core,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_indicator(input: UpdateIndicatorInput, db: State<Database>) -> Result<Indicator, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let existing = conn.query_row(
        "SELECT id, project_id, name, unit, reference_range, sort_order, is_core, created_at
         FROM indicators WHERE id = ?1",
        [&input.id],
        |row| {
            Ok(Indicator {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                unit: row.get(3)?,
                reference_range: row.get(4)?,
                sort_order: row.get(5)?,
                is_core: row.get::<_, i32>(6)? == 1,
                created_at: row.get(7)?,
            })
        },
    ).map_err(|e| format!("指标不存在: {}", e))?;

    let name = input.name.unwrap_or(existing.name);
    let unit = input.unit.unwrap_or(existing.unit);
    let reference_range = input.reference_range.unwrap_or(existing.reference_range);
    let is_core = input.is_core.unwrap_or(existing.is_core);
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    conn.execute(
        "UPDATE indicators SET name=?1, unit=?2, reference_range=?3, is_core=?4, sort_order=?5 WHERE id=?6",
        rusqlite::params![name, unit, reference_range, is_core as i32, sort_order, input.id],
    )
    .map_err(|e| format!("更新指标失败: {}", e))?;

    Ok(Indicator {
        id: input.id,
        project_id: existing.project_id,
        name,
        unit,
        reference_range,
        sort_order,
        is_core,
        created_at: existing.created_at,
    })
}

#[tauri::command]
pub fn delete_indicator(id: String, db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 检查是否有关联的指标值
    let value_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM indicator_values WHERE indicator_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询失败: {}", e))?;

    if value_count > 0 {
        return Err(format!("该指标有 {} 条历史数据，无法删除。", value_count));
    }

    conn.execute("DELETE FROM indicators WHERE id = ?1", [&id])
        .map_err(|e| format!("删除指标失败: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub fn ensure_indicator(input: CreateIndicatorInput, db: State<Database>) -> Result<Indicator, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if exists
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM indicators WHERE project_id = ?1 AND name = ?2)",
        [&input.project_id, &input.name],
        |row| row.get(0),
    ).unwrap_or(false);

    if exists {
        return Err("指标已存在".to_string());
    }

    // Create
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let unit = input.unit.unwrap_or_default();
    let reference_range = input.reference_range.unwrap_or_default();
    let is_core = input.is_core.unwrap_or(true); // Default to true for this quick add action

    conn.execute(
        "INSERT INTO indicators (id, project_id, name, unit, reference_range, sort_order, is_core, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![id, input.project_id, input.name, unit, reference_range, is_core as i32, now],
    )
    .map_err(|e| format!("创建指标失败: {}", e))?;

    Ok(Indicator {
        id,
        project_id: input.project_id,
        name: input.name,
        unit,
        reference_range,
        sort_order: 0,
        is_core,
        created_at: now,
    })
}
