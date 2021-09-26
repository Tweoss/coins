use usvg::NodeExt;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tminimal <in-svg> <out-png>");
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

    let svg_data = std::fs::read(&args[1]).unwrap();
    let mut rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    use usvg::PathSegment;
    let data = usvg::PathData(vec![
        PathSegment::MoveTo { x: 0.0, y: 0.0 },
        PathSegment::LineTo { x: 1.0, y: 1.0 },
        PathSegment::LineTo { x: 1.0, y: 0.0 },
        PathSegment::LineTo { x: 0.0, y: 0.0 },
        PathSegment::LineTo { x: 0.0, y: 1.0 },
        PathSegment::LineTo { x: 1.0, y: 1.0 },
        PathSegment::ClosePath,
    ]);

    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
        // id: "coin1beta".into(),
        data: std::rc::Rc::new(data),
        fill: Some(usvg::Fill {
            paint: usvg::Paint::Color(usvg::Color::new_rgba(0, 0, 0, 0)),
            opacity: usvg::NormalizedValue::new(1.0),
            ..usvg::Fill::default()
        }),
        stroke: Some(usvg::Stroke {
            paint: usvg::Paint::Color(usvg::Color::new_rgb(205, 0, 0)),
            width: usvg::StrokeWidth::new(0.005),
            ..usvg::Stroke::default()
        }),
        transform: usvg::Transform::new(52.92, 0.0, 0.0, -52.92, 31.7625, 232.815),
        //   <path d="M 0 0.0 L 1 1 L 1 0 L 0.0 0.00 L 0.0 1.0 L 1.0 1.0 Z" fill="transparent" stroke="red" stroke-width="0.005"/>
        // x1: 0.0,
        // y1: 0.0,
        // x2: 1.0,
        // y2: 0.0,
        // base: usvg::BaseGradient {
        //     units: usvg::Units::ObjectBoundingBox,
        //     transform: usvg::Transform::default(),
        //     spread_method: usvg::SpreadMethod::Pad,
        //     stops: vec![
        //         usvg::Stop {
        //             offset: usvg::StopOffset::new(0.0),
        //             color: usvg::Color::new_rgb(0, 255, 0),
        //             opacity: usvg::Opacity::new(1.0),
        //         },
        //         usvg::Stop {
        //             offset: usvg::StopOffset::new(1.0),
        //             color: usvg::Color::new_rgb(0, 255, 0),
        //             opacity: usvg::Opacity::new(0.0),
        //         },
        //     ],
        // },
        ..usvg::Path::default()
    }));
    println!("{:?}", rtree.to_string(&usvg::XmlOptions::default()));

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&rtree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();
    pixmap.save_png(&args[2]).unwrap();
}
