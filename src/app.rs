// use std::ops::DerefMut;
// use egui::RadioButton;
use eframe::egui::{self, FontId, RichText, TextStyle, WidgetText};

use egui::menu::menu_button;
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
    column: String,
    find: String,
    replace: String,

    basic_search: bool,
    group_search: bool,
    group_column: String,
    group_null: bool,

    tags_search: bool,
    deep_dive_search: bool,
    compare_db: bool,
    compare_db_path: Option<String>,
    order: Vec<String>,
    tag: String,
    tags: Vec<String>,
    group: GroupBy,
    
    #[serde(skip)] // This how you opt-out of serialization of a field
    safe: bool,
    dupes_db: bool,
    my_panel: Panel,
    new_tag: String,
    sel_tags: Vec<bool>,
    new_line: String,
    sel_line: Option<usize>,
    order_text: String,

}

#[derive(PartialEq, serde::Serialize, Deserialize)]
enum Panel { Duplicates, Order, OrderText, Tags, Find }
#[derive(PartialEq, serde::Serialize, Deserialize)]
enum GroupBy { Show, Library, Other, None}


impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            db_path: None,
            total_records: 0,
            columns: Vec::new(),
            column: "Filepath".to_owned(),
            find: String::new(),
            replace: String::new(),
            basic_search: true,
            group_search: false,
            group_column: "Show".to_owned(),
            group_null: false,
            tags_search: true,
            deep_dive_search: false,
            compare_db: false,
            compare_db_path: None,
            order: DEFAULT_ORDER_VEC.map(|s| s.to_string()).to_vec(),
            tag: "Tag".to_owned(),
            tags: DEFAULT_TAGS_VEC.map(|s| s.to_string()).to_vec(),
            group: GroupBy::Show,
            safe: true,
            dupes_db: true,
            my_panel: Panel::Duplicates,
            new_tag: String::new(),
            sel_tags: Vec::new(),
            new_line: String::new(),
            sel_line: None,
            order_text: String::new(),
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
    fn reset_to_defaults(&mut self, db_path: Option<String>) {
        *self = Self::default();
        self.db_path = db_path;
        if let Some(path) = self.db_path.clone() {
            self.total_records = get_db_size(path.clone());
            self.columns = get_columns(path.clone());
        }
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
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open Database").clicked() {
                            ui.close_menu();
                            self.db_path = open_db();
                            if let Some(path) = self.db_path.clone() {
                                self.total_records = get_db_size(path.clone());
                                self.columns = get_columns(path.clone());
                            }
                        }
                        if ui.button("Close Database").clicked() {ui.close_menu(); self.db_path = None;}
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    // ui.add_space(16.0);
                    // ui.menu_button("Run", |ui| {
                    //     if ui.button("Search For Duplicates").clicked() {}
                    //     if ui.button("Search and Remove Duplicates").clicked() {}
                    //     // if ui.button("Open").clicked() {}
                        
                    // });
                    // ui.add_space(16.0);
                    ui.menu_button("Config", |ui| {
                        if ui.button("Restore Defaults").clicked() {ui.close_menu(); self.reset_to_defaults(self.db_path.clone())}
                        if ui.button("Edit Duplicate Search Logic").clicked() {ui.close_menu(); self.my_panel = Panel::Order}
                        // if ui.button("Tag Editor").clicked() {ui.close_menu(); self.my_panel = Panel::Tags}
                    });
                    // ui.menu_button("View", |ui| {
                    //     if ui.button("Duplicates Search").clicked() {ui.close_menu(); self.my_panel = Panel::Duplicates}
                    //     if ui.button("Find & Replace Text").clicked() {ui.close_menu(); self.my_panel = Panel::Find}
                        
                    // });
                    ui.add_space(48.0);
                    // ui.add_space(16.0);
                    egui::menu::bar(ui, |ui| {
                        
                        ui.selectable_value(&mut self.my_panel, Panel::Duplicates, "Duplicate Filename Search",);
                        ui.add_space(16.0);
                        // ui.selectable_value(&mut self.my_panel, Panel::Order, "Adjust Search Order Config",);
                        // ui.add_space(16.0);
                        // ui.selectable_value(&mut self.my_panel, Panel::Tags, "Manage Audiosuite Tags",);
                        ui.selectable_value(&mut self.my_panel, Panel::Find, "Find/Replace Text in database",);
                        ui.add_space(16.0);
    
                    });
                }

            });

            
            
        });
            
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.db_path.is_none() {
                    // ui.horizontal_centered(|ui| {
                        ui.vertical_centered(|ui| {
                            if ui.add_sized([200.0, 50.0], egui::Button::new(RichText::new("Open Database").size(24.0).strong())).clicked() {
                                self.db_path = open_db();
                                if let Some(path) = self.db_path.clone() {
                                    self.total_records = get_db_size(path.clone());
                                    self.columns = get_columns(path.clone());
                                }
                            } 
                        });
                    // });
                    // if ui.button("Open Databse").clicked() {
                    //     self.db_path = open_db();
                    //     if let Some(path) = self.db_path.clone() {
                    //         self.total_records = get_db_size(path.clone());
                    //         self.columns = get_columns(path.clone());
                    //     }
                    // }              
                    // });
                    return;
                }
                ui.horizontal(|_| {});
                ui.vertical_centered(|ui| {
                    if let Some(path) = &self.db_path {
                        ui.heading(RichText::new(path.split('/').last().unwrap()).size(24.0).strong());
                    }
                    ui.label(format!("{} records", self.total_records));
                    
                });
                ui.horizontal(|_| {});
                ui.separator();
                ui.horizontal(|_| {});
                
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("SMDB Companion");
         
            // ui.separator();

            match self.my_panel {
                Panel::Find => {
                    ui.heading("Find and Replace");
        
                    ui.horizontal(|ui| {
                        ui.label("Find Text: ");
                        ui.text_edit_singleline(&mut self.find);
                        ui.label("in Column: ");
                        // ui.text_edit_singleline(&mut self.column);
                        // ui.radio_value(&mut self.column, "FilePath".to_string(), "File Path");
                        // ui.radio_value(&mut self.column, format!("{}",&mut self.column), "Other: ");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{}", self.column))
                            .show_ui(ui, |ui| {
                                for col in &self.columns {
                                    ui.selectable_value(&mut self.column, col.to_string(), format!("{col}"));
                                }
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
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
                        // if self.group_search {
                        
                            // ui.radio_value(&mut self.group, GroupBy::Show, "Show");
                            // ui.radio_value(&mut self.group, GroupBy::Library, "Library");
                            // ui.radio_value(&mut self.group, GroupBy::Other, "Other: ");
                            egui::ComboBox::from_label(" ")
                                .selected_text(format!("{}", self.group_column))
                                .show_ui(ui, |ui| {
                                    for col in &self.columns {
                                        ui.selectable_value(&mut self.group_column, col.to_string(), format!("{col}"));
                                    }
                            });
                        // }
                        
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(44.0);
                        ui.label("Records without group entry: ");
                        ui.radio_value(&mut self.group_null, false, "Skip/Ignore");
                        ui.radio_value(&mut self.group_null, true, "Process Together");
                        // ui.checkbox(&mut self.group_null, "Process records without defined group together, or skip?");
                    });
                    ui.horizontal(|_| {});
                    ui.checkbox(&mut self.deep_dive_search, "Deep Dive Duplicates Search (Slow)");
                    ui.horizontal( |ui| {
                        ui.add_space(24.0);
                        ui.label("Filenames ending in .#, .#.#.#, or .M will be examined as possible duplicates");
                    });
                    ui.horizontal(|_| {});
                    ui.checkbox(&mut self.tags_search, "Search for Records with AudioSuite Tags");

                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        if ui.button("Add Tag:").clicked {
                            self.tags.sort_by_key(|s| s.to_lowercase());
                            if self.new_tag.len() > 0 {
                                self.tags.push(self.new_tag.clone());
                                self.new_tag = "".to_string();
                        }}
                        ui.text_edit_singleline(&mut self.new_tag);    
                    });
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            if ui.button("Remove Tag").clicked {
                                self.tags.retain(|s| s != &self.tag);
                                self.tag = "".to_string();
                            }
                            egui::ComboBox::from_label("")
                            .selected_text(format!("{}", self.tag))
                            .show_ui(ui, |ui| {
                                for tag in &self.tags {
                                    ui.selectable_value(&mut self.tag, tag.to_string(), format!("{tag}"));
                                }
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label("Filenames with Common Protools AudioSuite Tags will be marked for removal")
                        });
                        
                    ui.horizontal(|_| {});
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.compare_db, "Compare against database: ");
                        if let Some(path) = &self.compare_db_path {
                            ui.label(path.split('/').last().unwrap());
                        }

                        
                        // ui.text_edit_singleline(&mut self.compare_db_path);
                        if ui.button("Select DB").clicked() {
                            self.compare_db_path = open_db();
                                
                        }
                    });
                   
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.label("Filenames from Target Database found in Comparison Database will be Marked for Removal");
                    });

                    ui.horizontal(|_| {});
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
                        // if ui.button("Reset to Defaults").clicked() {
                        //     self.reset_to_defaults();
                        // }
                    });

                }
                Panel::Order => {
                    ui.heading("Column in order of Priority and whether it should be DESCending or ASCending.");
                    ui.label("These are SQL arguments and Google/ChatGPT can help you figure out how to compose them");
                    ui.horizontal(|_|{});
                    ui.heading("Examples:");
                    ui.heading("CASE WHEN pathname LIKE '%Audio Files%' THEN 1 ELSE 0 END ASC");
                    ui.label("Records with 'Audio Files' in the path will be removed over something that does not have it");
                    ui.horizontal(|_|{});
                    ui.heading("CASE WHEN pathname LIKE '%LIBRARY%' THEN 0 ELSE 1 END ASC");
                    ui.label("Records with 'LIBRARY' (not case sensitive) in the path will be kept over records without");
                    ui.horizontal(|_|{});
                    ui.heading("Rules at the top of the list are prioritized over those below");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Up").clicked() {
                            if let Some(index) = self.sel_line {
                                if index > 0 {
                                    self.sel_line = Some(index-1);
                                    self.order.swap(index, index-1);
                                }
                            }
                        }
                        if ui.button("Down").clicked() {
                            if let Some(index) = self.sel_line {
                                if index < self.order.len() - 1 {
                                    self.sel_line = Some(index+1);
                                    self.order.swap(index, index+1);
                                }
                            }
                        }
                        if ui.button("Remove").clicked() {
                            if let Some(index) = self.sel_line {
                                self.order.remove(index);
                                self.sel_line = None;
                            }
                        }
                        
                        
                        if ui.button("Add Line:").clicked {
                            
                            if self.new_line.len() > 0 {
                                self.order.insert(0, self.new_line.clone());
                                self.new_line = "".to_string();
                        }}
                        ui.text_edit_singleline(&mut self.new_line);    
                    });
                    ui.separator();
                    for (index, line) in self.order.iter_mut().enumerate() {
                        let checked = self.sel_line == Some(index);
                        if ui.selectable_label(checked, line.clone()).clicked {
                            self.sel_line = if checked { None } else { Some(index) };
                        }
                    }
                    ui.separator();
                    if ui.button("Text Editor").clicked() {
                        self.order_text = self.order.join("\n");
                        self.my_panel = Panel::OrderText;
                    }
                    
                }

                Panel:: OrderText => {
                    ui.heading("Column in order of Priority and whether it should be DESCending or ASCending.");
                    ui.label("These are SQL arguments and Google/ChatGPT can help you figure out how to compose them");
                    ui.horizontal(|_|{});
                    ui.heading("Examples:");
                    ui.heading("CASE WHEN pathname LIKE '%Audio Files%' THEN 1 ELSE 0 END ASC");
                    ui.label("Records with 'Audio Files' in the path will be removed over something that does not have it");
                    ui.horizontal(|_|{});
                    ui.heading("CASE WHEN pathname LIKE '%LIBRARY%' THEN 0 ELSE 1 END ASC");
                    ui.label("Records with 'LIBRARY' (not case sensitive) in the path will be kept over records without");
                    ui.horizontal(|_|{});
                    ui.heading("Rules at the top of the list are prioritized over those below");
                    ui.separator();

                    ui.columns(1, |columns| {
                        // columns[0].heading("Duplicate Filename Keeper Priority Order:");
                        columns[0].text_edit_multiline(&mut self.order_text);
                    });
                    ui.separator();
                    if ui.button("Save").clicked() {
                        self.order = self.order_text.lines().map(|s| s.to_string()).collect();
                        self.my_panel = Panel::Order;
                    }
                }

                Panel::Tags => {
                    
                  
                    // let mut remove = None;
                    for (index, tag) in self.tags.iter_mut().enumerate() {
                        if self.sel_tags.len() == index {self.sel_tags.push(false)}
                        if ui.selectable_label(self.sel_tags[index], tag.clone()).clicked {
                            self.sel_tags[index] = !self.sel_tags[index];
                            
                        }
                     
                    }
                    // if let Some(index) = remove {
                    //     self.tags.remove(index);
                    // }

                    
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
    let mut column_names: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(1)?) // The 1st index corresponds to the "name" column
    }).unwrap()
    .filter_map(Result::ok) // Filter out any errors
    .filter(|c| !c.starts_with("_"))
    .collect();


    column_names.sort();
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

const DEFAULT_ORDER_VEC: [&str; 22] = [
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