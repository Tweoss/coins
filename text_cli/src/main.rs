mod utils;

use std::fs::File;
use utils::*;

fn main() {
    // get number of iterations from args
    let args: Vec<String> = std::env::args().collect();
    let iterations = if args.len() > 1 {
        args[1].parse::<usize>().unwrap()
    } else {
        println!("No iterations given, using default of 100");
        100
    };

    let data = Dump::load("../server/dump.cbor").to_filtered();
    let mut state = RenderState::new();

    // iterate for the longest number of turns taken. thompson, ucb, and naive should all have the same length
    let total_iterations = usize::min(data.thompson.len().max(data.best_player.len()), iterations);
    let mut output = RenderedStateContainer::new(total_iterations, data.best_player_name.clone());

    for i in 0..total_iterations {
        // update what each algorithm sees
        state.update(&data, i);
        // render to the state
        state.render(&mut output.state[i]);
    }
    // use serde to dump information
    let file = File::create("rendered_dump.cbor").expect("Could not open output file");
    serde_cbor::to_writer(file, &output).expect("Could not write to output file");
}
