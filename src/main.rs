use sdl2::{pixels::Color, rect::Rect, render::{Canvas, RenderTarget}};

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
    
    'main: loop {
        canvas.set_draw_color(Color::RGB(127, 127, 127));
        canvas.clear();

        grid.draw_to_canvas(&mut canvas, W, H);

        canvas.present();

        for e in pump.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => continue,
            }
        }
    }
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

