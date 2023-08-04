//! A wrapper around native file dialogs for egui.
//!
//! Based on the implementation from [kirjavascript/trueLMAO](https://github.com/kirjavascript/trueLMAO/tree/master).

use rfd;

pub struct FileDialog {
    file: Option<Vec<u8>>,
}

impl Default for FileDialog {
    fn default() -> Self {
        Self { file: None }
    }
}

impl FileDialog {
    pub fn open(&mut self) {
        let path = rfd::FileDialog::new().pick_file();
        if let Some(path) = path {
            self.file = std::fs::read(path).ok();
        }
    }

    pub fn get(&mut self) -> Option<Vec<u8>> {
        std::mem::replace(&mut self.file, None)
    }
}
