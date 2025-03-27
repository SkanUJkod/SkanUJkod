extern crate tree_sitter;

use std::io::{self, Write};
use std::collections::VecDeque;
use tree_sitter::{ Node };


pub struct AstNode<'a> {
    node: Node<'a>,
}

impl<'a> AstNode<'a> {
    pub fn new(node: Node<'a>) -> Self {
        Self { node }
    }

    pub fn kind(&self) -> &str {
        self.node.kind()
    }

    pub fn child_count(&self) -> usize {
        self.node.child_count()
    }

    pub fn child(&self, field_name: &str) -> Option<Self> {
        self.node.child_by_field_name(field_name).map(Self::new)
    }

    // Traversal Methods
    pub fn pre_order_traversal<F>(&self, callback: &mut F)
    where F: FnMut(&Self),
    {
        callback(self);
        for i in 0..self.child_count() {
            if let Some(child) = self.node.child(i) {
                Self::new(child).pre_order_traversal(callback);
            }
        }
    }

    pub fn post_order_traversal<F>(&self, callback: &mut F)
    where F: FnMut(&Self),
    {
        for i in 0..self.child_count() {
            if let Some(child) = self.node.child(i) {
                Self::new(child).post_order_traversal(callback);
            }
        }
        callback(self);
    }

    pub fn breadth_first_traversal<F>(&self, mut callback: F)
    where F: FnMut(&Self),
    {
        let mut queue = VecDeque::new();
        queue.push_back(Self::new(self.node));

        while let Some(current) = queue.pop_front() {
            callback(&current);
            for i in 0..current.child_count() {
                if let Some(child) = current.node.child(i) {
                    queue.push_back(Self::new(child));
                }
            }
        }
    }

    // Querying Methods
    pub fn find_nodes<F>(&self, condition: F) -> Vec<Self>
    where F: Fn(&Self) -> bool,
    {
        let mut result = Vec::new();
        self.pre_order_traversal(&mut |node| {
            if condition(node) {
                result.push(Self::new(node.node));
            }
        });
        result
    }

    pub fn find_nodes_by_type(&self, kind: &str) -> Vec<Self> {
        self.find_nodes(|node| node.kind() == kind)
    }

    pub fn find_nodes_by_attribute<F>(&self, attribute_fn: F, kind: &str) -> Vec<Self>
    where F: Fn(&Self) -> bool,
    {
        self.find_nodes(|node| node.kind() == kind && attribute_fn(node))
    }

    pub fn find_pattern<F>(&self, pattern_fn: F) -> Vec<Self>
    where F: Fn(&Self) -> bool,
    {
        self.find_nodes(pattern_fn)
    }

    // Utility Methods
    pub fn print_tree<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.print_tree_with_indent(writer, 0)
    }

    fn print_tree_with_indent<W: Write>(&self, writer: &mut W, indent_level: usize) -> io::Result<()> {
        writeln!(writer, "{}{}", "  ".repeat(indent_level), self.kind())?;
        for i in 0..self.child_count() {
            if let Some(child) = self.node.child(i) {
                AstNode::new(child).print_tree_with_indent(writer, indent_level + 1)?;
            }
        }
        Ok(())
    }
}