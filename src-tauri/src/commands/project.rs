use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;
use super::AppDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub fn list_projects(db: State<Database>) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, description, sort_order, is_active, created_at, updated_at FROM checkup_projects ORDER BY sort_order ASC, created_at ASC")
        .map_err(|e| format!("查询项目失败: {}", e))?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                sort_order: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询项目失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析项目数据失败: {}", e))?;

    Ok(projects)
}

#[tauri::command]
pub fn create_project(
    input: CreateProjectInput,
    db: State<Database>,
    app_dir: State<AppDir>,
) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let description = input.description.unwrap_or_default();

    // 创建对应的 pictures 子目录
    let project_dir = app_dir.0.join("pictures").join(&input.name);
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("创建项目文件夹失败: {}", e))?;

    conn.execute(
        "INSERT INTO checkup_projects (id, name, description, sort_order, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, 1, ?4, ?5)",
        rusqlite::params![id, input.name, description, now, now],
    )
    .map_err(|e| format!("创建项目失败: {}", e))?;

    Ok(Project {
        id,
        name: input.name,
        description,
        sort_order: 0,
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_project(input: UpdateProjectInput, db: State<Database>) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().to_rfc3339();

    // 先查询现有数据
    let existing = conn.query_row(
        "SELECT id, name, description, sort_order, is_active, created_at FROM checkup_projects WHERE id = ?1",
        [&input.id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                sort_order: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: String::new(),
            })
        },
    ).map_err(|e| format!("项目不存在: {}", e))?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);
    let is_active = input.is_active.unwrap_or(existing.is_active);
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    conn.execute(
        "UPDATE checkup_projects SET name=?1, description=?2, is_active=?3, sort_order=?4, updated_at=?5 WHERE id=?6",
        rusqlite::params![name, description, is_active as i32, sort_order, now, input.id],
    )
    .map_err(|e| format!("更新项目失败: {}", e))?;

    Ok(Project {
        id: input.id,
        name,
        description,
        sort_order,
        is_active,
        created_at: existing.created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_project(id: String, db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 检查是否有关联文件
    let file_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM checkup_files WHERE project_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询关联文件失败: {}", e))?;

    if file_count > 0 {
        return Err(format!("该项目下有 {} 个关联文件，无法删除。请先删除相关检查记录。", file_count));
    }

    // 先删除关联的指标
    conn.execute("DELETE FROM indicators WHERE project_id = ?1", [&id])
        .map_err(|e| format!("删除指标失败: {}", e))?;

    conn.execute("DELETE FROM checkup_projects WHERE id = ?1", [&id])
        .map_err(|e| format!("删除项目失败: {}", e))?;

    Ok(true)
}
