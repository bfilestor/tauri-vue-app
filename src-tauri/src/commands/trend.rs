use crate::db::Database;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TrendDataPoint {
    pub checkup_date: String,
    pub value: Option<f64>,
    pub value_text: String,
    pub is_abnormal: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct IndicatorTrend {
    pub indicator_id: String,
    pub indicator_name: String,
    pub unit: String,
    pub reference_range: String,
    pub data_points: Vec<TrendDataPoint>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProjectTrend {
    pub project_id: String,
    pub project_name: String,
    pub indicators: Vec<IndicatorTrend>,
}

/// 获取某个项目的趋势数据
#[tauri::command]
pub fn get_project_trends(
    project_id: String,
    db: tauri::State<Database>,
) -> Result<ProjectTrend, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 获取项目名称
    let project_name: String = conn
        .query_row(
            "SELECT name FROM checkup_projects WHERE id = ?1",
            [&project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("项目不存在: {}", e))?;

    // 获取该项目的所有指标
    let mut ind_stmt = conn
        .prepare(
            "SELECT id, name, unit, reference_range FROM indicators
             WHERE project_id = ?1
             ORDER BY is_core DESC, sort_order ASC, name ASC",
        )
        .map_err(|e| format!("查询指标失败: {}", e))?;

    let indicators: Vec<(String, String, String, String)> = ind_stmt
        .query_map([&project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2).unwrap_or_default(),
                row.get::<_, String>(3).unwrap_or_default(),
            ))
        })
        .map_err(|e| format!("查询指标失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析指标数据失败: {}", e))?;

    // 获取每个指标的历史值
    let mut trend_indicators = Vec::new();

    for (ind_id, ind_name, unit, ref_range) in &indicators {
        let mut val_stmt = conn
            .prepare(
                "SELECT checkup_date, value, value_text, is_abnormal
                 FROM indicator_values
                 WHERE indicator_id = ?1 AND project_id = ?2
                 ORDER BY checkup_date ASC",
            )
            .map_err(|e| format!("查询指标值失败: {}", e))?;

        let data_points: Vec<TrendDataPoint> = val_stmt
            .query_map(rusqlite::params![ind_id, project_id], |row| {
                Ok(TrendDataPoint {
                    checkup_date: row.get(0)?,
                    value: row.get(1)?,
                    value_text: row.get::<_, String>(2).unwrap_or_default(),
                    is_abnormal: row.get::<_, i32>(3).unwrap_or(0) != 0,
                })
            })
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("解析数据失败: {}", e))?;

        trend_indicators.push(IndicatorTrend {
            indicator_id: ind_id.clone(),
            indicator_name: ind_name.clone(),
            unit: unit.clone(),
            reference_range: ref_range.clone(),
            data_points,
        });
    }

    Ok(ProjectTrend {
        project_id,
        project_name,
        indicators: trend_indicators,
    })
}

/// 获取所有项目的概要趋势数据
#[tauri::command]
pub fn get_all_trends(db: tauri::State<Database>) -> Result<Vec<ProjectTrend>, String> {
    let conn_guard = db.conn.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard.as_ref().ok_or("数据库连接已关闭".to_string())?;

    // 获取所有活跃项目
    let mut proj_stmt = conn
        .prepare(
            "SELECT id, name FROM checkup_projects WHERE is_active = 1 ORDER BY sort_order ASC",
        )
        .map_err(|e| format!("查询项目失败: {}", e))?;

    let projects: Vec<(String, String)> = proj_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("查询项目失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析项目数据失败: {}", e))?;

    drop(proj_stmt);
    drop(conn_guard);

    let mut result = Vec::new();
    for (pid, _pname) in &projects {
        match get_project_trends(pid.clone(), db.clone()) {
            Ok(pt) => result.push(pt),
            Err(e) => log::error!("获取项目趋势失败: {}", e),
        }
    }

    Ok(result)
}
