// Draw a pulsing circle in the middle of the window
extern crate quicksilver;
extern crate futures;
extern crate rand;

use rand::prelude::*;

use quicksilver::{
    State, run,
    geom::{Circle, Vector, Transform},
    input::{Event},
    graphics::{Color, Font, FontLoader, Image, ImageLoader, Draw, Window, WindowBuilder},
};

use futures::{Async, Future};

struct Mole {
    obj: Circle,
    ttl: usize,
    image: ImageViewer,
}
// SampleText::Loading(Font::load("examples/assets/font.ttf")) }

impl Mole {
    fn new(x: f32, y: f32, ttl: usize) -> Mole {
        Mole {
            obj: Circle::new(x, y, 50f32),
            ttl: ttl,
            image: ImageViewer::Loading(Image::load("assets/oyvind2.png"))
        }
    }
}

enum GameState {
    NotStarted,
    Started,
    Lost(usize),
}

struct Whac {
    step: usize,
    moles: [Option<Mole>; 5],
    score_text_loader: ScoreTextLoader,
    score_text_font: Option<Font>,
    score: usize,
    ttl: usize,
    miss: usize,
    state: GameState,
    delay: usize,
}

enum ScoreTextLoader {
    Loading(FontLoader),
    Loaded
}

enum ImageViewer {
    Loading(ImageLoader),
    Loaded(Image)
}

impl State for Whac {
    fn new() -> Whac {
        Whac {
            score_text_font: None,
            score_text_loader: ScoreTextLoader::Loading(Font::load("assets/font.ttf")),
            score: 0,
            step: 0,
            moles: [Some(Mole::new(200f32, 200f32, 90)), None, None, None, None],
            ttl: 90,
            miss: 0,
            state: GameState::Started,
            delay: 0,
        }
    }

    fn update(&mut self, _window: &mut Window) {

        if self.miss >= 10 {
            self.state = GameState::Lost(self.score);
        }

        for mole in self.moles.iter_mut() {
            if let Some(ref mut mole) = mole {
                // Check to see the progress of the loading image
                let result = match mole.image {
                    ImageViewer::Loading(ref mut loader) => loader.poll().unwrap(),
                    _ => Async::NotReady
                };
                // If the image has been loaded move to the loaded state
                if let Async::Ready(asset) = result {
                    mole.image = ImageViewer::Loaded(asset);
                }
            }

        }

        // Check to see the progress of the loading font
        let result = match self.score_text_loader {
            ScoreTextLoader::Loading(ref mut loader) => loader.poll().unwrap(),
            ScoreTextLoader::Loaded => Async::NotReady
        };

        if let Async::Ready(font) = result {
            self.score_text_loader = ScoreTextLoader::Loaded;
            self.score_text_font = Some(font);
        }

        self.step += 1;
        if (self.delay >= 1) {
            self.delay -= 1
        }

        let num_active_moles = self
            .moles
            .iter()
            .filter(|m| m.is_some())
            .count();

        if num_active_moles < 1 {
            self.delay = 0;
            for (i, mole) in self.moles.iter_mut().enumerate() {
                if mole.is_none() {
                    let (x, y) = match self.step % 5 {
                        // 0 => {(200f32, 200f32)},
                        // 1 => {(400f32, 200f32)},
                        // 2 => {(600f32, 200f32)},
                        // 3 => {(300f32, 400f32)},
                        // 4 => {(500f32, 400f32)},
                        _ => {(((random::<usize>() % 600) + 50) as f32, ((random::<usize>() % 500) + 50) as f32)}
                    };
                    *mole = Some(Mole::new(x, y, self.step + self.ttl));
                    break;
                }
            }
        }

        for mole in self.moles.iter_mut() {
            let x = if let Some(m) = mole {m.ttl < self.step} else {false};

            if x {
                mole.take();
            }
        }
    }


    fn event(&mut self, event: &Event, window: &mut Window) {
        use quicksilver::input::MouseButton::Left;
        use quicksilver::input::ButtonState::Pressed;
        if let Event::MouseButton(Left, Pressed) = event {

            if let GameState::Lost(_) = self.state {
                *self = Whac::new();
                return;
            }

            let mouse = window.mouse().pos();
            let mut missed = true;
            for mole in self.moles.iter_mut() {
                let x = if let Some(ref mut mole) = mole {
                    mole.obj.contains(mouse)
                } else { false };

                if x {
                    self.score += 1;
                    mole.take();
                    self.ttl -= 3;
                    self.delay = 4*(self.step%5+1);
                    missed = false;
                }
            }

            if missed {
                self.miss += 1
            }
        }
    }

    fn draw(&mut self, window: &mut Window) {
        if let GameState::Lost(score) = self.state {
            if let Some(ref font) = self.score_text_font {
                let score_text = font.render(&format!("Score: {}", score), 72.0, Color::red());
                window.draw(&Draw::image(&score_text, Vector::new(350, 300)));
                let score_text = font.render("Click to restart", 72.0, Color::red());
                window.draw(&Draw::image(&score_text, Vector::new(350, 400)));
            }
            window.present();
            return;
        }

        window.clear(Color::black());

        for mole in self.moles.iter() {
            if let Some(mole) = mole {
                if let ImageViewer::Loaded(ref image) = mole.image {
                    let scale = Transform::scale(Vector::one()*0.3);
                    window.draw(&Draw::image(image, Vector::new(mole.obj.x, mole.obj.y)).with_transform(scale));
                }
                // window.draw(&Draw::circle(mole.obj).with_color(Color::green()));
            }
        }

        // If the image is loaded draw it
        if let Some(ref font) = self.score_text_font {
            let score_text = font.render(&format!("{}", self.score), 72.0, Color::white());
            window.draw(&Draw::image(&score_text, Vector::new(620, 100)));
        }
        window.present();
    }
}

fn main() {
    run::<Whac>(WindowBuilder::new("Whac-A-Oyvind", 700, 600));
}
