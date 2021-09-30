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
        include_str!("../template.svg").replace("## NAME HERE ##", &data.best_player_name);
    let base_tree = usvg::Tree::from_data(&svg_data.as_bytes(), &opt.to_ref()).unwrap();
    let rtree = usvg::Tree::create(*base_tree.svg_node());
    let pixmap_size = rtree.svg_node().size.to_screen_size();

    let mut top_pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    // have a white background for the top portion
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(tiny_skia::Color::WHITE);
    top_pixmap.fill_rect(
        tiny_skia::Rect::from_xywh(0.0, 0.0, 794.0, 638.0).unwrap(),
        &paint,
        tiny_skia::Transform::identity(),
        None,
    );

    resvg::render(&base_tree, usvg::FitTo::Original, top_pixmap.as_mut()).unwrap();

    let mut output_map = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    let total_iterations = data
        .thompson
        .len()
        .max(data.ucb.len())
        .max(data.naive.len())
        .max(data.best_player.len());
    for i in 0..=total_iterations {
        state.update(&data, i);
        let new_tree = state.render(&rtree);
        output_map.fill(tiny_skia::Color::WHITE);
        resvg::render(&new_tree, usvg::FitTo::Original, output_map.as_mut()).unwrap();
        output_map.draw_pixmap(
            0,
            0,
            top_pixmap.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::identity(),
            None,
        );

        output_map.save_png(&format!("./images/{}.png", i)).unwrap();
        println!("{}/{}", i, total_iterations)
    }
}
