use serde::Deserialize;
use std::default::Default;

const RESOLUTION_OF_DISTRIBUTION_SAMPLING: usize = 40;

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
	past: Vec<(usize, usize)>,
	total_flips: usize,
}

impl UcbCountState {
	fn new() -> Self {
		UcbCountState {
			past: vec![(0, 0); 3],
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

#[derive(serde::Serialize)]
pub struct RenderedStateContainer {
	pub state: Vec<RenderedState>,
	pub best_player_name: String,
}

impl RenderedStateContainer {
	pub fn new(length: usize, best_player_name: String) -> Self {
		Self {
			state: vec![RenderedState::default(); length],
			best_player_name,
		}
	}
}

#[derive(Default, Clone, serde::Serialize)]
pub struct RenderedState {
	pub thompson_rects: (Rectangle, Rectangle, Rectangle),
	pub ucb_rects: (Rectangle, Rectangle, Rectangle),
	pub naive_rects: (Rectangle, Rectangle, Rectangle),
	pub player_rects: (Rectangle, Rectangle, Rectangle),
	pub thompson_counts: (usize, usize),
	pub ucb_counts: (usize, usize),
	pub naive_counts: (usize, usize),
	pub player_counts: (usize, usize),
	pub thompson_paths: (String, String, String),
	pub ucb_paths: (String, String, String),
}

#[derive(Default, Clone, serde::Serialize)]
pub struct Rectangle {
	pub x: f64,
	pub y: f64,
	pub width: f64,
	pub height: f64,
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
			self.ucb.1.total_flips += 1;
			if *result {
				self.ucb.0.successes += 1;
				self.ucb.1.past[*coin].0 += 1;
			} else {
				self.ucb.0.failures += 1;
				self.ucb.1.past[*coin].1 += 1;
			}
		};
		if let Some((coin, result)) = data.best_player.get(index) {
			self.player.count[*coin] += 1;
			self.player.successes += if *result { 1 } else { 0 };
			self.player.failures += if *result { 0 } else { 1 };
		};
	}
	pub fn render(&self, base_state: &mut RenderedState){
		render_thompson(base_state, &self.thompson.1);
		render_ucb(base_state, &self.ucb.1);

		render_boxes(
			base_state,
			0,
			self.thompson.0.count[0],
			self.thompson.0.count[1],
			self.thompson.0.count[2],
			self.thompson.0.failures + self.thompson.0.successes,
		);
		render_boxes(
			base_state,
			1,
			self.naive.count[0],
			self.naive.count[1],
			self.naive.count[2],
			self.naive.failures + self.naive.successes,
		);
		render_boxes(
			base_state,
			2,
			self.ucb.0.count[0],
			self.ucb.0.count[1],
			self.ucb.0.count[2],
			self.ucb.0.failures + self.ucb.0.successes,
		);
		render_boxes(
			base_state,
			3,
			self.player.count[0],
			self.player.count[1],
			self.player.count[2],
			self.player.failures + self.player.successes,
		);

		render_text(
			base_state,
			(self.thompson.0.successes, self.thompson.0.failures),
			(self.naive.successes, self.naive.failures),
			(self.ucb.0.successes, self.ucb.0.failures),
			(self.player.successes, self.player.failures),
		);
		
	}
}

fn render_boxes(
	state: &mut RenderedState,
	index: usize,
	count1: usize,
	count2: usize,
	count3: usize,
	total: usize,
) {
	let base_height = 37.041668;
	let width = 26.458332;
	let scale_height = 142.874995 - base_height;
	let (base_width, output) = match index {
		0 => (26.458332, &mut state.thompson_rects),
		1 => (68.791664, &mut state.naive_rects),
		2 => (111.125, &mut state.ucb_rects),
		3 => (153.458332, &mut state.player_rects),
		_ => panic!("Invalid index"),
	};
	let (p1, p2, p3) = (
		count1 as f64 / total as f64,
		count2 as f64 / total as f64,
		count3 as f64 / total as f64,
	);

	fn append(x: f64, y: f64, width: f64, height: f64) -> Rectangle {
		Rectangle {
			x,
			y,
			width,
			height,
		}
	}

	output.0 = append(base_width, base_height + 0.0, width, scale_height * p1);
	output.1 = append(
		base_width,
		base_height + scale_height * p1,
		width,
		scale_height * p2,
	);
	output.2 = append(
		base_width,
		base_height + scale_height * (p1 + p2),
		width,
		scale_height * p3,
	);
}

fn render_thompson(state: &mut RenderedState, thompson: &ThompsonBetaState) {
	let (a1, a2, a3) = (thompson.a[0], thompson.a[1], thompson.a[2]);
	let (b1, b2, b3) = (thompson.b[0], thompson.b[1], thompson.b[2]);
	fn append(a: usize, b: usize) -> String {
		use rv::prelude::ContinuousDistr;
		let a = a as f64;
		let b = b as f64;
		let dist = rv::dist::Beta::new(a, b).unwrap();
		let mut path = vec![format!("M {} {} ", 0.001, dist.pdf(&0.001))];
		path.append(
			&mut (1..RESOLUTION_OF_DISTRIBUTION_SAMPLING)
				.map(|i| {
					let x = i as f64 / RESOLUTION_OF_DISTRIBUTION_SAMPLING as f64;
					let y = dist.pdf(&x);
					format!("L {} {} ", x, y)
				})
				.collect::<Vec<String>>(),
		);
		path.push("M 1.0 0.0 Z".to_string());
		path.iter().fold(String::new(), |acc, x| acc + x)
	}
	state.thompson_paths.0 = append(a1, b1);
	state.thompson_paths.1 = append(a2, b2);
	state.thompson_paths.2 = append(a3, b3);
}

fn render_ucb(state: &mut RenderedState, ucb: &UcbCountState) {
	let temp = vec![ucb.past[0], ucb.past[1], ucb.past[2]]
		.iter()
		.map(|(a, b)| {
			let a = *a as f64;
			let b = *b as f64;
			let total = a + b;
			let total_flips = ucb.total_flips as f64;
			(
				a / (a + b),
				a / (a + b) + f64::sqrt(2.0 * f64::log(total_flips, 10.0) / total),
			)
		})
		.collect::<Vec<(f64, f64)>>();
	let ((mean1, upper1), (mean2, upper2), (mean3, upper3)) = (temp[0], temp[1], temp[2]);
	fn append(mean: f64, upper: f64, y_offset: f64) -> String {
		if mean.is_nan() || upper.is_nan() {
			return "M 0 0 L 0 0 Z".to_string();
		}
		format!("M {} {} L {} {} Z", mean, y_offset, upper, y_offset)
	}
	state.ucb_paths.0 = append(mean1, upper1, 0.25);
	state.ucb_paths.1 = append(mean2, upper2, 0.5);
	state.ucb_paths.2 = append(mean3, upper3, 0.75);
}

fn render_text(
	state: &mut RenderedState,
	thompson: (usize, usize),
	naive: (usize, usize),
	ucb: (usize, usize),
	player: (usize, usize),
) {
	state.thompson_counts = thompson;
	state.naive_counts = naive;
	state.ucb_counts = ucb;
	state.player_counts = player;
}
