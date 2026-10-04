//! Data-directory resolution for the ui-ux-pro-max catalog.
//!
//! Python resolves `DATA_DIR = Path(__file__).parent.parent / "data"`. The
//! compiled binary has no script path, so we resolve in priority order:
//!   1. `CIEL_UIUX_DATA_DIR` environment variable
//!   2. `<ancestor>/skills/ui-ux-pro-max/data` walking up from the executable
//!   3. `<ancestor>/skills/ui-ux-pro-max/data` walking up from the cwd
//!   4. `~/.ciel/skills/ui-ux-pro-max/data` (installed skill location)

use crate::common::csvio::{self, read_csv, CsvTable};
use crate::common::py;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

fn ancestors_with_data(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join("skills/ui-ux-pro-max/data");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = env::var("CIEL_UIUX_DATA_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(found) = ancestors_with_data(parent) {
                return found;
            }
        }
    }
    if let Ok(cwd) = env::current_dir() {
        if let Some(found) = ancestors_with_data(&cwd) {
            return found;
        }
    }
    py::home_dir().join(".ciel/skills/ui-ux-pro-max/data")
}

/// Search context: data dir plus a per-process CSV row cache, mirroring
/// core.py's `_csv_cache` (one `ciel-uiux` run can issue many domain searches
/// over the same files during `--design-system`).
pub struct Catalog {
    pub dir: PathBuf,
    cache: RefCell<HashMap<PathBuf, Vec<csvio::Row>>>,
}

impl Default for Catalog {
    fn default() -> Catalog {
        Catalog::new()
    }
}

impl Catalog {
    pub fn new() -> Catalog {
        Catalog {
            dir: data_dir(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Explicit data dir — equivalent to `CIEL_UIUX_DATA_DIR`, used by tests
    /// and callers that pin a specific catalog.
    pub fn at(dir: impl Into<PathBuf>) -> Catalog {
        Catalog {
            dir: dir.into(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.join(rel)
    }

    /// `_load_csv` — cached DictReader rows.
    pub fn rows(&self, rel: &str) -> Result<Vec<csvio::Row>, String> {
        let path = self.path(rel);
        if let Some(rows) = self.cache.borrow().get(&path) {
            return Ok(rows.clone());
        }
        let table = read_csv(&path)?;
        let rows = table.rows;
        self.cache.borrow_mut().insert(path.clone(), rows.clone());
        Ok(rows)
    }

    /// `_load_rows_or_empty` — tolerate I/O failure for identity routing.
    pub fn rows_or_empty(&self, rel: &str) -> Vec<csvio::Row> {
        self.rows(rel).unwrap_or_default()
    }

    /// `_read_rows` — headers + rows, used by the validator.
    pub fn table(&self, rel: &str) -> Result<CsvTable, String> {
        read_csv(&self.path(rel))
    }
}
