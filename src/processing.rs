#![allow(non_snake_case)]
use sqlx::{sqlite::SqlitePool, Row};
use tokio;
use std::collections::HashSet;
// use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::error::Error;
// use terminal_size::{Width, terminal_size};
// use sqlx::{sqlite::SqlitePool, Row};

use eframe::egui::{self, Ui, RichText};
use crate::app::*;

const TABLE: &str = "justinmetadata";

pub async fn smreplace_get(pool: &SqlitePool, find: &mut String, column: &mut String ) -> Result<usize, sqlx::Error>  {
    
    let search_query = format!("SELECT COUNT(rowid) FROM {} WHERE {} LIKE ?", TABLE, column);
    let result: (i64,) = sqlx::query_as(&search_query)
        .bind(format!("%{}%", find))
        .fetch_one(pool)
        .await?;

    Ok(result.0 as usize)
}

pub async fn smreplace_process(pool: &SqlitePool, find: &mut String, replace: &mut String, column: &mut String, dirty: bool ) {
   
    let dirty_text = if dirty { ", _Dirty = 1" } else { "" };

    let replace_query = format!(
        "UPDATE {} SET {} = REPLACE({}, '{}', '{}'){} WHERE {} LIKE '%{}%'", 
        TABLE, column, column, find, replace, dirty_text, column, find
    );
    let _ = sqlx::query(&replace_query).execute(pool).await;

}

pub async fn gather_duplicate_filenames_in_database(
    pool: &SqlitePool, 
    order: Vec<String>, 
    group_sort: &Option<String>, 
    group_null: bool, 
    verbose: bool
) -> Result<HashSet<FileRecord>, sqlx::Error> {
    
    let mut file_records = HashSet::new();

    // Construct the ORDER BY clause dynamically
    let order_clause = order.join(", ");

    // Build the SQL query based on whether a group_sort is provided
    let (partition_by, where_clause) = match group_sort {
        Some(group) => {
            if verbose {
                println!("Grouping duplicate record search by {}", group);
            }
            let where_clause = if group_null {
                if verbose {
                    println!("Records without a {} entry will be processed together.", group);
                }
                String::new()
            } else {
                if verbose {
                    println!("Records without a {} entry will be skipped.", group);
                }
                format!("WHERE {} IS NOT NULL AND {} != ''", group, group)
            };
            (format!("{}, filename", group), where_clause)
        }
        None => ("filename".to_string(), String::new()),
    };
    
    let sql = format!(
        "
        WITH ranked AS (
            SELECT
                rowid AS id,
                filename,
                duration,
                ROW_NUMBER() OVER (
                    PARTITION BY {}
                    ORDER BY {}
                ) as rn
            FROM justinmetadata
            {}
        )
        SELECT id, filename, duration FROM ranked WHERE rn > 1
        ",
        partition_by, order_clause, where_clause
    );

    // Execute the query and fetch the results
    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await?;
    
    // Iterate through the rows and insert them into the hashset
    for row in rows {
        let id: u32 = row.get(0);
        let file_record = FileRecord {
            id: id as usize,
            filename: row.get(1),
            duration: row.try_get(2).unwrap_or("".to_string()),  // Handle possible NULL in duration
        };
        file_records.insert(file_record);
    }

    if verbose {
        println!("Marked {} duplicate records for deletion.", file_records.len());
    }

    Ok(file_records)
}

    pub async fn gather_filenames_with_tags(pool: &SqlitePool, tags: &Vec<String>) -> Result<HashSet<FileRecord>, sqlx::Error>  {
        // tags.status = format!("Searching for filenames containing tags");
        println!("Tokio Start");
        let mut file_records = HashSet::new();

        for tag in tags {
            let query = "SELECT rowid, filename, duration FROM justinmetadata WHERE filename LIKE '%' || ? || '%'";
    
            // Execute the query and fetch rows
            let rows = sqlx::query(query)
                .bind(tag.clone())
                .fetch_all(pool)
                .await?;
    
            // Collect file records from the query result
            for row in rows {
                let id: u32 = row.get(0);
                let file_record = FileRecord {
                    id: id as usize,
                    filename: row.get(1),
                    duration: row.try_get(2).unwrap_or("".to_string()),  // Handle possible NULL in duration
                };
                file_records.insert(file_record);
            }
        }
        println!("Found Tags");
        Ok(file_records)
        // tags.status = format!("{} total records containing tags marked for deletion", tags.records.len());
    }


pub fn remove_duplicates() {}

pub async fn open_db() -> Option<Database> {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let db_path = path.display().to_string();
        if db_path.ends_with(".sqlite") {
            println!("Opening Database {}", db_path);
            let db = Database::open(db_path).await;
            return Some(db);
        }
    }    
    None
}

pub async fn get_db_size(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM justinmetadata")
        .fetch_one(pool)
        .await?;

    Ok(count.0 as usize)
}

pub async fn get_columns(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    // Query for table info using PRAGMA
    let columns = sqlx::query("PRAGMA table_info(justinmetadata);")
        .fetch_all(pool)
        .await?
        .into_iter()
        .filter_map(|row| {
            let column_name: String = row.try_get("name").ok()?; // Extract "name" column
            if !column_name.starts_with('_') {
                Some(column_name)
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    // Sort the column names
    let mut sorted_columns = columns;
    sorted_columns.sort();
    Ok(sorted_columns)
}

pub fn default_tags() -> Vec<String> {
const DEFAULT_TAGS_VEC: [&str; 44] = [
    "-6030_", 
    "-7eqa_",
    "-A2sA_", 
    "-A44m_", 
    "-A44s_", 
    "-Alt7S_", 
    "-ASMA_", 
    "-AVrP_", 
    "-AVrT_", 
    "-AVSt_", 
    "-DEC4_", 
    "-Delays_", 
    "-Dn_",
    "-DUPL_",
    "-DVerb_", 
    "-GAIN_", 
    "-M2DN_", 
    "-NORM_",
    "-NYCT_", 
    "-PiSh_", 
    "-PnT2_", 
    "-PnTPro_", 
    "-ProQ2_", 
    "-PSh_", 
    "-Reverse_", 
    "-RVRS_", 
    "-RING_", 
    "-RX7Cnct_", 
    "-spce_", 
    "-TCEX_", 
    "-TiSh_", 
    "-TmShft_", 
    "-VariFi_", 
    "-VlhllVV_", 
    "-VSPD_",
    "-VitmnMn_", 
    "-VtmnStr_", 
    "-X2mA_", 
    "-X2sA_", 
    "-XForm_",
    "-Z2N5_",
    "-Z2S5_",
    "-Z4n2_",
    "-ZXN5_", 
];
DEFAULT_TAGS_VEC.map(|s| s.to_string()).to_vec()
}

pub fn tjf_tags() -> Vec<String> {
const TJF_TAGS_VEC: [&str; 48] = [
    "-6030_", 
    "-7eqa_",
    "-A2sA_", 
    "-A44m_", 
    "-A44s_", 
    "-Alt7S_", 
    "-ASMA_", 
    "-AVrP_", 
    "-AVrT_", 
    "-AVSt_", 
    "-DEC4_", 
    "-Delays_", 
    "-Dn_",
    "-DUPL_",
    "-DVerb_", 
    "-GAIN_", 
    "-M2DN_", 
    "-NORM_",
    "-NYCT_", 
    "-PiSh_", 
    "-PnT2_", 
    "-PnTPro_", 
    "-ProQ2_", 
    "-PSh_", 
    "-Reverse_", 
    "-RVRS_", 
    "-RING_", 
    "-RX7Cnct_", 
    "-spce_", 
    "-TCEX_", 
    "-TiSh_", 
    "-TmShft_", 
    "-VariFi_", 
    "-VlhllVV_", 
    "-VSPD_",
    "-VitmnMn_", 
    "-VtmnStr_", 
    "-X2mA_", 
    "-X2sA_", 
    "-XForm_",
    "-Z2N5_",
    "-Z2S5_",
    "-Z4n2_",
    "-ZXN5_",
    ".new.",
    ".aif.",
    ".mp3.",
    ".wav.", 
];
TJF_TAGS_VEC.map(|s| s.to_string()).to_vec()
}

pub fn default_order() -> Vec<String> {
const DEFAULT_ORDER_VEC: [&str; 12] = [

    "CASE WHEN Description IS NOT NULL AND Description != '' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%Audio Files%' THEN 1 ELSE 0 END ASC",
    "CASE WHEN pathname LIKE '%LIBRARIES%' THEN 0 ELSE 1 END ASC",  
    "CASE WHEN pathname LIKE '%LIBRARY%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%/LIBRARY%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%LIBRARY/%' THEN 0 ELSE 1 END ASC",
    "duration DESC",
    "channels DESC",
    "sampleRate DESC",
    "bitDepth DESC",
    "BWDate ASC",
    "scannedDate ASC",
];
DEFAULT_ORDER_VEC.map(|s| s.to_string()).to_vec()
}

pub fn tjf_order() -> Vec<String> {
const TJF_ORDER_VEC: [&str; 22] = [
    "CASE WHEN pathname LIKE '%TJF RECORDINGS%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%LIBRARIES%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%SHOWS/Tim Farrell%' THEN 1 ELSE 0 END ASC",
    "CASE WHEN Description IS NOT NULL AND Description != '' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%Audio Files%' THEN 1 ELSE 0 END ASC",
    "CASE WHEN pathname LIKE '%RECORD%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%CREATED SFX%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%CREATED FX%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%LIBRARY%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%/LIBRARY%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%LIBRARY/%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%SIGNATURE%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%PULLS%' THEN 0 ELSE 1 END ASC",
    "CASE WHEN pathname LIKE '%EDIT%' THEN 1 ELSE 0 END ASC",
    "CASE WHEN pathname LIKE '%MIX%' THEN 1 ELSE 0 END ASC",
    "CASE WHEN pathname LIKE '%SESSION%' THEN 1 ELSE 0 END ASC",
    "duration DESC",
    "channels DESC",
    "sampleRate DESC",
    "bitDepth DESC",
    "BWDate ASC",
    "scannedDate ASC",
];
    TJF_ORDER_VEC.map(|s| s.to_string()).to_vec()
}