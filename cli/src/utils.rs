use serde::Deserialize;
use usvg::NodeExt;
use usvg::PathSegment;

#[derive(Deserialize)]
pub struct Dump {
	algorithms: Vec<(String, Vec<(usize, bool)>)>,
	players: Vec<(String, Vec<(usize, bool)>)>,
}

impl Dump {
	pub fn load(path: &str) -> Self {
		let file = std::fs::File::open(path).unwrap();
		let reader = std::io::BufReader::new(file);
		let dump: Dump = serde_json::from_reader(reader).unwrap();
		dump
	}
	pub fn to_filtered(&self) -> FilteredData {
		let mut filtered = FilteredData::new();
		for (algorithm, data) in &self.algorithms {
			match algorithm.as_str() {
				"Thompson Strategy" => {
					filtered.thompson = data.clone();
				}
				"UCB Strategy" => {
					filtered.ucb = data.clone();
				}
				"Naive Strategy" => {
					filtered.naive = data.clone();
				}
				_ => (),
			}
		}
		let temp_vec = Vec::new();
		let (name, _, past) = self.players.iter().fold(
			("", 0.0, &temp_vec),
			|(acc_name, acc_proportion, acc_past), (name, past)| {
				let (flips, heads) =
					past.iter()
						.fold((0, 0), |(a_flips, a_heads), (_, success)| {
							(a_flips + 1, if *success { a_heads + 1 } else { a_heads })
						});
				if heads as f64 / flips as f64 > acc_proportion {
					(&name, heads as f64 / flips as f64, &past)
				} else {
					(acc_name, acc_proportion, acc_past)
				}
			},
		);
		filtered.best_player_name = name.to_string();
		filtered.best_player = past.clone();
		filtered
	}
}

pub struct FilteredData {
	pub thompson: Vec<(usize, bool)>,
	pub naive: Vec<(usize, bool)>,
	pub ucb: Vec<(usize, bool)>,
	pub best_player: Vec<(usize, bool)>,
	pub best_player_name: String,
}

impl FilteredData {
	fn new() -> Self {
		Self {
			thompson: Vec::new(),
			naive: Vec::new(),
			ucb: Vec::new(),
			best_player: Vec::new(),
			best_player_name: String::new(),
		}
	}
}

pub struct GeneralState {
	/// Count for each coin
	count: Vec<usize>,
	successes: usize,
	failures: usize,
}

impl GeneralState {
	fn new() -> Self {
		Self {
			count: vec![0; 3],
			successes: 0,
			failures: 0,
		}
	}
}

pub struct ThompsonBetaState {
	a: Vec<usize>,
	b: Vec<usize>,
}

impl ThompsonBetaState {
	fn new() -> Self {
		Self {
			a: vec![1; 3],
			b: vec![1; 3],
		}
	}
}

pub struct UcbCountState {
	successes: Vec<usize>,
	total_flips: usize,
}

impl UcbCountState {
	fn new() -> Self {
		UcbCountState {
			successes: vec![0; 3],
			total_flips: 0,
		}
	}
}

pub struct RenderState {
	thompson: (GeneralState, ThompsonBetaState),
	ucb: (GeneralState, UcbCountState),
	naive: GeneralState,
	player: GeneralState,
}

impl RenderState {
	pub fn new() -> Self {
		Self {
			thompson: (GeneralState::new(), ThompsonBetaState::new()),
			ucb: (GeneralState::new(), UcbCountState::new()),
			naive: GeneralState::new(),
			player: GeneralState::new(),
		}
	}
	pub fn update(&mut self, data: &FilteredData, index: usize) {
		if let Some((coin, result)) = data.thompson.get(index) {
			self.thompson.0.count[*coin] += 1;
			self.thompson.0.successes += if *result { 1 } else { 0 };
			self.thompson.0.failures += if *result { 0 } else { 1 };
			self.thompson.1.a[*coin] += if *result { 1 } else { 0 };
			self.thompson.1.b[*coin] += if *result { 0 } else { 1 };
		};
		if let Some((coin, result)) = data.naive.get(index) {
			self.naive.count[*coin] += 1;
			self.naive.successes += if *result { 1 } else { 0 };
			self.naive.failures += if *result { 0 } else { 1 };
		};
		if let Some((coin, result)) = data.ucb.get(index) {
			self.ucb.0.count[*coin] += 1;
			self.ucb.0.successes += if *result { 1 } else { 0 };
			self.ucb.0.failures += if *result { 0 } else { 1 };
			self.ucb.1.successes[*coin] += if *result { 1 } else { 0 };
			self.ucb.1.total_flips += 1;
		};
		if let Some((coin, result)) = data.best_player.get(index) {
			self.player.count[*coin] += 1;
			self.player.successes += if *result { 1 } else { 0 };
			self.player.failures += if *result { 0 } else { 1 };
		};
	}
	pub fn render(&self, base_svg: &usvg::Tree) -> usvg::Tree {
		let svg = usvg::Tree::create(*base_svg.svg_node());
		svg.root().clone_from(&base_svg.root().make_deep_copy());
		render_thompson(&svg, &self.thompson.1);

		render_boxes(
			&svg,
			0,
			self.thompson.0.count[0],
			self.thompson.0.count[1],
			self.thompson.0.count[2],
			self.thompson.0.failures + self.thompson.0.successes,
		);
		render_boxes(
			&svg,
			1,
			self.naive.count[0],
			self.naive.count[1],
			self.naive.count[2],
			self.naive.failures + self.naive.successes,
		);
		render_boxes(
			&svg,
			2,
			self.ucb.0.count[0],
			self.ucb.0.count[1],
			self.ucb.0.count[2],
			self.ucb.0.failures + self.ucb.0.successes,
		);
		render_boxes(
			&svg,
			3,
			self.player.count[0],
			self.player.count[1],
			self.player.count[2],
			self.player.failures + self.player.successes,
		);

		render_text(
			&svg,
			(self.thompson.0.successes, self.thompson.0.failures),
			(self.naive.successes, self.naive.failures),
			(self.ucb.0.successes, self.ucb.0.failures),
			(self.player.successes, self.player.failures),
		);
		svg
	}
}

fn render_boxes(
	tree: &usvg::Tree,
	index: usize,
	count1: usize,
	count2: usize,
	count3: usize,
	total: usize,
) {
	let base_height = 37.041668;
	let width = 26.458332;
	let scale_height = 142.874995 - base_height;
	let base_width = match index {
		0 => 26.458332,
		1 => 68.791664,
		2 => 111.125,
		3 => 153.458332,
		_ => panic!("Invalid index"),
	};
	let (p1, p2, p3) = (
		count1 as f64 / total as f64,
		count2 as f64 / total as f64,
		count3 as f64 / total as f64,
	);

	fn append(tree: &usvg::Tree, x: f64, y: f64, width: f64, height: f64, fill: usvg::Fill) {
		tree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
			data: std::rc::Rc::new(usvg::PathData(vec![
				PathSegment::MoveTo { x, y },
				PathSegment::LineTo { x: x + width, y },
				PathSegment::LineTo {
					x: x + width,
					y: y + height,
				},
				PathSegment::LineTo { x, y: y + height },
				PathSegment::LineTo { x, y },
				PathSegment::ClosePath,
			])),
			stroke: Some(usvg::Stroke {
				paint: usvg::Paint::Color(usvg::Color::new_rgb(249, 249, 249)),
				width: usvg::StrokeWidth::new(0.632455),
				..usvg::Stroke::default()
			}),
			fill: Some(fill),
			..usvg::Path::default()
		}));
	}

	append(
		tree,
		base_width,
		base_height + 0.0,
		width,
		scale_height * p1,
		usvg::Fill::from_paint(usvg::Paint::Color(usvg::Color::new_rgb(0, 157, 255))),
	);
	append(
		tree,
		base_width,
		base_height + scale_height * p1,
		width,
		scale_height * p2,
		usvg::Fill::from_paint(usvg::Paint::Color(usvg::Color::new_rgb(255, 95, 89))),
	);
	append(
		tree,
		base_width,
		base_height + scale_height * (p1 + p2),
		width,
		scale_height * p3,
		usvg::Fill::from_paint(usvg::Paint::Color(usvg::Color::new_rgb(0, 176, 89))),
	);
}

pub fn render_paths(
	thompson_paths: Vec<(usvg::PathData, usvg::Paint)>,
	ucb_paths: Vec<(usvg::PathData, usvg::Paint)>,
	rtree: &usvg::Tree,
) {
	for path in thompson_paths {
		rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
			data: std::rc::Rc::new(path.0),
			stroke: Some(usvg::Stroke {
				paint: path.1,
				width: usvg::StrokeWidth::new(0.005),
				..usvg::Stroke::default()
			}),
			transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 31.7625, 232.815),
			..usvg::Path::default()
		}));
	}
	for path in ucb_paths {
		rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
			data: std::rc::Rc::new(path.0),
			stroke: Some(usvg::Stroke {
				paint: path.1,
				width: usvg::StrokeWidth::new(0.005),
				..usvg::Stroke::default()
			}),
			transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 116.445, 232.815),
			..usvg::Path::default()
		}));
	}
}

fn render_thompson(tree: &usvg::Tree, thompson: &ThompsonBetaState) {
	let (a1, a2, a3) = (thompson.a[0], thompson.a[1], thompson.a[2]);
	let (b1, b2, b3) = (thompson.b[0], thompson.b[1], thompson.b[2]);
	fn append(tree: &usvg::Tree, a: f64, b: f64, stroke_color: usvg::Paint) {
		use rv::prelude::ContinuousDistr;
		let dist = rv::dist::Beta::new(a, b).unwrap();
		let mut path = vec![PathSegment::MoveTo {
			x: 0.001,
			y: dist.pdf(&0.001),
		}];
		path.append(
			&mut (1..20)
				.map(|i| {
					let x = i as f64 / 20.0;
					let y = dist.pdf(&x);
					PathSegment::LineTo { x, y }
				})
				.collect::<Vec<PathSegment>>(),
		);
		path.push(PathSegment::MoveTo { x: 1.0, y: 0.0 });
		path.push(PathSegment::ClosePath);
		tree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
			data: std::rc::Rc::new(usvg::PathData(path)),
			stroke: Some(usvg::Stroke {
				paint: stroke_color,
				width: usvg::StrokeWidth::new(0.02),
				..usvg::Stroke::default()
			}),
			transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 31.7625, 232.815),
			..usvg::Path::default()
		}));
	}
	append(
		tree,
		a1 as f64,
		b1 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(0, 157, 255)),
	);
	append(
		tree,
		a2 as f64,
		b2 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(255, 95, 89)),
	);
	append(
		tree,
		a3 as f64,
		b3 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(0, 176, 89)),
	);
	// block out anything above
	tree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
		data: std::rc::Rc::new(usvg::PathData(vec![
			PathSegment::MoveTo {
				x: 0.0,
				y: 1.1969697,
			},
			PathSegment::LineTo {
				x: 1.0,
				y: 1.1969697,
			},
			PathSegment::LineTo { x: 1.0, y: 50.0 },
			PathSegment::LineTo { x: 0.0, y: 50.0 },
			PathSegment::ClosePath,
		])),
		transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 31.7625, 232.815),
		fill: Some(usvg::Fill::from_paint(usvg::Paint::Color(
			usvg::Color::new_rgb(255, 255, 255),
		))),
		stroke: None,
		..usvg::Path::default()
	}));
}

fn render_ucb(tree: &usvg::Tree, thompson: &ThompsonBetaState) {
	let (a1, a2, a3) = (thompson.a[0], thompson.a[1], thompson.a[2]);
	let (b1, b2, b3) = (thompson.b[0], thompson.b[1], thompson.b[2]);
	fn append(tree: &usvg::Tree, a: f64, b: f64, stroke_color: usvg::Paint) {
		use rv::prelude::ContinuousDistr;
		let dist = rv::dist::Beta::new(a, b).unwrap();
		let mut path = vec![PathSegment::MoveTo { x: 0.0, y: 0.0 }];
		path.append(
			&mut (1..20)
				.map(|i| {
					let x = i as f64 / 20.0;
					let y = dist.pdf(&x);
					PathSegment::LineTo { x, y }
				})
				.collect::<Vec<PathSegment>>(),
		);
		path.push(PathSegment::MoveTo { x: 1.0, y: 0.0 });
		path.push(PathSegment::ClosePath);
		tree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
			data: std::rc::Rc::new(usvg::PathData(path)),
			stroke: Some(usvg::Stroke {
				paint: stroke_color,
				width: usvg::StrokeWidth::new(0.02),
				..usvg::Stroke::default()
			}),
			transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 31.7625, 232.815),
			..usvg::Path::default()
		}));
	}
	append(
		tree,
		a1 as f64,
		b1 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(0, 157, 255)),
	);
	append(
		tree,
		a2 as f64,
		b2 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(255, 95, 89)),
	);
	append(
		tree,
		a3 as f64,
		b3 as f64,
		usvg::Paint::Color(usvg::Color::new_rgb(0, 176, 89)),
	);
}

fn render_text(
	tree: &usvg::Tree,
	thompson: (usize, usize),
	naive: (usize, usize),
	ucb: (usize, usize),
	player: (usize, usize),
) {
	let mut opt = usvg::Options {
		..usvg::Options::default()
	};
	opt.fontdb.load_system_fonts();
	let svg_data = include_str!("../text_template.svg")
		.replace(
			"## THOMPSON COUNT ##",
			&format!("{}/{}", thompson.0, thompson.0 + thompson.1),
		)
		.replace(
			"## NAIVE COUNT ##",
			&format!("{}/{}", naive.0, naive.0 + naive.1),
		)
		.replace("## UCB COUNT ##", &format!("{}/{}", ucb.0, ucb.0 + ucb.1))
		.replace(
			"## PLAYER COUNT ##",
			&format!("{}/{}", player.0, player.1 + player.1),
		);
	let text_tree = usvg::Tree::from_data(&svg_data.as_bytes(), &opt.to_ref()).unwrap();
	tree.root().append(text_tree.root().make_deep_copy());
}
