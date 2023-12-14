use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
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
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.content.push(c);
        } else {
            let mut result: String = self.content[..].graphemes(true).take(at).collect();
            let remainder: String = self.content[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.content = result;
        }
        self.update_len();
    }
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } else {
            let mut result: String = self.content[..].graphemes(true).take(at).collect();
            let remainder: String = self.content[..].graphemes(true).skip(at + 1).collect();
            result.push_str(&remainder);
            self.content = result;
        }
        self.update_len();
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
    pub fn append(&mut self, new: &Self) {
        self.content = format!("{}{}", self.content, new.content);
        self.update_len();
    }
    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = self.content[..].graphemes(true).take(at).collect();
        let remainder: String = self.content[..].graphemes(true).skip(at).collect();
        self.content = beginning;
        self.update_len();
        Self::from(&remainder[..])
    }
}