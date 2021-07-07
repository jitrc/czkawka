extern crate gtk;
use crate::gui_data::GuiData;
use crate::help_functions::*;
use crate::notebook_enums::*;
use gtk::prelude::*;

// TODO add support for checking if really symlink doesn't point to correct directory/file

pub fn connect_button_hide(gui_data: &GuiData) {
    let gui_data = gui_data.clone();
    let buttons_hide = gui_data.bottom_buttons.buttons_hide.clone();
    let tree_view_duplicate_finder = gui_data.main_notebook.tree_view_duplicate_finder.clone();
    let notebook_main = gui_data.main_notebook.notebook_main.clone();
    let tree_view_empty_folder_finder = gui_data.main_notebook.tree_view_empty_folder_finder.clone();
    let tree_view_big_files_finder = gui_data.main_notebook.tree_view_big_files_finder.clone();
    let tree_view_empty_files_finder = gui_data.main_notebook.tree_view_empty_files_finder.clone();
    let tree_view_temporary_files_finder = gui_data.main_notebook.tree_view_temporary_files_finder.clone();
    let tree_view_similar_images_finder = gui_data.main_notebook.tree_view_similar_images_finder.clone();
    let tree_view_zeroed_files_finder = gui_data.main_notebook.tree_view_zeroed_files_finder.clone();
    let tree_view_same_music_finder = gui_data.main_notebook.tree_view_same_music_finder.clone();
    let tree_view_invalid_symlinks = gui_data.main_notebook.tree_view_invalid_symlinks.clone();
    let tree_view_broken_files = gui_data.main_notebook.tree_view_broken_files.clone();
    let image_preview_similar_images = gui_data.main_notebook.image_preview_similar_images;

    buttons_hide.connect_clicked(move |_| match to_notebook_main_enum(notebook_main.current_page().unwrap()) {
        NotebookMainEnum::Duplicate => {
            tree_hide(&tree_view_duplicate_finder.clone(), ColumnsDuplicates::Color as i32);
        }
        NotebookMainEnum::EmptyDirectories => {
            basic_hide(&tree_view_empty_folder_finder.clone());
        }
        NotebookMainEnum::EmptyFiles => {
            basic_hide(&tree_view_empty_files_finder.clone());
        }
        NotebookMainEnum::Temporary => {
            basic_hide(&tree_view_temporary_files_finder.clone());
        }
        NotebookMainEnum::BigFiles => {
            basic_hide(&tree_view_big_files_finder.clone());
        }
        NotebookMainEnum::SimilarImages => {
            tree_hide(&tree_view_similar_images_finder.clone(), ColumnsSimilarImages::Color as i32);
            image_preview_similar_images.hide();
        }
        NotebookMainEnum::Zeroed => {
            basic_hide(&tree_view_zeroed_files_finder.clone());
        }
        NotebookMainEnum::SameMusic => {
            tree_hide(&tree_view_same_music_finder.clone(), ColumnsSameMusic::Color as i32);
        }
        NotebookMainEnum::Symlinks => {
            basic_hide(&tree_view_invalid_symlinks.clone());
        }
        NotebookMainEnum::BrokenFiles => {
            basic_hide(&tree_view_broken_files.clone());
        }
    });
}

pub fn basic_hide(tree_view: &gtk::TreeView) {
    let selection = tree_view.selection();

    let (selection_rows, _) = selection.selected_rows();
    if selection_rows.is_empty() {
        return;
    }
    let list_store = get_list_store(&tree_view);

    // Must be deleted from end to start, because when deleting entries, TreePath(and also TreeIter) will points to invalid data
    for tree_path in selection_rows.iter().rev() {
        list_store.remove(&list_store.iter(tree_path).unwrap());
    }
    selection.unselect_all();
}

// Remove all occurrences - remove every element which have same path and name as even non selected ones
pub fn tree_hide(tree_view: &gtk::TreeView, column_color: i32) {
    let selection = tree_view.selection();

    let (selection_rows, tree_model) = selection.selected_rows();
    if selection_rows.is_empty() {
        return;
    }
    let list_store = get_list_store(&tree_view);

    // Save to variable paths of files, and remove it when not removing all occurrences.
    for tree_path in selection_rows.iter().rev() {
        list_store.remove(&list_store.iter(tree_path).unwrap());
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

    selection.unselect_all();
}
