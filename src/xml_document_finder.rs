use roxmltree::Node;

pub trait XmlDocumentFinder {
    fn find(&self, name: &str) -> Option<Node>;
    fn find_into<T>(&self, name: &str, convert: impl Fn(&str) -> Option<T>) -> Option<T>;
    fn filter(&self, name: &str) -> Vec<Node>;
}

impl XmlDocumentFinder for Node<'_, '_> {
    fn find(&self, name: &str) -> Option<Node> {
        let mut node = Some(*self);
        name.split('/').for_each(|n| {
            node = node
                .and_then(|it|
                    it.children().find(|child|
                        child.is_element() && child.tag_name().name() == n))
        });
        node
    }

    fn find_into<T>(&self, name: &str, convert: impl Fn(&str) -> Option<T>) -> Option<T> {
        self.find(name)
            .and_then(|node| node.text())
            .and_then(|text| convert(text))
    }

    fn filter(&self, name: &str) -> Vec<Node> {
        let mut nodes = vec![*self];
        name.split('/').for_each(|n| {
            nodes = nodes
                .iter()
                .map(|node| node.children())
                .flat_map(|child|
                    child
                        .filter(|node|
                            node.is_element() && node.tag_name().name() == n)
                        .collect::<Vec<Node>>()
                )
                .collect();
        });
        nodes
    }
}
