use std::ops::DerefMut;
use rusqlite::{Connection, Result};

use serde::Deserialize;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    db_path: Option<String>,
    total_records: usize,
    columns: Vec<String>,
    find: String,
    replace: String,
    column: String,

    basic_search: bool,
    group_search: bool,
    group_column: String,
    group_null: bool,

    tags_search: bool,
    deep_dive_search: bool,
    compare_db: bool,
    compare_db_path: String,
    order: String,
    tags: String,
    
    #[serde(skip)] // This how you opt-out of serialization of a field
    safe: bool,
    dupes_db: bool,
    my_panel: Panel,

}

#[derive(PartialEq, serde::Serialize, Deserialize)]
enum Panel { Duplicates, Order, Tags, Find }



impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            db_path: None,
            total_records: 0,
            columns: Vec::new(),
            find: String::new(),
            replace: String::new(),
            column: "Filepath".to_owned(),
            basic_search: true,
            group_search: false,
            group_column: "Show".to_owned(),
            group_null: false,
            tags_search: true,
            deep_dive_search: false,
            compare_db: false,
            compare_db_path: String::new(),
            order: TJF_DEFAULT_ORDER.to_owned(),
            tags: DEFAULT_TAGS.to_owned(),
            safe: true,
            dupes_db: false,
            my_panel: Panel::Duplicates,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            ui.horizontal(|ui| {
                // ui.text_edit_singleline(&mut self.db_path);
                ui.heading("Target Database: ");
                if ui.button("Open Databse").clicked() {
                    self.db_path = open_db();
                    if let Some(path) = self.db_path.clone() {
                        self.total_records = get_db_size(path.clone());
                        self.columns = get_columns(path);
                    }
                }
            });
            if self.db_path.is_none() {return;}
            if let Some(path) = &self.db_path {
                ui.label(path);
            }
            if self.total_records > 0 {
                ui.label(format!("{} total records found in database", self.total_records));
                // ui.horizontal(|ui| {
                //     ui.label(self.total_records.to_string());
                //     ui.label(" total records found in database");
                    
                // });
            }
        });
            
            egui::CentralPanel::default().show(ctx, |ui| {
            if self.db_path.is_none() {return;}
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("SMDB Companion");
            egui::menu::bar(ui, |ui| {
                
                ui.selectable_value(&mut self.my_panel, Panel::Duplicates, "Duplicate Filename Search",);
                ui.selectable_value(&mut self.my_panel, Panel::Order, "Adjust Search Order Config",);
                ui.selectable_value(&mut self.my_panel, Panel::Tags, "Manage Audiosuite Tags",);
                ui.selectable_value(&mut self.my_panel, Panel::Find, "Find/Replace Text in database",);
    
            });
            ui.separator();

            match self.my_panel {
                Panel::Find => {
                    ui.heading("Find and Replace");
        
                    ui.horizontal(|ui| {
                        ui.label("Find Text");
                        ui.text_edit_singleline(&mut self.find);
                        ui.label("in Column: ");
                        // ui.text_edit_singleline(&mut self.column);
                        egui::ComboBox::from_label("in Column: ")
                            .selected_text(format!("{}", self.column))
                            .show_ui(ui, |ui| {
                                for col in &self.columns {
                                    ui.selectable_value(&mut self.column, col.to_string(), format!("{col}"));
                                }
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace: ");
                        ui.text_edit_singleline(&mut self.replace);
                    });
        
                    if ui.button("Process").clicked() {
                        smreplace();
                    }
                }
                Panel::Duplicates => {
                    ui.heading("Search for Duplicate Records");
        
                    ui.checkbox(&mut self.basic_search, "Basic Duplicate Filename Search");
                    // ui.style_mut().spacing.indent = 5.0;
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.checkbox(&mut self.group_search, "Group Duplicate Filename Search by: ");
                        ui.text_edit_singleline(&mut self.group_column);
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.checkbox(&mut self.group_null, "Process records without defined group together, or skip?");
                    });
                    ui.horizontal(|ui| {});
                    ui.checkbox(&mut self.tags_search, "Plug-In Tags Search");
                    ui.checkbox(&mut self.deep_dive_search, "Deep Dive Duplicates Search");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.compare_db, "Compare against database: ");
                        ui.text_edit_singleline(&mut self.compare_db_path);
                        if ui.button("Open file…").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.compare_db_path = path.display().to_string();
                                // self.db_path = Some(path.display().to_string());
                            }
                        }
                    });
                    ui.horizontal(|ui| {});
                    ui.checkbox(&mut self.safe, "Create Safety Database of Thinned Records");
                    ui.checkbox(&mut self.dupes_db, "Create Database of Duplicate Records");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Search for Duplicates").clicked() {
                            gather_dupicates();
                        }
                        if ui.button("Remove Duplicates").clicked() {
                            remove_dupicates();
                        }
                    });

                }
                Panel::Order => {
                    ui.columns(1, |columns| {
                        columns[0].heading("Duplicate Filename Keeper Priority Order:");
                        columns[0].text_edit_multiline(&mut self.order);
                        if columns[0].button("Default Order").clicked() {
                            self.order = TJF_DEFAULT_ORDER.to_owned();
                        }
                   
        
                    });
                    
                }

                Panel::Tags => {
                    ui.columns(1, |columns| {
                     
                        columns[0].heading("Protools Plugin Tag Search:");
                        columns[0].text_edit_multiline(&mut self.tags);
                        if columns[0].button("Default Tags").clicked() {
                            self.tags = DEFAULT_TAGS.to_owned();
                        }
        
                    });
                    
                }



            }

        });
    }
}

fn gather_dupicates() {}
fn remove_dupicates() {}
fn smreplace() {}

fn open_db() -> Option<String> {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let db_path = path.display().to_string();
        if db_path.ends_with(".sqlite") {return Some(db_path);}
    }    
    None
}

fn get_db_size(db_path: String) -> usize {
    let conn = Connection::open(db_path).unwrap();
     let count: usize = conn.query_row(
         "SELECT COUNT(*) FROM justinmetadata",
         [],
         |row| row.get(0) 
     ).unwrap();
     count
}

fn get_columns(db_path: String) -> Vec<String> {
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(justinmetadata);").unwrap();

    // Execute the query and collect the column names into a Vec<String>
    let column_names: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(1)?) // The 1st index corresponds to the "name" column
    }).unwrap()
    .filter_map(Result::ok) // Filter out any errors
    .collect();

    column_names
}




const TJF_DEFAULT_ORDER: &str = r#"CASE WHEN pathname LIKE '%TJF RECORDINGS%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%LIBRARIES%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%SHOWS/Tim Farrell%' THEN 1 ELSE 0 END ASC
CASE WHEN Description IS NOT NULL AND Description != '' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%Audio Files%' THEN 1 ELSE 0 END ASC
CASE WHEN pathname LIKE '%RECORD%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%CREATED SFX%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%CREATED FX%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%LIBRARY%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%/LIBRARY%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%LIBRARY/%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%SIGNATURE%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%PULLS%' THEN 0 ELSE 1 END ASC
CASE WHEN pathname LIKE '%EDIT%' THEN 1 ELSE 0 END ASC
CASE WHEN pathname LIKE '%MIX%' THEN 1 ELSE 0 END ASC
CASE WHEN pathname LIKE '%SESSION%' THEN 1 ELSE 0 END ASC
duration DESC
channels DESC
sampleRate DESC
bitDepth DESC
BWDate ASC
scannedDate ASC
"#;

const DEFAULT_TAGS: &str = r#"-1eqa_ 
-6030_  
-7eqa_ 
-A2sA_  
-A44m_  
-A44s_  
-Alt7S_  
-ASMA_  
-AVrP_  
-AVrT_  
-AVSt_ 
-DEC4_  
-Delays_  
-Dn_ 
-DUPL_ 
-DVerb_  
-GAIN_  
-M2DN_  
-NORM_ 
-NYCT_  
-PiSh    
-PnT2_  
-PnTPro_  
-ProQ2_  
-PSh_  
-Reverse_  
-RVRS_  
-RING_  
-RX7Cnct_  
-spce_  
-TCEX_
-TiSh_
-TmShft_ 
-VariFi_
-VlhllVV_ 
-VSPD_
-VitmnMn_ 
-VtmnStr_ 
-X2mA_ 
-X2sA_ 
-XForm_
-Z2N5_
-Z2S5_
-Z4n2_
-ZXN5_  
"#;