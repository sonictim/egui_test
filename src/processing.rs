#![allow(non_snake_case)]
use rusqlite::{Connection, Result};
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

pub fn smreplace_get(db_path: String, find: &mut String, column: &mut String ) -> usize {
    let conn: Connection = Connection::open(db_path).unwrap(); 

    let search_query = format!("SELECT COUNT(rowid) FROM {} WHERE {} LIKE ?1", TABLE, column);
    // Prepare the SQL query with the search text
    let stmt = conn.prepare(search_query.as_str()).ok();
    let count = stmt.expect("Failed to prepare statement").query_row([format!("%{}%", find)], |row| row.get(0)).unwrap();
    count
}

// pub async fn smreplace_get(db_path: &str, find: &str, column: &str) -> Result<usize, sqlx::Error> {
//     // Create a connection pool
//     let pool = SqlitePool::connect(db_path).await?;

//     // Create the search query using column
//     let search_query = format!(
//         "SELECT COUNT(rowid) FROM {} WHERE {} LIKE ?",
//         TABLE, column
//     );

//     // Execute the query and get the result
//     let row = sqlx::query(&search_query)
//         .bind(format!("%{}%", find)) // Bind the search text
//         .fetch_one(&pool)
//         .await?;

//     // Extract the count from the first column
//     let count: i64 = row.get(0);

//     // Convert to usize
//     Ok(usize::from_str(&count.to_string()).unwrap())
// }

pub fn smreplace_process(db_path: String, find: &mut String, replace: &mut String, column: &mut String, dirty: bool ) {
    let conn: Connection = Connection::open(db_path).unwrap(); 
    let dirty_text = if dirty { ", _Dirty = 1" } else { "" };
   
    let replace_query = format!("UPDATE {} SET {} = REPLACE({}, '{}', '{}'){} WHERE {} LIKE '%{}%'", TABLE, column, column, find, replace, dirty_text, column, find);
    conn.execute(replace_query.as_str(), []).ok();

}


pub fn gather_duplicates(main: &mut Config, group: &mut Config, deep: &mut Config, tags: &mut Config, compare: &mut Config) {
    let mut source_db_path = String::new();
    if let Some(path) = &main.option {
        source_db_path = path.clone();
    }
    let source_db_name = source_db_path.split('/').last().unwrap();

    main.status = format!("Opening {}", source_db_name);
    let mut conn = Connection::open(&source_db_path).unwrap(); 

    // if main.search {
    //     main.working = true;
    //     group.records = gather_duplicate_filenames_in_database(&mut conn, &config.group_sort, config.group_null, config.verbose)?;
    //     main.records.extend(group.records);
    //     main.working = false;
    // }

    // if deep.search {
    //     deep.working = true;
    //     deep.records = gather_records_with_trailing_numbers(&mut conn, total_records)?;
    //     main.records.extend(deep.records);
    //     deep.working = false;
    // }

    if tags.search {
        main.status = format!("Searching for tags");
        tags.working = true;
        tags.status = format!{"Found {} records with matching tags", tags.records.len()};
        gather_filenames_with_tags(&mut conn, tags).ok();
        tags.working = false;
        tags.status = format!{"Found {} records with matching tags", tags.records.len()};
        main.records.extend(tags.records.clone());
    }
    

    // if let Some(compare_db_path) = config.compare_db {
    //     let compare_conn = Connection::open(&compare_db_path)?; 
    //     let ids_from_compare_db = gather_compare_database_overlaps(&conn, &compare_conn)?;
    //     main.records.extend(ids_from_compare_db);
    // }



    if main.records.is_empty() {
        main.status = format!("No records marked for removal.");
       
    }

    main.status = format!("Marked {} total records for removal.", main.records.len());

}

pub fn gather_filenames_with_tags(conn: &mut Connection, tags: &mut Config) -> Result<()> {
    // tags.status = format!("Searching for filenames containing tags");
    // let mut file_records = HashSet::new();

    for tag in &tags.list {
        let query = format!("SELECT rowid, filename, duration FROM justinmetadata WHERE filename LIKE '%' || ? || '%'");
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([tag.clone()], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
                duration: row.get(2)?,
            })
        })?;

        for file_record in rows {
            let file_record = file_record?;
            tags.records.insert(file_record);
        }
    }
    // tags.records = file_records;
    Ok(())
    // tags.status = format!("{} total records containing tags marked for deletion", tags.records.len());
}

pub fn remove_duplicates() {}

pub fn open_db() -> Option<String> {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let db_path = path.display().to_string();
        if db_path.ends_with(".sqlite") {return Some(db_path);}
    }    
    None
}

pub fn get_db_size(db_path: String) -> usize {
    let conn = Connection::open(db_path).unwrap();
     let count: usize = conn.query_row(
         "SELECT COUNT(*) FROM justinmetadata",
         [],
         |row| row.get(0) 
     ).unwrap();
     count
}

pub fn get_columns(db_path: String) -> Vec<String> {
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(justinmetadata);").unwrap();

    // Execute the query and collect the column names into a Vec<String>
    let mut column_names: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(1)?) // The 1st index corresponds to the "name" column
    }).unwrap()
    .filter_map(Result::ok) // Filter out any errors
    .filter(|c| !c.starts_with("_"))
    .collect();


    column_names.sort();
    column_names
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