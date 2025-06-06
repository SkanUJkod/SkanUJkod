use std::collections::VecDeque;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: String,
    pub name: Option<String>,
    pub type_info: Option<String>,
    pub children: Vec<AstNode>,
}

impl AstNode {
    #[must_use]
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
            name: None,
            type_info: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_name(kind: &str, name: &str) -> Self {
        Self {
            kind: kind.to_string(),
            name: Some(name.to_string()),
            type_info: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_name_and_type(kind: &str, name: &str, type_info: &str) -> Self {
        Self {
            kind: kind.to_string(),
            name: Some(name.to_string()),
            type_info: Some(type_info.to_string()),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    #[must_use]
    pub fn recursive_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(Self::recursive_count) // Directly call the method instead of using a closure
            .sum::<usize>()
    }

    // Traversal Methods
    pub fn pre_order_traversal<F>(&self, visit: &mut F)
    where
        F: FnMut(&Self),
    {
        visit(self);
        for child in &self.children {
            child.pre_order_traversal(visit);
        }
    }

    pub fn post_order_traversal<F>(&self, visit: &mut F)
    where
        F: FnMut(&Self),
    {
        for child in &self.children {
            child.post_order_traversal(visit);
        }
        visit(self);
    }

    pub fn breadth_first_traversal<F>(&self, mut visit: F)
    where
        F: FnMut(&Self),
    {
        let mut queue = VecDeque::new();
        queue.push_back(self);

        while let Some(current) = queue.pop_front() {
            visit(current);
            for child in &current.children {
                queue.push_back(child);
            }
        }
    }

    // Querying Methods
    pub fn find_nodes<F>(&self, condition: F) -> Vec<Self>
    where
        F: Fn(&Self) -> bool,
    {
        let mut result: Vec<AstNode> = Vec::new();
        self.breadth_first_traversal(|node: &Self| {
            if condition(node) {
                result.push(node.clone());
            }
        });
        result
    }

    #[must_use]
    pub fn find_nodes_by_kind(&self, kind: &str) -> Vec<Self> {
        self.find_nodes(|node| node.kind == kind)
    }

    // Utility Methods
    pub fn print_tree(&self) {
        self.print_tree_with_indent(0);
    }

    fn print_tree_with_indent(&self, level: usize) {
        println!("{}{}", "  ".repeat(level), self.kind);
        for child in &self.children {
            child.print_tree_with_indent(level + 1);
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn write_tree<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        self.write_tree_with_indent(writer, 0)
    }

    fn write_tree_with_indent<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        level: usize,
    ) -> io::Result<()> {
        write!(writer, "{}{}", "  ".repeat(level), self.kind)?;
        if let Some(ref name) = self.name {
            write!(writer, " Name: {name}")?;
        }
        if let Some(ref type_info) = self.type_info {
            write!(writer, " Type: {type_info}")?;
        }
        writeln!(writer)?;
        for child in &self.children {
            child.write_tree_with_indent(writer, level + 1)?;
        }
        Ok(())
    }
}
