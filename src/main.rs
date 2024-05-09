use std::time::Instant;

use sdl2::{pixels::Color, rect::Rect, render::{Canvas, RenderTarget, TextureCreator}, ttf::Font};

const W: u32 = 819;
const H: u32 = 819;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let video = sdl_context.video().unwrap();

    let window = video.window("dijkstra", W, H).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut pump = sdl_context.event_pump().unwrap();

    let mut grid = Grid::default();

    grid
        .set_width(20)
        .set_height(20);

    let texture_creator = canvas.texture_creator();
    
    let ttf = sdl2::ttf::init().unwrap();

    let font = ttf.load_font("/usr/share/fonts/liberation/LiberationMono-Regular.ttf", 20).unwrap();

    // first set frame_time to it's value, if we were at 60 fps
    let mut frame_time = 0.016;
    
    'main: loop {
        let start_time = Instant::now();

        canvas.set_draw_color(Color::RGB(127, 127, 127));
        canvas.clear();

        grid.draw_to_canvas(&mut canvas, W, H);

        render_text(&mut canvas, &texture_creator, &font, &format!("Frame Time: {:.5}", frame_time), 0, 0);

        render_text(&mut canvas, &texture_creator, &font, &format!("{:.1}FPS", 1.0 / frame_time), 0, 20);

        canvas.present();

        frame_time = start_time.elapsed().as_secs_f64();

        for e in pump.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => continue,
            }
        }
    }
}

fn render_text<T: RenderTarget, C>(canvas: &mut Canvas<T>, texture_creater: &TextureCreator<C>, font: &Font, text: &str, x: i32, y: i32) {
    let surface = font.render(text).solid(Color::WHITE).unwrap();
    let mut rect = surface.rect();
    rect.offset(x, y);

    canvas.copy(&surface.as_texture(texture_creater).unwrap(), None, rect).unwrap();
}

#[derive(Default)]
pub struct Grid {
    width: u32,
    height: u32,
}

impl Grid {
    pub fn set_width(&mut self, w: u32) -> &mut Grid {
        self.width = w;
        self
    }

    pub fn set_height(&mut self, h: u32) -> &mut Grid {
        self.height = h;
        self
    }

    pub fn draw_to_canvas<T: RenderTarget>(&self, canvas: &mut Canvas<T>, w: u32, h: u32) {
        let x_spacing = 1;
        let y_spacing = 1;

        let avail_width = w - ((self.width - 1) * x_spacing);
        let avail_height = w - ((self.height - 1) * y_spacing);

        let wide = avail_width / self.width;
        let high = avail_height / self.width;

        println!("drawing grid size: {} by {}", w, h);
        println!("drawing grid cells: {} by {}", wide, high);

        canvas.set_draw_color(Color::RGB(63, 63, 63));

        let rect_x = wide;
        let rect_y = high;

        for x in 0..self.width {
            for y in 0..self.height {
                let rect = Rect::new((x * (wide + x_spacing)) as i32, (y * (high + y_spacing)) as i32, rect_x, rect_y);

                println!("drawing {rect:?}");

                canvas.fill_rect(rect).unwrap();
            }
        }
    }
}

