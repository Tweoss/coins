mod utils;
use utils::*;
use std::fs;

fn main() {

    let data = Dump::load("../server/dump.json").to_filtered();
    let mut state = RenderState::new();

    // iterate for the longest number of turns taken. thompson, ucb, and naive should all have the same length
    let total_iterations = data.thompson.len().max(data.best_player.len());
    let mut output = RenderedStateContainer::new(total_iterations, data.best_player_name.clone());

    for i in 0..total_iterations {
        // update what each algorithm sees
        state.update(&data, i);
        // render to the state
        state.render(&mut output.state[i]);
    }
    // use serde to dump information
	let string = serde_json::to_string(&output).unwrap();
    fs::write("rendered_dump.json", string).unwrap();
}
