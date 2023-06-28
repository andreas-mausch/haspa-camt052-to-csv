use std::error::Error;

use roxmltree::Node;

pub trait XmlDocumentFinder {
    fn find(&self, name: &str) -> Option<Node>;
    fn find_into<T>(&self, name: &str) -> Result<T, Box<dyn Error>>
        where T: for<'a> TryFrom<&'a str>,
              for<'a> <T as TryFrom<&'a str>>::Error: Error;
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

    fn find_into<T>(&self, name: &str) -> Result<T, Box<dyn Error>>
        where T: for<'a> TryFrom<&'a str>,
              for<'a> <T as TryFrom<&'a str>>::Error: Error {
        self.find(name)
            .and_then(|node| node.text())
            .ok_or::<Box<dyn Error>>(format!("No node '{}'", name).into())
            .and_then(|x| x.try_into().map_err(|e: T::Error| e.to_string().into()))
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
