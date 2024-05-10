use std::{
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use clap::Parser;
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, RenderTarget, TextureCreator},
    ttf::Font,
};
use tracing_subscriber::fmt::format::FmtSpan;

const W: u32 = 879;
const H: u32 = 879;

/// Visual dijkstra/A* demo
#[derive(clap::Parser)]
#[command(about)]
struct Args {
    /// Delay between dijkstra iterations (ms)
    #[arg(short, long, default_value_t = 30)]
    delay: u64,

    /// Target FPS
    #[arg(long, default_value_t = 60)]
    fps: u32,

    /// Enable A* instead of dijkstra, using euclidean distance as heuristic
    #[arg(long)]
    enable_astar: bool,
}

fn main() {
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let sdl_context = sdl2::init().unwrap();

    let mut histogram =
        hdrhistogram::Histogram::<u64>::new_with_bounds(1, 15 * 1000 * 1000, 3).unwrap();

    let video = sdl_context.video().unwrap();

    let window = video
        .window("dijkstra", W, H)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut pump = sdl_context.event_pump().unwrap();

    let mut grid = Grid::new(80, 80, (64, 4), (74, 40), args.enable_astar);

    grid.draw_obstacle((4, 16), (18, 4));
    grid.draw_obstacle((24, 40), (80, 0));
    grid.draw_obstacle((15, 8), (80, 8));
    grid.draw_obstacle((0, 30), (30, 30));
    grid.draw_obstacle((4, 70), (70, 20));

    let texture_creator = canvas.texture_creator();

    let ttf = sdl2::ttf::init().unwrap();

    let font = ttf
        .load_font("/usr/share/fonts/liberation/LiberationMono-Regular.ttf", 20)
        .unwrap();

    let mut last_iteration = Instant::now();

    let mut last_frame = Instant::now();

    'main: loop {
        if last_iteration.elapsed() >= Duration::from_millis(args.delay) {
            grid.dijkstra_iteration();

            last_iteration = Instant::now();
        }

        if last_frame.elapsed() >= Duration::from_secs_f64(1.0 / args.fps as f64) {
            canvas.set_draw_color(Color::GRAY);
            canvas.clear();

            grid.draw_to_canvas(&mut canvas, W, H);

            render_text(
                &mut canvas,
                &texture_creator,
                &font,
                &format!("AVG Frame Time: {:.5}", histogram.mean()),
                0,
                0,
            );

            render_text(
                &mut canvas,
                &texture_creator,
                &font,
                &format!("95th Frame Time: {}", histogram.value_at_quantile(0.95)),
                0,
                20,
            );

            render_text(
                &mut canvas,
                &texture_creator,
                &font,
                if args.enable_astar {
                    "RUNNING A*"
                } else {
                    "RUNNING PURE DIJKSTRA"
                },
                0,
                40,
            );

            canvas.present();

            histogram
                .record(last_frame.elapsed().as_micros() as u64)
                .unwrap();

            // TODO: technically we want the time between presents to be 1.0 / args.fps seconds
            last_frame = Instant::now();
        }

        for e in pump.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => continue,
            }
        }
    }
}

fn render_text<T: RenderTarget, C>(
    canvas: &mut Canvas<T>,
    texture_creater: &TextureCreator<C>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
) {
    let surface = font.render(text).solid(Color::BLACK).unwrap();
    let mut rect = surface.rect();
    rect.offset(x, y);

    canvas
        .copy(&surface.as_texture(texture_creater).unwrap(), None, rect)
        .unwrap();
}

#[derive(Clone, Copy, Debug)]
enum CellState {
    Unknown,
    Unvisited,
    Visited { dist: u32 },
    Obstacle,
    OnPath,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct UnvisitedState {
    /// This optionally includes euclidean distance when using A*
    pub dist: u32,
    /// This never includes euclidean distance
    pub actual_dist: u32,
    pub cell: (u32, u32),
}

impl Ord for UnvisitedState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .dist
            .cmp(&self.dist)
            .then(other.actual_dist.cmp(&self.actual_dist))
            .then_with(|| self.cell.cmp(&other.cell))
    }
}

impl PartialOrd for UnvisitedState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Grid {
    enable_astar: bool,

    cells: Vec<Vec<CellState>>,
    unvisited: BinaryHeap<UnvisitedState>,

    start: (u32, u32),
    current: (u32, u32),
    current_dist: u32,
    goal: (u32, u32),
}

impl Grid {
    pub fn new(w: u32, h: u32, start: (u32, u32), goal: (u32, u32), enable_astar: bool) -> Self {
        assert!(start.0 < w && start.1 < h, "start isn't in bounds");
        assert!(goal.0 < w && goal.1 < h, "goal isn't in bounds");

        let mut grid = Self {
            enable_astar,
            cells: vec![vec![CellState::Unknown; h as usize]; w as usize],
            unvisited: BinaryHeap::new(),
            start,
            current: start,
            current_dist: 0,
            goal,
        };

        grid.set_cell(grid.current, CellState::Unvisited);

        grid
    }

    pub fn set_width(&mut self, w: u32) -> &mut Grid {
        let height = self.height();

        self.cells
            .resize_with(w as usize, || vec![CellState::Unknown; height as usize]);
        self
    }

    pub fn width(&self) -> u32 {
        self.cells.len() as u32
    }

    pub fn set_height(&mut self, h: u32) -> &mut Grid {
        self.cells
            .iter_mut()
            .for_each(|v| v.resize_with(h as usize, || CellState::Unknown));
        self
    }

    pub fn height(&self) -> u32 {
        self.cells.get(0).map(Vec::len).unwrap_or(0) as u32
    }

    fn get_cell(&self, cell: (u32, u32)) -> Option<CellState> {
        self.cells
            .get(cell.0 as usize)
            .and_then(|col| col.get(cell.1 as usize))
            .copied()
    }

    fn set_cell(&mut self, cell: (u32, u32), state: CellState) {
        let _ = self
            .cells
            .get_mut(cell.0 as usize)
            .and_then(|col| col.get_mut(cell.1 as usize))
            .map(|cell| {
                *cell = state;
            });
    }

    pub fn draw_obstacle(&mut self, start: (u32, u32), end: (u32, u32)) {
        let m = (start.1 as f64 - end.1 as f64) / (start.0 as f64 - end.0 as f64);

        for x in start.0..end.0 {
            let y = (m * (x as f64 - start.0 as f64)) + start.1 as f64;

            let y = y.round() as u32;

            self.set_cell((x, y), CellState::Obstacle);
        }
    }

    fn get_neighbors(&self, cell: (u32, u32)) -> Vec<(u32, u32)> {
        let mut neighbors = Vec::with_capacity(4);

        // up
        if cell.1 > 0 {
            neighbors.push((cell.0, cell.1 - 1));
        }
        // down
        if cell.1 < self.height() - 1 {
            neighbors.push((cell.0, cell.1 + 1));
        }
        // left
        if cell.0 > 0 {
            neighbors.push((cell.0 - 1, cell.1));
        }
        // right
        if cell.0 < self.width() - 1 {
            neighbors.push((cell.0 + 1, cell.1));
        }

        neighbors
    }

    fn iter(&self) -> impl Iterator<Item = ((u32, u32), CellState)> + '_ {
        self.cells
            .iter()
            .enumerate()
            .map(|(x, col)| {
                col.iter()
                    .enumerate()
                    .map(move |(y, cell)| ((x as u32, y as u32), *cell))
            })
            .flatten()
    }

    fn get_dist(&self, cell: (u32, u32), dist: u32) -> u32 {
        if self.enable_astar {
            let euclid_dist = (((cell.0 as i32 - self.goal.0 as i32).pow(2)
                + (cell.1 as i32 - self.goal.1 as i32).pow(2))
                as f64)
                .sqrt() as u32;

            dist + euclid_dist
        } else {
            dist
        }
    }

    #[tracing::instrument(skip(self))]
    fn dijkstra_iteration(&mut self) {
        if self.current == self.goal {
            return;
        }

        for n in self.get_neighbors(self.current) {
            let state = self.get_cell(n).unwrap();

            match state {
                CellState::Unknown => {
                    let dist = self.current_dist + 1;

                    self.set_cell(n, CellState::Unvisited);

                    self.unvisited.push(UnvisitedState {
                        dist: self.get_dist(n, dist),
                        actual_dist: dist,
                        cell: n,
                    })
                }
                CellState::Unvisited => continue,
                CellState::Visited { dist } => {
                    assert!(dist <= self.current_dist + 1);
                }
                CellState::Obstacle => continue,
                CellState::OnPath => unreachable!(
                    "we shouldn't get here, because cells are only set to onpath on completion"
                ),
            }
        }

        self.set_cell(
            self.current,
            CellState::Visited {
                dist: self.current_dist,
            },
        );

        if let Some(cell) = self.unvisited.pop() {
            self.current = cell.cell;
            self.current_dist = cell.actual_dist;
        } else {
            println!("no possible path");
            return;
        }

        if self.current == self.goal {
            println!("we are done");
            self.color_path();
        }
    }

    fn color_path(&mut self) {
        if self.current != self.goal {
            return;
        }

        let mut cursor = self.goal;

        while cursor != self.start {
            self.set_cell(cursor, CellState::OnPath);

            cursor = self
                .get_neighbors(cursor)
                .into_iter()
                .filter_map(|cell| match self.get_cell(cell).unwrap() {
                    CellState::Visited { dist } => Some((cell, dist)),
                    _ => None,
                })
                .min_by_key(|(_, dist)| *dist)
                .unwrap()
                .0
        }
    }

    pub fn draw_to_canvas<T: RenderTarget>(&self, canvas: &mut Canvas<T>, w: u32, h: u32) {
        let x_spacing = 1;
        let y_spacing = 1;

        let avail_width = w - ((self.width() - 1) * x_spacing);
        let avail_height = h - ((self.height() - 1) * y_spacing);

        let wide = avail_width / self.width();
        let high = avail_height / self.width();

        for (x, col) in self.cells.iter().enumerate() {
            for (y, cell) in col.iter().enumerate() {
                let x = x as u32;
                let y = y as u32;

                let rect = Rect::new(
                    (x * (wide + x_spacing)) as i32,
                    (y * (high + y_spacing)) as i32,
                    wide,
                    high,
                );

                let color = {
                    if (x, y) == self.start {
                        Color::BLUE
                    } else if (x, y) == self.goal {
                        Color::GREEN
                    } else if (x, y) == self.current {
                        Color::CYAN
                    } else {
                        match cell {
                            CellState::Unknown => Color::GREY,
                            CellState::Unvisited { .. } => Color::RED,
                            CellState::Visited { .. } => Color::YELLOW,
                            CellState::Obstacle => Color::WHITE,
                            CellState::OnPath => Color::MAGENTA,
                        }
                    }
                };

                canvas.set_draw_color(color);

                canvas.fill_rect(rect).unwrap();
            }
        }
    }
}
