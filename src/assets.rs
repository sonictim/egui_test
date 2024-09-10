use eframe::egui::{self, Ui, RichText};
use crate::app::*;

// A reusable button component that takes a function (callback) to run when clicked
pub fn button<F>(ui: &mut Ui, label: &str, action: F)
where
    F: FnOnce(),
{
    if ui.button(label).clicked() {
        action(); // Call the passed function when the button is clicked
    }
}


pub fn large_button<F>(ui: &mut Ui, label: &str, action: F)
where
    F: FnOnce(),
{
    if ui.add_sized([200.0, 50.0], egui::Button::new(RichText::new(label).size(24.0).strong())).clicked() {
        action();
    } 
}


pub fn combo_box(ui: &mut Ui, label: &str, selected: &mut String, list: &Vec<String>) {
    egui::ComboBox::from_id_source(label)
        .selected_text(selected.clone())
        .show_ui(ui, |ui| {
            for item in list {
                ui.selectable_value(selected, item.clone(), item);
            }
        });
}

pub fn order_help(ui: &mut Ui) {
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





                    //SMALL TAG EDITOR

                    // ui.horizontal(|ui| {
                    //     ui.add_space(24.0);
                    //     if ui.button("Add Tag:").clicked {
                    //         app.tags.sort_by_key(|s| s.to_lowercase());
                    //         if app.new_tag.len() > 0 {
                    //             app.tags.push(app.new_tag.clone());
                    //             app.new_tag = "".to_string();
                    //     }}
                    //     ui.text_edit_singleline(&mut app.new_tag);    
                    // });
                    //     ui.horizontal(|ui| {
                    //         ui.add_space(24.0);
                    //         if let Some(tag_ref) = &mut app.tags.option {
                    //             if ui.button("Remove Tag").clicked {
                    //                 app.tags.retain(|s| s != tag_ref);
                    //                 tag_ref.clear();
                    //             }
                    //             egui::ComboBox::from_label("")
                    //             .selected_text(format!("{}", tag_ref))
                    //             .show_ui(ui, |ui| {
                    //                 for tag in &app.tags {
                    //                     ui.selectable_value(tag_ref, tag.to_string(), format!("{tag}"));
                    //                 }
                    //             });
                    //         }
                    //     });