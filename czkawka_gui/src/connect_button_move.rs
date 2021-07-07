extern crate gtk;
use crate::gui_data::GuiData;
use crate::help_functions::*;
use crate::notebook_enums::*;
use gtk::prelude::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const MOVE_STR: &str = "!_MOVED_CZKAWAKA_";

pub fn connect_button_move(gui_data: &GuiData) {
    let gui_data = gui_data.clone();
    let shared_duplication_state = gui_data.shared_duplication_state.clone();
    let shared_similar_images_state = gui_data.shared_similar_images_state.clone();
    let shared_same_music_state = gui_data.shared_same_music_state.clone();
    let buttons_move = gui_data.bottom_buttons.buttons_move.clone();
    let tree_view_duplicate_finder = gui_data.main_notebook.tree_view_duplicate_finder.clone();
    let notebook_main = gui_data.main_notebook.notebook_main.clone();
    let tree_view_similar_images_finder = gui_data.main_notebook.tree_view_similar_images_finder.clone();
    let tree_view_same_music_finder = gui_data.main_notebook.tree_view_same_music_finder.clone();
    let image_preview_similar_images = gui_data.main_notebook.image_preview_similar_images.clone();

    buttons_move.connect_clicked(move |_| match to_notebook_main_enum(notebook_main.current_page().unwrap()) {
        NotebookMainEnum::Duplicate => {
            tree_move(
                &tree_view_duplicate_finder.clone(),
                ColumnsDuplicates::Name as i32,
                ColumnsDuplicates::Path as i32,
                ColumnsDuplicates::Color as i32,
                &shared_duplication_state.borrow_mut().get_base_paths(),
                &gui_data,
            );
        }
        NotebookMainEnum::SimilarImages => {
            tree_move(
                &tree_view_similar_images_finder.clone(),
                ColumnsSimilarImages::Name as i32,
                ColumnsSimilarImages::Path as i32,
                ColumnsSimilarImages::Color as i32,
                &shared_similar_images_state.borrow_mut().get_base_paths(),
                &gui_data,
            );
            image_preview_similar_images.hide();
        }
        NotebookMainEnum::SameMusic => {
            tree_move(
                &tree_view_same_music_finder.clone(),
                ColumnsSameMusic::Name as i32,
                ColumnsSameMusic::Path as i32,
                ColumnsSameMusic::Color as i32,
                &shared_same_music_state.borrow_mut().get_base_paths(),
                &gui_data,
            );
        }
        e => panic!("Not existent {:?}", e),
    });
}

// Remove all occurrences - remove every element which have same path and name as even non selected ones
pub fn tree_move(tree_view: &gtk::TreeView, column_file_name: i32, column_path: i32, column_color: i32, base_paths: &BTreeMap<String, PathBuf>, gui_data: &GuiData) {
    let text_view_errors = gui_data.text_view_errors.clone();

    let selection = tree_view.selection();

    let (selection_rows, tree_model) = selection.selected_rows();
    if selection_rows.is_empty() {
        return;
    }
    let list_store = get_list_store(&tree_view);

    let mut messages: String = "".to_string();

    let mut vec_path_to_delete: Vec<(String, String)> = Vec::new();
    let mut map_with_path_to_delete: BTreeMap<String, Vec<String>> = Default::default(); // BTreeMap<Path,Vec<FileName>>

    // Save to variable paths of files, and remove it when not removing all occurrences.
    for tree_path in selection_rows.iter().rev() {
        let file_name = tree_model.value(&tree_model.iter(tree_path).unwrap(), column_file_name).get::<String>().unwrap();
        let path = tree_model.value(&tree_model.iter(tree_path).unwrap(), column_path).get::<String>().unwrap();

        list_store.remove(&list_store.iter(tree_path).unwrap());

        map_with_path_to_delete.entry(path.clone()).or_insert_with(Vec::new);
        map_with_path_to_delete.get_mut(path.as_str()).unwrap().push(file_name);
    }

    // Delete duplicated entries, and remove real files
    for (path, mut vec_file_name) in map_with_path_to_delete {
        vec_file_name.sort();
        vec_file_name.dedup();
        for file_name in vec_file_name {
            let dir_path = PathBuf::from(path.clone());
            let full_file_path = dir_path.join(file_name.clone());
            let mut base_path = dir_path.clone();

            if let Some(result_path) = base_paths.get(&full_file_path.to_string_lossy().to_string()) {
                base_path = result_path.clone();
            }

            if full_file_path.starts_with(base_path.clone()) {
                let residue_dir = dir_path.strip_prefix(base_path.clone()).unwrap().to_path_buf();
                let new_dir = base_path.join(MOVE_STR).join(residue_dir);

                if fs::create_dir_all(new_dir.clone()).is_err() {
                    messages += format!("Failed to create drieectory {} to move file. It is possible you don't have permissions.\n", new_dir.display()).as_str()
                } else {
                    let new_path = new_dir.join(file_name.clone());
                    if fs::rename(&full_file_path, &new_path).is_err() {
                        messages += format!(
                            "Failed to move file {} to {}. It is possible that you already deleted it, or you don't have permissions.\n",
                            full_file_path.display(),
                            new_path.display()
                        )
                        .as_str()
                    } else {
                        vec_path_to_delete.push((path.clone(), file_name.clone()));
                    }
                }
            } else {
                messages += format!("Cannot move file {}/{}, with base_path {}\n", path, file_name, base_path.display()).as_str();
            }
        }
    }

    // Remove only child from header
    if let Some(first_iter) = list_store.iter_first() {
        let mut vec_tree_path_to_delete: Vec<gtk::TreePath> = Vec::new();
        let mut current_iter = first_iter;
        if tree_model.value(&current_iter, column_color).get::<String>().unwrap() != HEADER_ROW_COLOR {
            panic!("First deleted element, should be a header"); // First element should be header
        };

        let mut next_iter;
        let mut next_next_iter;
        'main: loop {
            if tree_model.value(&current_iter, column_color).get::<String>().unwrap() != HEADER_ROW_COLOR {
                panic!("First deleted element, should be a header"); // First element should be header
            };

            next_iter = current_iter.clone();
            if !list_store.iter_next(&next_iter) {
                // There is only single header left (H1 -> END) -> (NOTHING)
                vec_tree_path_to_delete.push(list_store.path(&current_iter).unwrap());
                break 'main;
            }

            if tree_model.value(&next_iter, column_color).get::<String>().unwrap() == HEADER_ROW_COLOR {
                // There are two headers each others(we remove just first) -> (H1 -> H2) -> (H2)
                vec_tree_path_to_delete.push(list_store.path(&current_iter).unwrap());
                current_iter = next_iter.clone();
                continue 'main;
            }

            next_next_iter = next_iter.clone();
            if !list_store.iter_next(&next_next_iter) {
                // There is only one child of header left, so we remove it with header (H1 -> C1 -> END) -> (NOTHING)
                vec_tree_path_to_delete.push(list_store.path(&current_iter).unwrap());
                vec_tree_path_to_delete.push(list_store.path(&next_iter).unwrap());
                break 'main;
            }

            if tree_model.value(&next_next_iter, column_color).get::<String>().unwrap() == HEADER_ROW_COLOR {
                // One child between two headers, we can remove them  (H1 -> C1 -> H2) -> (H2)
                vec_tree_path_to_delete.push(list_store.path(&current_iter).unwrap());
                vec_tree_path_to_delete.push(list_store.path(&next_iter).unwrap());
                current_iter = next_next_iter.clone();
                continue 'main;
            }

            loop {
                // (H1 -> C1 -> C2 -> Cn -> END) -> (NO CHANGE, BECAUSE IS GOOD)
                if !list_store.iter_next(&next_next_iter) {
                    break 'main;
                }
                // Move to next header
                if tree_model.value(&next_next_iter, column_color).get::<String>().unwrap() == HEADER_ROW_COLOR {
                    current_iter = next_next_iter.clone();
                    continue 'main;
                }
            }
        }
        for tree_path in vec_tree_path_to_delete.iter().rev() {
            list_store.remove(&list_store.iter(&tree_path).unwrap());
        }
    }

    // Last step, remove orphan header if exists
    if let Some(iter) = list_store.iter_first() {
        if !list_store.iter_next(&iter) {
            list_store.clear();
        }
    }

    text_view_errors.buffer().unwrap().set_text(messages.as_str());
    selection.unselect_all();
}
