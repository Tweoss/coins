#![allow(unused_braces)]
use web_sys::console::log;
use mogwai::prelude::*;

/// One item - keeps track of clicks.
pub struct Audio {
    pub id: usize,
    pub yes: bool,
}

#[derive(Clone)]
pub enum AudioIn {
    Remove,
}

#[derive(Clone)]
pub enum AudioOut {
    /// Remove the item from the parent view
    Remove(usize),
}

impl Component for Audio {
    type ModelMsg = AudioIn;
    type ViewMsg = AudioOut;
    type DomNode = HtmlElement;

    fn update(
        &mut self,
        msg: &Self::ModelMsg,
        tx: &Transmitter<Self::ViewMsg>,
        _sub: &Subscriber<Self::ModelMsg>,
    ) {
        match msg {
            // tell the subscriber to remove this item
            AudioIn::Remove => {
                tx.send(&AudioOut::Remove(self.id));
            }
        }
    }

    fn view(
        &self,
        tx: &Transmitter<Self::ModelMsg>,
        _rx: &Receiver<Self::ViewMsg>,
    ) -> ViewBuilder<Self::DomNode> {
        let (tx_play, rx_play) = txrx();
        let t = tx.clone();
        rx_play.respond(move |e: &Event| {
            macro_rules! console_log {
                // Note that this is using the `log` function imported above during
                // `bare_bones`
                ($($t:tt)*) => (let mut __a__ = js_sys::Array::new(); __a__.set(0, format_args!($($t)*).to_string().into()); log(&__a__))
            }
            
            console_log!("HI AGAINN");
            if e.target()
                .expect("Must have target for event")
                .unchecked_ref::<web_sys::HtmlAudioElement>()
                .play()
                .is_err()
            {
                t.send(&AudioIn::Remove);
            }
        });
        if self.yes {
            builder! {
                <audio on:canplay=tx_play.contra_map(|e: &Event| e.clone()) on:ended=tx.contra_map(|_| AudioIn::Remove)>
                    <source src="./audio/yes.mp3" type="audio/mpeg"></source>
                </audio>
            }
        } else {
            builder! {
                <audio on:canplay=tx_play.contra_map(|e: &Event| e.clone()) on:ended=tx.contra_map(|_| AudioIn::Remove)>
                    <source src="./audio/no.mp3" type="audio/mpeg"></source>
                </audio>
            }
        }
    }
}
