use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

pub struct Row {
    content: String,
    len: usize
}

impl From <&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            content: String::from(slice), len: 0
        };
        row.update_len();
        row
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.content.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        for grapheme in self.content[..].graphemes(true).skip(start).take(end - start) {
            result.push_str(
                if grapheme == "\t" {"  "}
                else {grapheme}
            );
        }
        result
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn update_len(&mut self) {
        self.len = self.content[..].graphemes(true).count();
    }
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}