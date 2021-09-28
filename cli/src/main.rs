use usvg::NodeExt;
use usvg::PathSegment;

fn render_paths(thompson_paths: Vec<(usvg::PathData, usvg::Paint)>, ucb_paths: Vec<(usvg::PathData, usvg::Paint)>, rtree: &usvg::Tree) {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tminimal <name> <out-png>");
        return;
    }

    let mut opt = usvg::Options {
        resources_dir: std::fs::canonicalize(&args[1])
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        ..usvg::Options::default()
    };

    // Get file's absolute directory.
    opt.fontdb.load_system_fonts();
    let svg_data = include_str!("../example.svg").replace("## NAME HERE ##", &args[1]);

    let rtree = usvg::Tree::from_data(&svg_data.as_bytes(), &opt.to_ref()).unwrap();

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
    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&rtree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();
    pixmap.save_png(&args[2]).unwrap();
}
