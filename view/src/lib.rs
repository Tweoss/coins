use log::Level;
use mogwai::prelude::*;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log;
use web_sys::{HtmlInputElement, KeyboardEvent};

#[derive(serde::Deserialize, Default)]
struct RenderedStateContainer {
    #[allow(clippy::vec_box)]
    // box so that no cloning is performed => huge performance boost
    state: Vec<Box<RenderedState>>,
    best_player_name: String,
}

#[derive(Clone, serde::Deserialize)]
struct RenderedState {
    thompson_rects: (Rectangle, Rectangle, Rectangle),
    ucb_rects: (Rectangle, Rectangle, Rectangle),
    naive_rects: (Rectangle, Rectangle, Rectangle),
    player_rects: (Rectangle, Rectangle, Rectangle),
    thompson_counts: (usize, usize),
    ucb_counts: (usize, usize),
    naive_counts: (usize, usize),
    player_counts: (usize, usize),
    thompson_paths: (String, String, String),
    ucb_paths: (String, String, String),
}

#[derive(Clone, serde::Deserialize)]
struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! console_log {
    ($($t:tt)*) => (let mut __a__ = js_sys::Array::new(); __a__.set(0, format_args!($($t)*).to_string().into()); log(&__a__))
}

struct Viewer {
    data: RenderedStateContainer,
    index: usize,
}

#[derive(Clone)]
enum ViewerIn {
    Input(String),
    Forward,
    Backward,
    None,
}

#[derive(Clone)]
enum ViewerOut{
    Update(Box<RenderedState>),
    NameLength((String, usize)),
    Index(usize),
}

impl Component for Viewer {
    type ModelMsg = ViewerIn;
    type ViewMsg = ViewerOut;
    type DomNode = HtmlElement;

    fn update(
        &mut self,
        msg: &ViewerIn,
        tx_view: &Transmitter<ViewerOut>,
        _subscriber: &Subscriber<ViewerIn>,
    ) {
        match msg {
            ViewerIn::Input(data_string) => {
                if let Ok(data) = serde_json::from_str::<RenderedStateContainer>(&data_string) {
                    self.index = 0;
                    self.data = data;
                    tx_view.send(&ViewerOut::Update(
                        self.data.state[self.index].clone(),
                    ));
                    tx_view.send(&ViewerOut::NameLength((
                        self.data
                            .best_player_name
                            .splitn(2, '_')
                            .collect::<Vec<&str>>()[1]
                            .to_string(),
                            self.data.state.len())
                    ),
                );
                } else {
                    console_log!("{} could not be parsed.", data_string);
                }
            }
            ViewerIn::Forward => {
                if self.index + 1 < self.data.state.len() {
                    self.index += 1;
                    tx_view.send(&ViewerOut::Update(
                        self.data.state[self.index].clone(),
                    ));
                    tx_view.send(&ViewerOut::Index(self.index));
                }
            }
            ViewerIn::Backward => {
                if self.index > 0 {
                    self.index -= 1;
                    tx_view.send(&ViewerOut::Update(
                        self.data.state[self.index].clone(),
                    ));
                    tx_view.send(&ViewerOut::Index(self.index));
                }
            }
            ViewerIn::None => {}
        }
    }

    #[allow(unused_braces)]
    fn view(
        &self,
        tx: &Transmitter<ViewerIn>,
        rx: &Receiver<ViewerOut>,
    ) -> ViewBuilder<HtmlElement> {
        let rx_name = rx.branch_filter_map(|msg: &ViewerOut| match msg {
            ViewerOut::NameLength((name, _)) => Some(name.clone()),
            _ => None,
        });
        let rx_length = rx.branch_filter_map(|msg: &ViewerOut| match msg {
            ViewerOut::NameLength((_, length)) => Some(format!("{} ", length)),
            _ => None,
        });
        let rx_data = rx.branch_filter_map(|msg: &ViewerOut| match msg {
            ViewerOut::Update(data) => Some(data.clone()),
            _ => None,
        });
        let rx_index = rx.branch_filter_map(|msg: &ViewerOut| match msg {
            ViewerOut::Index(index) => Some(format!(" {}/", index.clone())),
            _ => None,
        });

        let tx_data = tx.contra_map(|e: &Event| {
            ViewerIn::Input(
                e.target()
                    .expect("Must have target for event")
                    .unchecked_ref::<HtmlInputElement>()
                    .value()
                    .trim()
                    .to_string(),
            )
        });

        let tx_key = tx.contra_map(|e: &Event| match e.unchecked_ref::<KeyboardEvent>().key() {
            e if e == *"ArrowRight" => ViewerIn::Forward,
            e if e == *"ArrowLeft" => ViewerIn::Backward,
            _ => ViewerIn::None,
        });
        let tx_forward = tx.contra_map(|_| ViewerIn::Forward);
        let tx_backward = tx.contra_map(|_| ViewerIn::Backward);

        let ns = "http://www.w3.org/2000/svg";

        builder!(
            <div on:keydown=tx_key>
                <svg style="width: 100%; height: 100%;" width= "793.7007874015749" height= "1122.5196850393702" viewBox= "0 0 210 297" xmlns= "http://www.w3.org/2000/svg" xmlns=ns>
                    <defs xmlns=ns>
                        <clipPath id="clipPath1" xmlns=ns>
                            <path fill= "rgba(0,0,0,255)" stroke= "none" d= "M 0 0 L 210 0 L 210 297 L 0 297 Z" xmlns=ns/>
                        </clipPath>
                    </defs>
                    <g clip-path= "url(#clipPath1)" xmlns=ns>
                        <path fill= "rgba(0,0,0,0)" stroke= "rgba(255,0,0,255)" stroke-width= "0.005" transform= "matrix(52.92 0 0 -52.92 31.7625 232.815)" d= "M 0 0 L 1 1 L 1 0 L 0 0 L 0 1 L 1 1 Z" />
                    </g>
                    <g clip-path= "url(#clipPath2)" xmlns=ns>
                        <path fill= "rgba(0,0,0,0)" stroke= "rgba(255,0,0,255)" stroke-width= "0.005" transform= "matrix(52.92 0 0 -52.92 116.445 232.815)" d= "M 0 0 L 1 1 L 1 0 L 0 0 L 0 1 L 1 1 Z" />
                    </g>
                    // ? horizontal lines below titles
                    <path id= "path975" fill= "none" stroke= "rgba(29,0,255,255)" stroke-width= "1.165" d= "M 26.458333 26.458333 L 52.916666 26.458333"  xmlns=ns/>
                    <path id= "path977" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" d= "M 68.791666 26.458333 L 95.249999 26.458333"  xmlns=ns/>
                    <path id= "path979" fill= "none" stroke= "rgba(230,156,255,255)" stroke-width= "1.165" d= "M 111.125 26.458333 L 137.58333 26.458333"  xmlns=ns/>
                    <path id= "path981" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" d= "M 153.45833 26.458333 L 179.91666 26.458333"  xmlns=ns/>
                    // ? titles (that don't change)
                    <text style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="39.6875" y="23.8125" xmlns=ns><tspan id="tspan1180" x="39.6875" y="23.8125" style="stroke-width:0.264583" xmlns=ns>{"Thompson"}</tspan></text>
                    <text style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="82.020836" y="23.8125" xmlns=ns><tspan id="tspan1184" x="82.020836" y="23.8125" style="stroke-width:0.264583" xmlns=ns>{"Naive"}</tspan></text>
                    <text style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="124.35416" y="23.8125" xmlns=ns><tspan id="tspan1188" x="124.35416" y="23.8125" style="stroke-width:0.264583" xmlns=ns>{"UCB"}</tspan></text>
                    // ? squares around graphs
                    <path id= "rect1196" fill= "rgba(0,0,0,255)" fill-opacity= "0" stroke= "rgba(29,0,255,255)" stroke-linejoin= "round" d= "M 26.458334 169.33333 L 95.250013 169.33333 L 95.250013 238.125002 L 26.458334 238.125002 Z"  xmlns=ns/>
                    <path id= "rect1196-5" fill= "rgba(0,0,0,255)" fill-opacity= "0" stroke= "rgba(230,156,255,255)" stroke-linejoin= "round" d= "M 111.125 169.33333 L 179.916679 169.33333 L 179.916679 238.125002 L 111.125 238.125002 Z"  xmlns=ns/>
                    // ? graph axes and ticks
                    <path id= "path1215" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.665" stroke-linejoin= "round" d= "M 31.75 174.625 L 31.75 232.83333 L 89.958332 232.83333"  xmlns=ns/>
                    <path id= "path1241" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(0.000000000000000024431703642989698 0.399 -0.399 0.000000000000000024431703642989698 31.75 174.625)" d= "M 8.7185878 4.0337352 L -2.2072895 0.016013256 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z"  xmlns=ns/>
                    <path id= "path1244" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(-0.399 0.000000000000000048863407285979396 -0.000000000000000048863407285979396 -0.399 89.958332 232.83333)" d= "M 8.7185878 4.0337352 L -2.2072895 0.016013256 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z"  xmlns=ns/>
                    <path id= "path1697" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" d= "M 84.666666 234.15625 C 84.666666 231.51041 84.666666 231.51041 84.666666 231.51041"  xmlns=ns/>
                    <path id= "path1693" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" d= "M 30.427083 179.91666 C 33.072916 179.91666 33.072916 179.91666 33.072916 179.91666"  xmlns=ns/>
                    <path id= "path1893" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.665" stroke-linejoin= "round" d= "M 31.75 174.625 L 31.75 232.83333 L 89.958332 232.83333"  xmlns=ns/>
                    <path id= "path2160" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(0.000000000000000024431703642989698 0.399 -0.399 0.000000000000000024431703642989698 31.75 174.625)" d= "M 8.7185878 4.0337352 L -2.2072895 0.016013256 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z"  xmlns=ns/>
                    <path id= "path2170" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(-0.399 0.000000000000000048863407285979396 -0.000000000000000048863407285979396 -0.399 89.958332 232.83333)" d= "M 8.7185878 4.0337352 L -2.2072895 0.016013256 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z" xmlns=ns/>
                    <path id= "path1899" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" d= "M 84.666666 234.15625 C 84.666666 231.51041 84.666666 231.51041 84.666666 231.51041"  xmlns=ns/>
                    <path id= "path1693-2" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" transform= "matrix(1 0 0 1 84.667335 0)" d= "M 30.427083 179.91666 C 33.072916 179.91666 33.072916 179.91666 33.072916 179.91666"  xmlns=ns/>
                    <path id= "path1893-0" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.665" stroke-linejoin= "round" transform= "matrix(1 0 0 1 84.667335 0)" d= "M 31.75 174.625 L 31.75 232.83333 L 89.958332 232.83333"  xmlns=ns/>
                    <path id= "path2160-5" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(0.000000000000000024431703642989698 0.399 -0.399 0.000000000000000024431703642989698 116.417335 174.625)" d= "M 8.7185878 4.0337352 L -2.2072895 0.01601326 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z"  xmlns=ns/>
                    <path id= "path2170-9" fill= "rgba(0,0,0,255)" fill-rule= "evenodd" stroke= "rgba(0,0,0,255)" stroke-width= "0.625" stroke-linejoin= "round" transform= "matrix(-0.399 0 0 -0.399 174.625667 232.83333)" d= "M 8.7185878 4.0337352 L -2.2072895 0.01601326 L 8.7185884 -4.0017078 C 6.97309 -1.6296469 6.9831476 1.6157441 8.7185878 4.0337352 Z"  xmlns=ns/>
                    <path id= "path1899-2" fill= "none" stroke= "rgba(0,0,0,255)" stroke-width= "0.264583" transform= "matrix(1 0 0 1 84.667335 0)" d= "M 84.666666 234.15625 C 84.666666 231.51041 84.666666 231.51041 84.666666 231.51041"  xmlns=ns/>
                    // ? the text for the axes
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="84.666664" y="238.125" id="text1903-1" xmlns=ns><tspan id="tspan1901-6" x="84.666664" y="238.125" style="stroke-width:0.264583" xmlns=ns>{"1"}</tspan></text>
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="29.104166" y="182.5625" id="text1691" xmlns=ns><tspan  id="tspan1689" x="29.104166" y="182.5625" style="stroke-width:0.264583" xmlns=ns>{"2"}</tspan></text>
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="84.666664" y="238.125" id="text1903-1" transform="translate(84.667335)" xmlns=ns><tspan id="tspan1901-6" x="84.666664" y="238.125" style="stroke-width:0.264583" xmlns=ns>{"2"}</tspan></text>
                    // ? the rectangles
                    <rect id="player-1" style="fill:#009dff;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.width)))} height={("42.333351", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.height)))} x={("153.45833", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.x)))} y={("37.041672", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.y)))} xmlns=ns/>
                    <rect id="player-2" style="fill:#ff5f59;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.width)))} height={("42.333355", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.height)))} x={("153.45833", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.x)))} y={("79.375023", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.y)))} xmlns=ns/>
                    <rect id="player-3" style="fill:#00b059;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.447212;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.width)))} height={("21.166685", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.height)))} x={("153.45833", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.x)))} y={("121.70834", rx_data.branch_map(|m| format!("{}", m.thompson_rects.0.y)))} xmlns=ns/>
                    <rect id="ucb-1" style="fill:#009dff;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.width)))} height={("42.333351", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.height)))} x={("111.125", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.x)))} y={("37.041649", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.y)))} xmlns=ns/>
                    <rect id="ucb-2" style="fill:#ff5f59;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.width)))} height={("42.333355", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.height)))} x={("111.125", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.x)))} y={("79.375", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.y)))} xmlns=ns/>
                    <rect id="ucb-3" style="fill:#00b059;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.447212;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.width)))} height={("21.166685", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.height)))} x={("111.125", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.x)))} y={("121.70831", rx_data.branch_map(|m| format!("{}", m.naive_rects.0.y)))} xmlns=ns/>
                    <rect id="naive-1" style="fill:#009dff;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.width)))} height={("42.333351", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.height)))} x={("68.791664", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.x)))} y={("37.041649", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.y)))} xmlns=ns/>
                    <rect id="naive-2" style="fill:#ff5f59;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.width)))} height={("42.333355", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.height)))} x={("68.791664", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.x)))} y={("79.375", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.y)))} xmlns=ns/>
                    <rect id="naive-3" style="fill:#00b059;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.447212;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.width)))} height={("21.166685", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.height)))} x={("68.791664", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.x)))} y={("121.70831", rx_data.branch_map(|m| format!("{}", m.ucb_rects.0.y)))} xmlns=ns/>
                    <rect id="thompson-1" style="fill:#009dff;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.player_rects.0.width)))} height={("42.333351", rx_data.branch_map(|m| format!("{}", m.player_rects.0.height)))} x={("26.458332", rx_data.branch_map(|m| format!("{}", m.player_rects.0.x)))} y={("37.041649", rx_data.branch_map(|m| format!("{}", m.player_rects.0.y)))} xmlns=ns/>
                    <rect id="thompson-2" style="fill:#ff5f59;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.632455;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.player_rects.0.width)))} height={("42.333355", rx_data.branch_map(|m| format!("{}", m.player_rects.0.height)))} x={("26.458332", rx_data.branch_map(|m| format!("{}", m.player_rects.0.x)))} y={("79.375", rx_data.branch_map(|m| format!("{}", m.player_rects.0.y)))} xmlns=ns/>
                    <rect id="thompson-3" style="fill:#00b059;fill-opacity:1;stroke:#f9f9f9;stroke-width:0.447212;stroke-linejoin:round;paint-order:markers fill stroke" width={("26.458334", rx_data.branch_map(|m| format!("{}", m.player_rects.0.width)))} height={("21.166685", rx_data.branch_map(|m| format!("{}", m.player_rects.0.height)))} x={("26.458332", rx_data.branch_map(|m| format!("{}", m.player_rects.0.x)))} y={("121.70831", rx_data.branch_map(|m| format!("{}", m.player_rects.0.y)))} xmlns=ns/>
                    // ? the text for the counts
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="39.6875" y="158.75" id="text1076" xmlns=ns><tspan x="39.6875" y="158.75" style="stroke-width:0.264583" id="tspan1078" xmlns=ns>{("THOMPSON COUNT", rx_data.branch_map(|m| format!("{}/{}", m.thompson_counts.0,  m.thompson_counts.0 + m.thompson_counts.1)))}</tspan></text>
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="82.020836" y="158.75" id="text1076-9" xmlns=ns><tspan x="82.020836" y="158.75" style="stroke-width:0.264583" id="tspan1078-4" xmlns=ns>{("NAIVE COUNT", rx_data.branch_map(|m| format!("{}/{}", m.naive_counts.0,  m.naive_counts.0 + m.naive_counts.1)))}</tspan></text>
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="124.56371" y="158.74974" id="text1076-5" xmlns=ns><tspan x="124.56371" y="158.74974" style="stroke-width:0.264583" id="tspan1078-9" xmlns=ns>{("UCB COUNT", rx_data.branch_map(|m| format!("{}/{}", m.ucb_counts.0,  m.ucb_counts.0 + m.ucb_counts.1)))}</tspan></text>
                    <text xml:space="preserve" style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="166.5509" y="158.74974" id="text1076-9-3" xmlns=ns><tspan  x="166.5509" y="158.74974" style="stroke-width:0.264583" id="tspan1078-4-9" xmlns=ns>{("PLAYER COUNT", rx_data.branch_map(|m| format!("{}/{}", m.player_counts.0,  m.player_counts.0 + m.player_counts.1)))}</tspan></text>
                    // ? the title for the player name
                    <text style="font-size:4.5861px;line-height:1.25;font-family:sans-serif;text-align:center;text-anchor:middle;stroke-width:0.264583" x="166.6875" y="23.8125" xmlns=ns><tspan id="tspan1192" x="166.6875" y="23.8125" style="stroke-width:0.264583" xmlns=ns>{("Player Name", rx_name)}</tspan></text>
                    // ? the paths for thompson
                    <path d={rx_data.branch_map(|m| m.thompson_paths.0.clone())} style="stroke:rgb(0,157,255);stroke-width:0.02;fill:none;" transform="matrix(52.92 0.0 0.0 -26.46 31.76225 232.815)" xmlns=ns/>
                    <path d={rx_data.branch_map(|m| m.thompson_paths.1.clone())} style="stroke:rgb(255,957,89);stroke-width:0.02;fill:none;" transform="matrix(52.92 0.0 0.0 -26.46 31.76225 232.815)" xmlns=ns/>
                    <path d={rx_data.branch_map(|m| m.thompson_paths.2.clone())} style="stroke:rgb(0,176,89);stroke-width:0.02;fill:none;" transform="matrix(52.92 0.0 0.0 -26.46 31.76225 232.815)" xmlns=ns/>
                    // ? the paths for ucb
                    <path d={rx_data.branch_map(|m| m.ucb_paths.0.clone())} style="stroke:rgb(0,157,255);stroke-width:0.02;fill:none;" transform="matrix(26.46 0.0 0.0 -52.92 116.445 232.815)" xmlns=ns/>
                    <path d={rx_data.branch_map(|m| m.ucb_paths.1.clone())} style="stroke:rgb(255,957,89);stroke-width:0.02;fill:none;" transform="matrix(26.46 0.0 0.0 -52.92 116.445 232.815)" xmlns=ns/>
                    <path d={rx_data.branch_map(|m| m.ucb_paths.2.clone())} style="stroke:rgb(0,176,89);stroke-width:0.02;fill:none;" transform="matrix(26.46 0.0 0.0 -52.92 116.445 232.815)" xmlns=ns/>
                </svg>
                <div>
                    <p><button on:click=tx_backward type="button">{"<-"}</button> <span>{(" 0/",rx_index)}</span>{("0 ",rx_length)} <button on:click=tx_forward type="button">{"->"}</button></p>
                </div>
                <br></br>
                <textarea on:change=tx_data rows="4" cols="50">{"Paste data here"}</textarea>
            </div>
        )
    }
}

#[wasm_bindgen]
pub fn main(parent_id: Option<String>) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    let gizmo = Gizmo::from(Viewer {
        data: RenderedStateContainer::default(),
        index: 0,
    });
    let view = View::from(gizmo.view_builder());

    if let Some(id) = parent_id {
        let parent = utils::document().get_element_by_id(&id).unwrap();
        view.run_in_container(&parent)
    } else {
        view.run()
    }
}
