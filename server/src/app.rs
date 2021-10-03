use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Bernoulli, Beta, Distribution};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

const EXPLORATION_TRIALS: usize = 30;

/// Actor for managing state
pub struct AppState {
	/// a map to the history of flips for each participant
	past: HashMap<String, Vec<(usize, bool)>>,
	/// algorithm states
	algorithm_state: AlgoState,
	/// whether or not to print out log messages
	verbose: bool,
}

impl AppState {
	pub fn new(rng: ThreadRng, prob_heads: Vec<f64>, verbose: bool) -> AppState {
		AppState {
			past: HashMap::new(),
			algorithm_state: AlgoState::new(
				rng,
				prob_heads
					.iter()
					.map(|p| Bernoulli::new(*p).unwrap())
					.collect(),
			),
			verbose,
		}
	}
	fn to_dump(&self) -> Dump {
		Dump {
			algorithms: self.algorithm_state.to_dump(),
			players: self
				.past
				.iter()
				.map(|e| (e.0.clone(), e.1.clone()))
				.collect::<Vec<(String, Vec<(usize, bool)>)>>(),
		}
	}
}

/// the current state of the algorithms
struct AlgoState {
	/// The state of naive
	naive: NaiveAlgoState,
	/// The past of Upper Condfidence Bound
	ucb: UcbAlgoState,
	/// The past of Thompson Sampling
	thompson: ThompsonAlgoState,
	/// The rng for the algorithms
	rng: ThreadRng,
	/// The arms / coins
	arms: Vec<Bernoulli>,
}

impl AlgoState {
	/// Create a new AlgoState
	fn new(rng: ThreadRng, arms: Vec<Bernoulli>) -> AlgoState {
		AlgoState {
			naive: NaiveAlgoState::new(arms.len()),
			ucb: UcbAlgoState::new(arms.len()),
			thompson: ThompsonAlgoState::new(arms.len()),
			rng,
			arms,
		}
	}
	/// Run every algorithm once
	fn update(&mut self) {
		self.naive.choose_flip(&mut self.rng, &self.arms);
		self.ucb.choose_flip(&mut self.rng, &self.arms);
		self.thompson.choose_flip(&mut self.rng, &self.arms);
	}
	/// Dump the state of the algorithms
	fn to_dump(&self) -> Vec<(String, Vec<(usize, bool)>)> {
		vec![
			self.naive.to_dump(),
			self.ucb.to_dump(),
			self.thompson.to_dump(),
		]
	}
}

trait Algorithm {
	fn new(num_arms: usize) -> Self;
	fn choose_flip(&mut self, rng: &mut ThreadRng, arms: &[Bernoulli]);
	fn flip(&mut self, rng: &mut ThreadRng, probs: &[Bernoulli], arm: usize);
	fn to_dump(&self) -> (String, Vec<(usize, bool)>);
}

struct NaiveAlgoState {
	/// The heads and tails seen during exploration period for each arm
	stats: Vec<(u32, u32)>,
	/// The past of naive: which coin it flipped and the result
	past_flips: Vec<(usize, bool)>,
	/// Best coin.
	/// The best coin seen so far, evaluated once after the exploration phase has ended
	best_coin: Option<usize>,
}

impl Algorithm for NaiveAlgoState {
	fn new(num_arms: usize) -> NaiveAlgoState {
		NaiveAlgoState {
			stats: vec![(0, 0); num_arms],
			past_flips: Vec::new(),
			best_coin: None,
		}
	}
	fn choose_flip(&mut self, rng: &mut ThreadRng, probs: &[Bernoulli]) {
		if self.past_flips.len() < EXPLORATION_TRIALS {
			// continue exploration phase
			let arm = rng.gen_range(0..probs.len());
			self.flip(rng, probs, arm);
		} else if let Some(index) = self.best_coin {
			// flip the best coin seen in exploration phase
			self.flip(rng, probs, index);
		} else {
			// find the best coin seen in exploration phase and set its index
			let index = self
				.stats
				.iter()
				.enumerate()
				.fold((0, 0.0), |a, index_heads_tails| {
					let proportion = index_heads_tails.1 .0 as f64
						/ (index_heads_tails.1 .0 as f64 + index_heads_tails.1 .1 as f64);
					if proportion > a.1 {
						(index_heads_tails.0, proportion)
					} else {
						a
					}
				})
				.0;
			self.best_coin = Some(index);
			self.flip(rng, probs, index);
		}
	}
	fn flip(&mut self, rng: &mut ThreadRng, probs: &[Bernoulli], arm: usize) {
		let result = probs[arm].sample(rng);
		if result {
			self.stats[arm].0 += 1;
		} else {
			self.stats[arm].1 += 1;
		}
		self.past_flips.push((arm, result));
	}
	fn to_dump(&self) -> (String, Vec<(usize, bool)>) {
		(
			"Naive Strategy".to_string(),
			self.past_flips
				.iter()
				.copied()
				.collect::<Vec<(usize, bool)>>(),
		)
	}
}

struct UcbAlgoState {
	/// The past of UCB
	past_flips: Vec<(usize, bool)>,
	/// Heads, tails seen for each arm (ucb without an exploration period)
	arm_results: Vec<(u32, u32)>,
	/// Total flips so far
	total_flips: u32,
}

impl Algorithm for UcbAlgoState {
	fn new(num_arms: usize) -> UcbAlgoState {
		UcbAlgoState {
			past_flips: Vec::new(),
			arm_results: vec![(0, 0); num_arms],
			total_flips: 0,
		}
	}
	fn choose_flip(&mut self, rng: &mut ThreadRng, arms: &[Bernoulli]) {
		let arm = self
			.arm_results
			.iter()
			.enumerate()
			.fold((0, 0.0), |a, index_heads_tails| {
				let proportion = index_heads_tails.1 .0 as f64
					/ (index_heads_tails.1 .0 as f64 + index_heads_tails.1 .1 as f64);
				let confidence = proportion
					+ f64::sqrt(
						2.0 * f64::log(self.total_flips as f64, 10.0)
							/ (index_heads_tails.1 .0 + index_heads_tails.1 .1) as f64,
					);
				if confidence > a.1 || confidence.is_nan() {
					(index_heads_tails.0, confidence)
				} else {
					a
				}
			});
		self.flip(rng, arms, arm.0);
	}
	fn flip(&mut self, rng: &mut ThreadRng, arms: &[Bernoulli], arm: usize) {
		let result = arms[arm].sample(rng);
		if result {
			self.arm_results[arm].0 += 1;
		} else {
			self.arm_results[arm].1 += 1;
		}
		self.past_flips.push((arm, result));
		self.total_flips += 1;
	}
	fn to_dump(&self) -> (String, Vec<(usize, bool)>) {
		(
			"UCB Strategy".to_string(),
			self.past_flips
				.iter()
				.copied()
				.collect::<Vec<(usize, bool)>>(),
		)
	}
}

struct ThompsonAlgoState {
	/// The past of Thompson
	past_flips: Vec<(usize, bool)>,
	/// Heads, tails, and beta distribution (storing = less update) seen for each arm
	arm_results: Vec<(u32, u32, Beta<f64>)>,
}

impl Algorithm for ThompsonAlgoState {
	fn new(num_arms: usize) -> ThompsonAlgoState {
		ThompsonAlgoState {
			past_flips: Vec::new(),
			arm_results: vec![(1, 1, Beta::new(1.0, 1.0).unwrap()); num_arms],
		}
	}
	fn choose_flip(&mut self, rng: &mut ThreadRng, arms: &[Bernoulli]) {
		// choose the arm with the highest sample from its beta distribution
		let arm = self
			.arm_results
			.iter()
			.enumerate()
			.fold((0, 0.0), |a, (index, (_, _, beta))| {
				let sample = beta.sample(rng);
				if sample > a.1 {
					(index, sample)
				} else {
					a
				}
			})
			.0;
		self.flip(rng, arms, arm);
	}
	fn flip(&mut self, rng: &mut ThreadRng, arms: &[Bernoulli], index: usize) {
		let mut arm = &mut self.arm_results[index];
		let result = arms[index].sample(rng);
		if result {
			arm.0 += 1;
		} else {
			arm.1 += 1;
		}
		arm.2 = Beta::new(arm.0 as f64, arm.1 as f64).unwrap();
		self.past_flips.push((index, result));
	}
	fn to_dump(&self) -> (String, Vec<(usize, bool)>) {
		(
			"Thompson Strategy".to_string(),
			self.past_flips
				.iter()
				.copied()
				.collect::<Vec<(usize, bool)>>(),
		)
	}
}

impl Actor for AppState {
	type Context = actix::Context<Self>;
}

#[derive(Serialize)]
pub struct Dump {
	algorithms: Vec<(String, Vec<(usize, bool)>)>,
	players: Vec<(String, Vec<(usize, bool)>)>,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CoinFlipped {
	pub user_id: String,
	pub arm: usize,
	pub result: bool,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Flush {}

/// Register a new player \
/// Forwarded from App
#[derive(Message, Debug)]
#[rtype(i32)]
pub struct GetCount {
	pub id: String,
}

/// Handler for CoinFlipped message.
impl Handler<CoinFlipped> for AppState {
	type Result = ();
	fn handle(&mut self, msg: CoinFlipped, _: &mut Context<Self>) -> Self::Result {
		if self.verbose {
			println!("{:?}", msg);
		}
		self.past
			.entry(msg.user_id)
			.or_insert_with(Vec::new)
			.push((msg.arm, msg.result));
		// self.algorithm_state.update();
	}
}

/// Handler for CoinFlipped message.
impl Handler<Flush> for AppState {
	type Result = ();
	fn handle(&mut self, msg: Flush, _: &mut Context<Self>) -> Self::Result {
		if self.verbose {
			println!("{:?}", msg);
		}
		let string = serde_json::to_string(&self.to_dump()).unwrap();
		fs::write("dump.json", string).unwrap();
	}
}

/// Handler for CoinFlipped message.
impl Handler<GetCount> for AppState {
	type Result = i32;
	fn handle(&mut self, msg: GetCount, _: &mut Context<Self>) -> Self::Result {
		if self.verbose {
			println!("{:?}", msg);
		}
		self.past
			.get(&msg.id)
			.map(|past| past.iter().fold(0, |a, e| if e.1 { a + 1 } else { a - 1 }))
			.unwrap_or(0)
	}
}
