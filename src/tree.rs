pub struct Tree<T> {
    pub root: Node<T>,
}

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub children: Vec<Node<T>>,
}

impl<T> Tree<T> {
    pub fn new(value: T) -> Self {
        Tree {
            root: Node {
                value,
                children: Vec::new(),
            },
        }
    }
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, value: T) {
        let node = Node::new(value);
        self.children.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree() {
        let mut tree = Tree::new(1);
        tree.root.add_child(2);
        tree.root.add_child(3);
        tree.root.children[0].add_child(4);
        tree.root.children[0].add_child(5);
        tree.root.children[1].add_child(6);
        tree.root.children[1].add_child(7);

        assert_eq!(tree.root.value, 1);
        assert_eq!(tree.root.children.len(), 2);
        assert_eq!(tree.root.children[0].value, 2);
        assert_eq!(tree.root.children[0].children.len(), 2);
        assert_eq!(tree.root.children[0].children[0].value, 4);
        assert_eq!(tree.root.children[0].children[1].value, 5);
        assert_eq!(tree.root.children[1].value, 3);
        assert_eq!(tree.root.children[1].children.len(), 2);
        assert_eq!(tree.root.children[1].children[0].value, 6);
        assert_eq!(tree.root.children[1].children[1].value, 7);
    }
}
