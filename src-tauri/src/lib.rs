mod commands;
mod db;
mod models;
pub mod research_adapter;

use commands::{
    create_map, delete_map, export_map_json, get_maps, load_map, rename_map, save_map_state,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_pool = db::init_db().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(db_pool)
        .invoke_handler(tauri::generate_handler![
            get_maps,
            create_map,
            delete_map,
            rename_map,
            load_map,
            save_map_state,
            export_map_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
