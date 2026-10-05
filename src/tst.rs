/// A high-performance, Unicode-safe Ternary Search Tree (TST) for fast lexical symbol
/// and keyword lookups without fixed-character limitation or panics.

#[derive(Debug, Clone, Default)]
pub struct TstNode {
    pub ch: char,
    pub is_end: bool,
    pub left: Option<Box<TstNode>>,
    pub equal: Option<Box<TstNode>>,
    pub right: Option<Box<TstNode>>,
}

impl TstNode {
    #[inline]
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            is_end: false,
            left: None,
            equal: None,
            right: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Nodes {
    pub name: String,
    root: Option<Box<TstNode>>,
}

unsafe impl Send for Nodes {}
unsafe impl Sync for Nodes {}

impl Nodes {
    pub fn new(name: String) -> Self {
        Self {
            name,
            root: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insert(&mut self, word: &str) {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return;
        }
        let chars: Vec<char> = trimmed.to_uppercase().chars().collect();
        self.root = Self::insert_rec(self.root.take(), &chars, 0);
    }

    fn insert_rec(node: Option<Box<TstNode>>, chars: &[char], idx: usize) -> Option<Box<TstNode>> {
        let c = chars[idx];
        let mut curr = node.unwrap_or_else(|| Box::new(TstNode::new(c)));

        if c < curr.ch {
            curr.left = Self::insert_rec(curr.left.take(), chars, idx);
        } else if c > curr.ch {
            curr.right = Self::insert_rec(curr.right.take(), chars, idx);
        } else if idx + 1 < chars.len() {
            curr.equal = Self::insert_rec(curr.equal.take(), chars, idx + 1);
        } else {
            curr.is_end = true;
        }

        Some(curr)
    }

    pub fn contains(&self, word: &str) -> bool {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return false;
        }
        let chars: Vec<char> = trimmed.to_uppercase().chars().collect();
        let mut curr = self.root.as_deref();
        let mut idx = 0;

        while let Some(node) = curr {
            let c = chars[idx];
            if c < node.ch {
                curr = node.left.as_deref();
            } else if c > node.ch {
                curr = node.right.as_deref();
            } else {
                idx += 1;
                if idx == chars.len() {
                    return node.is_end;
                }
                curr = node.equal.as_deref();
            }
        }

        false
    }

    pub fn from_str(content: &str, name: &str) -> Self {
        let mut nodes = Self::new(name.to_string());
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                nodes.insert(trimmed);
            }
        }
        nodes
    }

    pub fn from_file(path: &str) -> Self {
        let sep = if cfg!(windows) { '\\' } else { '/' };
        let extn = path.split(sep).last().unwrap_or(path).to_string();
        let name = extn.split('.').next().unwrap_or(&extn).to_string();
        let mut nodes = Self::new(name);
        nodes.insert_from_file(path);
        nodes
    }

    pub fn insert_from_file(&mut self, path_any: &str) {
        let normalized = path_any.replace('\\', "/");
        let mut content = std::fs::read_to_string(&normalized);
        if content.is_err() {
            let parent_path = format!("../{}", normalized);
            content = std::fs::read_to_string(&parent_path);
        }
        if content.is_err() && normalized.ends_with("action_v.txt") {
            let alt = normalized.replace("action_v.txt", "action_V.txt");
            content = std::fs::read_to_string(&alt);
        }
        if let Ok(c) = content {
            for line in c.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    self.insert(trimmed);
                }
            }
        }
    }

    pub fn extends(&mut self, other: Nodes) {
        // Collect words or merge nodes
        // If other has root, we can traverse and insert
        Self::collect_words(other.root.as_deref(), &mut Vec::new(), self);
    }

    fn collect_words(node: Option<&TstNode>, current_prefix: &mut Vec<char>, target: &mut Nodes) {
        if let Some(n) = node {
            Self::collect_words(n.left.as_deref(), current_prefix, target);

            current_prefix.push(n.ch);
            if n.is_end {
                let word: String = current_prefix.iter().collect();
                target.insert(&word);
            }
            Self::collect_words(n.equal.as_deref(), current_prefix, target);
            current_prefix.pop();

            Self::collect_words(n.right.as_deref(), current_prefix, target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tst_basic() {
        let mut t = Nodes::new("test".into());
        t.insert("EURUSD");
        t.insert("BTCUSDT");
        t.insert("árbol");
        t.insert("🚀MOON");

        assert!(t.contains("EURUSD"));
        assert!(t.contains("eurusd"));
        assert!(t.contains("BTCUSDT"));
        assert!(t.contains("árbol"));
        assert!(t.contains("🚀moon"));
        assert!(!t.contains("ETHUSDT"));
    }
}
