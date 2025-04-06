use std::collections::VecDeque;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: String,
    pub children: Vec<AstNode>,
}

impl AstNode {
    pub fn new(kind: &str) -> Self {
        AstNode { 
            kind: kind.to_string(), 
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn recursive_count(&self) -> usize {
        1 + self.children.iter().map(|child| child.recursive_count()).sum::<usize>()
    }

    // Traversal Methods
    pub fn pre_order_traversal<F>(&self, visit: &mut F)
    where
        F: FnMut(&AstNode),
    {
        visit(self);
        for child in &self.children {
            child.pre_order_traversal(visit);
        }
    }

    pub fn post_order_traversal<F>(&self, visit: &mut F)
    where
        F: FnMut(&AstNode),
    {
        for child in &self.children {
            child.post_order_traversal(visit);
        }
        visit(self);
    }

    pub fn breadth_first_traversal<F>(&self, mut visit: F)
    where
        F: FnMut(&AstNode),
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
    pub fn find_nodes<F>(&self, condition: F) -> Vec<AstNode>
    where
        F: Fn(&AstNode) -> bool,
    {
        let mut result = Vec::new();
        self.breadth_first_traversal(|node| {
            if condition(node) {
                result.push(node.clone());
            }
        });
        result
    }

    pub fn find_nodes_by_kind(&self, kind: &str) -> Vec<AstNode> {
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

    pub fn write_tree<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
        self.write_tree_with_indent(writer, 0)
    }

    fn write_tree_with_indent<W: Write + ?Sized>(&self, writer: &mut W, level: usize) -> io::Result<()> {
        writeln!(writer, "{}{}", "  ".repeat(level), self.kind)?;
        for child in &self.children {
            child.write_tree_with_indent(writer, level + 1)?;
        }
        Ok(())
    }
}