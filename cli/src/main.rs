use usvg::PathSegment;
mod utils;
use utils::*;

fn main() {
    let mut opt = usvg::Options {
        ..usvg::Options::default()
    };
    opt.fontdb.load_system_fonts();

    let data = Dump::load("../server/dump.json").to_filtered();
    let mut state = RenderState::new();

    let svg_data =
        include_str!("../example.svg").replace("## NAME HERE ##", &data.best_player_name);
    let rtree = usvg::Tree::from_data(&svg_data.as_bytes(), &opt.to_ref()).unwrap();
    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    let total_iterations =
    data
        .thompson
        .len()
        .max(data.ucb.len())
        .max(data.naive.len())
        .max(data.best_player.len());
    for i in 0..=total_iterations
    {
        state.update(&data, i);
        let new_tree = state.render(&rtree.clone());
        pixmap.fill(tiny_skia::Color::from_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
        resvg::render(&new_tree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();
        pixmap.save_png(&format!("./images/{}.png", i)).unwrap();
        println!("{}/{}", i, total_iterations)
    }

    let thompson_paths = vec![(
        usvg::PathData(vec![
            PathSegment::MoveTo { x: 0.0, y: 0.0 },
            PathSegment::LineTo { x: 1.0, y: 1.0 },
            PathSegment::LineTo { x: 1.0, y: 0.0 },
            PathSegment::LineTo { x: 0.0, y: 0.0 },
            PathSegment::LineTo { x: 0.0, y: 1.0 },
            PathSegment::LineTo { x: 1.0, y: 1.0 },
            PathSegment::ClosePath,
        ]),
        usvg::Paint::Color(usvg::Color::new_rgb(0, 157, 221)),
    )];

    let ucb_paths = vec![(
        usvg::PathData(vec![
            PathSegment::MoveTo { x: 0.0, y: 0.0 },
            PathSegment::LineTo { x: 1.0, y: 1.0 },
            PathSegment::LineTo { x: 1.0, y: 0.0 },
            PathSegment::LineTo { x: 0.0, y: 0.0 },
            PathSegment::LineTo { x: 0.0, y: 1.0 },
            PathSegment::LineTo { x: 1.0, y: 1.0 },
            PathSegment::ClosePath,
        ]),
        usvg::Paint::Color(usvg::Color::new_rgb(255, 95, 89)),
    )];

    render_paths(thompson_paths, ucb_paths, &rtree);
}
