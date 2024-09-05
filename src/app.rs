// use std::ops::DerefMut;
// use egui::RadioButton;
use eframe::egui::{self, FontId, RichText, TextStyle, WidgetText};

use egui::{menu::menu_button, ModifierNames};
use rusqlite::{Connection, Result};
use std::collections::HashSet;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::error::Error;
// use terminal_size::{Width, terminal_size};
// use regex::Regex;
// use ordered_float::OrderedFloat;




use serde::Deserialize;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] 
struct Config {
    enabled: bool,
    // enabled2: bool,
    option: Option<String>,
    #[serde(skip)]
    status: String,
    #[serde(skip)]
    records: HashSet<FileRecord>,
}

impl Config {
    fn new(on: bool) -> Self {
        Self {
            enabled: on,
            // enabled2: false,
            option: None,
            status: String::new(),
            records: HashSet::new(),

        }
    }
    fn new_option(on: bool, o: &str) -> Self {
        Self {
            enabled: on,
            // enabled2: false,
            option: Some(o.to_string()),
            status: String::new(),
            records: HashSet::new(),

        }
    }
    
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct FileRecord {
    id: usize,
    filename: String,
    duration: String,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    total_records: usize,
    columns: Vec<String>,
    column: String,
    find: String,
    replace: String,

    basic_search: Config,
    group_search: Config,
    group_null: bool,

    tags_search: Config,
    deep_dive_search: Config,
    compare_db: Config,

    order: Vec<String>,
    tags: Vec<String>,

    
    #[serde(skip)] // This how you opt-out of serialization of a field
    safe: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    dupes_db: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    my_panel: Panel,
    #[serde(skip)] // This how you opt-out of serialization of a field
    new_tag: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    sel_tags: Vec<usize>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    new_line: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    sel_line: Option<usize>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    order_text: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    help: bool,

}    



#[derive(PartialEq, serde::Serialize, Deserialize)]
enum Panel { Duplicates, Order, OrderText, Tags, Find }


impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:

            total_records: 0,
            columns: Vec::new(),
            column: "Filepath".to_owned(),
            find: String::new(),
            replace: String::new(),
            basic_search: Config::new(true),
            group_search: Config::new_option(false, "Show"),
            group_null: false,
     
            tags_search: Config::new_option(false, "-"),
            deep_dive_search: Config::new(false),
            compare_db: Config::new(false),

            order: DEFAULT_ORDER_VEC.map(|s| s.to_string()).to_vec(),
            tags: DEFAULT_TAGS_VEC.map(|s| s.to_string()).to_vec(),


            safe: true,
            dupes_db: false,
            my_panel: Panel::Duplicates,
            new_tag: String::new(),
            sel_tags: Vec::new(),
            new_line: String::new(),
            sel_line: None,
            order_text: String::new(),
            help: false,
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
        self.basic_search.option = db_path;
        if let Some(path) = self.basic_search.option.clone() {
            self.total_records = get_db_size(path.clone());
            self.columns = get_columns(path.clone());
        }
    }
    fn reset_to_TJFdefaults(&mut self, db_path: Option<String>) {
        *self = Self::default();
        self.basic_search.option = db_path;
        self.order = TJF_ORDER_VEC.map(|s| s.to_string()).to_vec();
        self.tags = TJF_TAGS_VEC.map(|s| s.to_string()).to_vec();
        if let Some(path) = self.basic_search.option.clone() {
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
                            self.basic_search.option = open_db();
                            if let Some(path) = self.basic_search.option.clone() {
                                self.total_records = get_db_size(path.clone());
                                self.columns = get_columns(path.clone());
                            }
                        }
                        if ui.button("Close Database").clicked() {ui.close_menu(); self.basic_search.option = None;}
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
                        if ui.button("Restore Defaults").clicked() {ui.close_menu(); self.reset_to_defaults(self.basic_search.option.clone())}
                        if  ui.input(|i| i.modifiers.alt ) {
                            if ui.button("TJF Defaults").clicked() {ui.close_menu(); self.reset_to_TJFdefaults(self.basic_search.option.clone())}
                        }
                        if ui.button("Duplicate Search Logic").clicked() {ui.close_menu(); self.my_panel = Panel::Order}
                            if ui.button("Tag Editor").clicked() {ui.close_menu(); self.my_panel = Panel::Tags}

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
        
    // The central panel the region left after adding TopPanel's and SidePanel's
            
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.basic_search.option.is_none() {
                    // ui.horizontal_centered(|ui| {
                        ui.vertical_centered(|ui| {
                            if ui.add_sized([200.0, 50.0], egui::Button::new(RichText::new("Open Database").size(24.0).strong())).clicked() {
                                self.basic_search.option = open_db();
                                if let Some(path) = self.basic_search.option.clone() {
                                    self.total_records = get_db_size(path.clone());
                                    self.columns = get_columns(path.clone());
                                }
                            } 
                        });
       
                    return;
                }
                ui.horizontal(|_| {});
                ui.vertical_centered(|ui| {
                    if let Some(path) = &self.basic_search.option {
                        ui.heading(RichText::new(path.split('/').last().unwrap()).size(24.0).strong());
                    }
                    ui.label(format!("{} records", self.total_records));
                    
                });
                ui.horizontal(|_| {});
                ui.separator();
                ui.horizontal(|_| {});


            match self.my_panel {
                Panel::Find => {
                    ui.heading("Find and Replace");
        
                    ui.horizontal(|ui| {
                        ui.label("Find Text: ");
                        ui.text_edit_singleline(&mut self.find);
                        
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace: ");
                        ui.add_space(8.0);
                        ui.text_edit_singleline(&mut self.replace);
                    });
                    ui.horizontal(|ui| {
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
        
                    if ui.button("Process").clicked() {
                        smreplace();
                    }
                }
                Panel::Duplicates => {
                    ui.heading("Search for Duplicate Records");
        
                    ui.checkbox(&mut self.basic_search.enabled, "Basic Duplicate Filename Search");
                    // ui.style_mut().spacing.indent = 5.0;
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.checkbox(&mut self.group_search.enabled, "Group Duplicate Filename Search by: ");
                        if let Some(group) = &mut self.group_search.option {
                        
                            // ui.radio_value(&mut self.group, GroupBy::Show, "Show");
                            // ui.radio_value(&mut self.group, GroupBy::Library, "Library");
                            // ui.radio_value(&mut self.group, GroupBy::Other, "Other: ");
                            egui::ComboBox::from_label(" ")
                                .selected_text(format!("{}", group))
                                .show_ui(ui, |ui| {
                                    for col in &self.columns {
                                        ui.selectable_value(group, col.to_string(), format!("{col}"));
                                    }
                            });
                        }
                        
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(44.0);
                        ui.label("Records without group entry: ");
                        ui.radio_value(&mut self.group_null, false, "Skip/Ignore");
                        ui.radio_value(&mut self.group_null, true, "Process Together");
                        // ui.checkbox(&mut self.group_null, "Process records without defined group together, or skip?");
                    });
                    ui.horizontal(|_| {});
                    ui.checkbox(&mut self.deep_dive_search.enabled, "Deep Dive Duplicates Search (Slow)");
                    ui.horizontal( |ui| {
                        ui.add_space(24.0);
                        ui.label("Filenames ending in .#, .#.#.#, or .M will be examined as possible duplicates");
                    });
                    ui.horizontal(|_| {});
                    ui.checkbox(&mut self.tags_search.enabled, "Search for Records with AudioSuite Tags");

                    // ui.horizontal(|ui| {
                    //     ui.add_space(24.0);
                    //     if ui.button("Add Tag:").clicked {
                    //         self.tags.sort_by_key(|s| s.to_lowercase());
                    //         if self.new_tag.len() > 0 {
                    //             self.tags.push(self.new_tag.clone());
                    //             self.new_tag = "".to_string();
                    //     }}
                    //     ui.text_edit_singleline(&mut self.new_tag);    
                    // });
                    //     ui.horizontal(|ui| {
                    //         ui.add_space(24.0);
                    //         if let Some(tag_ref) = &mut self.tags_search.option {
                    //             if ui.button("Remove Tag").clicked {
                    //                 self.tags.retain(|s| s != tag_ref);
                    //                 tag_ref.clear();
                    //             }
                    //             egui::ComboBox::from_label("")
                    //             .selected_text(format!("{}", tag_ref))
                    //             .show_ui(ui, |ui| {
                    //                 for tag in &self.tags {
                    //                     ui.selectable_value(tag_ref, tag.to_string(), format!("{tag}"));
                    //                 }
                    //             });
                    //         }
                    //     });
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label("Filenames with Common Protools AudioSuite Tags will be marked for removal")
                        });
                        
                    ui.horizontal(|_| {});
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.compare_db.enabled, "Compare against database: ");
                        if let Some(path) = &self.compare_db.option {
                            ui.label(path.split('/').last().unwrap());
                        }

                        
                        // ui.text_edit_singleline(&mut self.compare_db_path);
                        if ui.button("Select DB").clicked() {
                            self.compare_db.option = open_db();
                                
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
                    });

                }
                Panel::Order => {
                    if self.help {
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
                    }
                    
                    for (index, line) in self.order.iter_mut().enumerate() {
                        let checked = self.sel_line == Some(index);
                        if ui.selectable_label(checked, line.clone()).clicked {
                            self.sel_line = if checked { None } else { Some(index) };
                        }
                    }
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
                                self.new_line.clear();
                        }}
                        ui.text_edit_singleline(&mut self.new_line);    
                        if ui.button("Help").clicked {self.help = !self.help}
                    });
                    ui.separator();
                    if ui.button("Text Editor").clicked() {
                        self.order_text = self.order.join("\n");
                        self.my_panel = Panel::OrderText;
                    }
                    
                }

                Panel:: OrderText => {
                    if self.help {
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
                    }

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
                    ui.heading("Tag Editor");
                    ui.label("Protools Audiosuite Tags use the following format:  -example_");
                    ui.label("You can enter any string of text and if it is a match, the file will be marked for removal");
                    
                    ui.separator();
                    let num_columns = 6;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("Tags Grid")
                        .num_columns(num_columns)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for (index, tag) in self.tags.iter_mut().enumerate() {
                                // Check if current index is in `sel_tags`
                                let is_selected = self.sel_tags.contains(&index);
                                
                                if ui.selectable_label(is_selected, tag.clone()).clicked() {
                                    if is_selected {
                                        // Deselect
                                        self.sel_tags.retain(|&i| i != index);
                                    } else {
                                        // Select
                                        self.sel_tags.push(index);
                                    }
                                }
                                
                                if (index + 1) % num_columns == 0 {
                                    ui.end_row(); // Move to the next row after 4 columns
                                }
                            }
                            
                            // End the last row if not fully filled
                            if self.tags.len() % 4 != 0 {
                                ui.end_row();
                            }
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Add Tag:").clicked() && !self.new_tag.is_empty() {

                            self.tags.push(self.new_tag.clone());
                            self.new_tag.clear(); // Clears the string      
                            self.tags.sort_by_key(|s| s.to_lowercase());
                        }
                        ui.text_edit_singleline(&mut self.new_tag);
                        
                        
                    });
                    if ui.button("Remove Selected Tags").clicked() {
                        // Sort and remove elements based on `sel_tags`
                        let mut sorted_indices: Vec<usize> = self.sel_tags.clone();
                        sorted_indices.sort_by(|a, b| b.cmp(a)); // Sort in reverse order
                
                        for index in sorted_indices {
                            if index < self.tags.len() {
                                self.tags.remove(index);
                            }
                        }
                
                        // Clear the selection list after removal
                        self.sel_tags.clear();
                    }
                   
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