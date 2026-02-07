pub mod ids;

use ids::{IDSTable, Tag, parse};

const WILDCARD_CHAR: char = '.';

// Shared search functions used by both CLI and WASM

pub fn search_find(table: &IDSTable, needle_strs: &[String]) -> Result<Vec<(char, Tag)>, String> {
    let needles = {
        let mut needles = vec![];
        for needle_str in needle_strs {
            let needle = parse(needle_str)
                .map_err(|_| format!("Cannot parse needle {}", needle_str))?;
            needles.push(needle);
        }
        needles
    };

    let mut result: Vec<(char, Tag)> = table.iter()
        .filter_map(|((k, t), ids)| {
            if needles.iter().all(|needle| table.ids_has_subcomponent(&ids, &needle)) {
                Some((*k, t.clone()))
            } else {
                None
            }
        })
        .collect();
    result.sort();
    Ok(result)
}

pub fn search_match(table: &IDSTable, pattern_str: &str) -> Result<Vec<(char, Tag)>, String> {
    let pattern = parse(pattern_str)
        .map_err(|_| format!("Cannot parse pattern {}", pattern_str))?;

    let mut result: Vec<(char, Tag) > = table.iter()
        .filter_map(|((k, t), ids)| {
            if table.ids_match(ids, &pattern, WILDCARD_CHAR) {
                Some((*k, t.clone()))
            } else {
                None
            }
        })
        .collect();
    result.sort();
    Ok(result)
}

pub fn search_pmatch(table: &IDSTable, pattern_str: &str) -> Result<Vec<(char, Tag)>, String> {
    let pattern = parse(pattern_str)
        .map_err(|_| format!("Cannot parse pattern {}", pattern_str))?;

    let mut result: Vec<_> = table.iter()
        .filter_map(|((k, t), ids)| {
            if table.ids_has_matching_subcomponent(&ids, &pattern, WILDCARD_CHAR) {
                Some((*k, t.clone()))
            } else {
                None
            }
        })
        .collect();
    result.sort();
    result.dedup();
    Ok(result)
}

// WASM-specific code
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use serde::{Deserialize, Serialize};
    use crate::ids::IDSTable;

    const CHAI_DATA: &str = include_str!("../chai.txt");

    #[derive(Serialize, Deserialize)]
    pub struct SearchResult {
        pub results: Vec<String>,
    }

    fn get_table() -> IDSTable {
        IDSTable::load_from_string(CHAI_DATA).expect("Failed to load embedded data")
    }

    #[wasm_bindgen]
    pub fn find(needles_str: String) -> JsValue {
        let table = get_table();
        let needle_strs: Vec<String> = needles_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let result = match crate::search_find(&table, &needle_strs) {
            Ok(tchars) => tchars.iter().map(|(c, t)| format!("{}{}", c, t)).collect(),
            Err(e) => vec![format!("Error: {}", e)],
        };

        serde_wasm_bindgen::to_value(&SearchResult { results: result }).unwrap()
    }

    #[wasm_bindgen]
    pub fn match_pattern(pattern: String) -> JsValue {
        let table = get_table();

        let result = match crate::search_match(&table, &pattern) {
            Ok(tchars) => tchars.iter().map(|(c, t)| format!("{}{}", c, t)).collect(),
            Err(e) => vec![format!("Error: {}", e)],
        };

        serde_wasm_bindgen::to_value(&SearchResult { results: result }).unwrap()
    }

    #[wasm_bindgen]
    pub fn pmatch(pattern: String) -> JsValue {
        let table = get_table();

        let result = match crate::search_pmatch(&table, &pattern) {
            Ok(tchars) => tchars.iter().map(|(c, t)| format!("{}{}", c, t)).collect(),
            Err(e) => vec![format!("Error: {}", e)],
        };

        serde_wasm_bindgen::to_value(&SearchResult { results: result }).unwrap()
    }
}

// Re-export for wasm32 target
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
