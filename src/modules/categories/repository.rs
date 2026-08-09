use rusqlite::params;

use crate::core::db::modeles::CategoryRow;
use crate::core::db::pool::DatabasePool;
use crate::domaine::categorie::Category;

fn row_to_category(row: &CategoryRow) -> Result<Category, String> {
    let kind = crate::domaine::transaction::TransactionKind::from_str(&row.kind).ok_or_else(|| {
        format!("Catégorie {} : type inconnu « {} ».", row.id, row.kind)
    })?;
    let parse_horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut)
            .map(|d| d.with_timezone(&chrono::Utc))
            .map_err(|_| format!("Catégorie {} : horodatage invalide « {} ».", row.id, brut))
    };

    Ok(Category {
        id: row.id.clone(),
        kind,
        name: row.name.clone(),
        icon: row.icon.clone(),
        color: row.color.clone(),
        sort_order: row.sort_order,
        is_default: row.is_default,
        is_active: row.is_active,
        created_at: parse_horodatage(&row.created_at)?,
        updated_at: parse_horodatage(&row.updated_at)?,
    })
}

pub fn find_all_active(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at
             FROM categories
             WHERE is_active = 1
             ORDER BY kind, sort_order, name",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                is_default: row.get::<_, i32>(6)? != 0,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut categories = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        categories.push(row_to_category(&row)?);
    }

    Ok(categories)
}

pub fn find_all(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at
             FROM categories
             ORDER BY kind, sort_order, name",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                is_default: row.get::<_, i32>(6)? != 0,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut categories = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        categories.push(row_to_category(&row)?);
    }

    Ok(categories)
}

pub fn find_by_kind(pool: &DatabasePool, kind: &str) -> Result<Vec<Category>, String> {
    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at
             FROM categories
             WHERE kind = ?1
             ORDER BY sort_order, name",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map(params![kind], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                is_default: row.get::<_, i32>(6)? != 0,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut categories = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        categories.push(row_to_category(&row)?);
    }

    Ok(categories)
}

pub fn find_by_id(pool: &DatabasePool, id: &str) -> Result<Option<Category>, String> {
    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at
             FROM categories WHERE id = ?1",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                is_default: row.get::<_, i32>(6)? != 0,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    match rows.next() {
        Some(Ok(row)) => Ok(Some(row_to_category(&row)?)),
        Some(Err(e)) => Err(format!("Erreur de lecture : {}", e)),
        None => Ok(None),
    }
}

pub fn insert(pool: &DatabasePool, cat: &Category) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                cat.id,
                cat.kind.as_str(),
                cat.name,
                cat.icon,
                cat.color,
                cat.sort_order,
                cat.is_default as i32,
                cat.is_active as i32,
                &now,
                &now,
            ],
        )
        .map_err(|e| format!("Erreur d'insertion : {}", e))?;
    Ok(())
}

pub fn update(pool: &DatabasePool, cat: &Category) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let maj = pool
        .conn
        .execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3, sort_order = ?4, is_active = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                cat.name,
                cat.icon,
                cat.color,
                cat.sort_order,
                cat.is_active as i32,
                &now,
                cat.id,
            ],
        )
        .map_err(|e| format!("Erreur de mise à jour : {}", e))?;

    if maj == 0 {
        return Err(format!("Catégorie introuvable : {}", cat.id));
    }
    Ok(())
}
