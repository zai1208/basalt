use std::ops::Range;

// TODO: More generic naming to use this pattern in explorer too
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Heading {
        range: Range<usize>,
        content: String,
    },
    HeadingEntry {
        range: Range<usize>,
        content: String,
        children: Vec<Item>,
        expanded: bool,
    },
}

impl Item {
    pub fn get_range(&self) -> &Range<usize> {
        match self {
            Item::Heading { range, .. } | Item::HeadingEntry { range, .. } => range,
        }
    }
    fn contains_index(&self, index: usize) -> bool {
        self.get_range().contains(&index)
    }
}

fn flatten(item: &Item) -> Vec<Item> {
    match item {
        Item::Heading { .. }
        | Item::HeadingEntry {
            expanded: false, ..
        } => {
            vec![item.clone()]
        }
        Item::HeadingEntry {
            expanded: true,
            children,
            ..
        } => {
            let mut items = vec![item.clone()];
            items.extend(children.iter().flat_map(flatten));
            items
        }
    }
}

pub trait Flatten {
    fn flatten(&self) -> Vec<Item>;
}

impl Flatten for Vec<Item> {
    fn flatten(&self) -> Vec<Item> {
        self.iter().flat_map(flatten).collect()
    }
}

pub trait FindItem {
    fn find_item(&self, index: usize) -> Option<(usize, Item)>;
}

impl FindItem for Vec<Item> {
    fn find_item(&self, index: usize) -> Option<(usize, Item)> {
        self.flatten()
            .into_iter()
            .enumerate()
            .find_map(|(i, item)| item.contains_index(index).then_some((i, item)))
    }
}
