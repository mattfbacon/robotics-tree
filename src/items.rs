pub struct Item {
	pub name: &'static str,
	pub description: &'static str,
	pub content: &'static str,
}

pub(crate) static MAIN_ITEMS: &[Item] = &[
	Item {
		name: "abc",
		description: "def",
		content: include_str!("../items/main/abc.md"),
	},
	Item {
		name: "abc2",
		description: "def2",
		content: include_str!("../items/main/abc2.md"),
	},
];

pub(crate) static PROGRAMMING_ITEMS: &[Item] = &[Item {
	name: "aprogrambc",
	description: "a",
	content: include_str!("../items/programming/aprogrambc.md"),
}];

pub(crate) static BUILDING_ITEMS: &[Item] = &[
	Item {
		name: "abuildingc",
		description: "b",
		content: include_str!("../items/building/abuildingc.md"),
	},
	Item {
		name: "abuildingd",
		description: "baaa",
		content: "other content",
	},
];

pub(crate) static SIDEBAR_ITEMS: &[Item] = &[Item {
	name: "Screws",
	description: "How to use screws, tell them apart, etc",
	content: include_str!("../items/sidebar/screws.md"),
}];
