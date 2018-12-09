extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use rand::Rng;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

mod enums;


trait RenderAndUpdate {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs);
    fn update(&mut self);
    fn get_pos(&self) -> Vec<i32>;
    fn get_is_food(&self) -> bool;
    fn change_color(&mut self);
}

struct Game {
    score: i32,
    epileptic_mode_on: bool,
    gl: GlGraphics,
    snake: Snake,
    food_and_barriers: Vec<Box<RenderAndUpdate>>,
    background_color: [f32; 4],
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics;

        let color = self.background_color;

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(color, gl);
        });

        self.snake.render(&mut self.gl, args);

        for item in  &self.food_and_barriers {
            item.render(&mut self.gl, args);
        }
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        if self.epileptic_mode_on {
            if self.background_color[1] == 1.0 {
                self.background_color = [0.0, 0.0, 1.0, 1.0];
                self.snake.color = [0.0, 1.0, 0.0, 1.0];
            } else {
                self.background_color = [0.0, 1.0, 0.0, 1.0];
                self.snake.color = [1.0, 0.0, 0.0, 1.0];
            }
            for item in &mut self.food_and_barriers {
                item.change_color();
            }
        }
        if self.snake.update(&self.food_and_barriers) {
            if self.snake.food_is_eaten {
                self.snake.food_is_eaten = false;
                self.score += 1;
                for item in &mut self.food_and_barriers {
                    item.update();
                }
            }
            return true
        } else { 
            return false
        }
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match btn {
            &Button::Keyboard(Key::Up)
                if self.snake.direction != enums::Directions::Down => enums::Directions::Up,
            &Button::Keyboard(Key::Down)
                if self.snake.direction != enums::Directions::Up => enums::Directions::Down,
            &Button::Keyboard(Key::Right)
                if self.snake.direction != enums::Directions::Left => enums::Directions::Right,
            &Button::Keyboard(Key::Left)
                if self.snake.direction != enums::Directions::Right => enums::Directions::Left,
            _ => last_direction
        };

        if btn == &Button::Keyboard(Key::Tab) {
            self.epileptic_mode_on = !self.epileptic_mode_on;
        }
    }
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    direction: enums::Directions,
    food_is_eaten: bool,
    color: [f32; 4],
}

impl Snake {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        let squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|&(x, y)| {
                graphics::rectangle::square(
                    (x * 20) as f64, 
                    (y * 20) as f64, 
                    20.0)
            })
            .collect(); 

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares.into_iter()
                .for_each(|square| graphics::rectangle(self.color, square, transform, gl));
        })
    }

    fn update(&mut self, food_and_barriers: &Vec<Box<RenderAndUpdate>>) -> bool {
        let mut new_head = (self.body.front().expect("Snake has no body")).clone();

        match self.direction {
            enums::Directions::Right => new_head.0 += 1,
            enums::Directions::Left => new_head.0 -= 1,
            enums::Directions::Up => new_head.1 -= 1,
            enums::Directions::Down => new_head.1 += 1,
        }

        if new_head.0 < 0 || new_head.0 > 34 || new_head.1 < 0 || new_head.1 > 19 {
            return false
        }

        for item in food_and_barriers {
            let pos = item.get_pos();
            if pos[0] == new_head.0 && pos[1] == new_head.1 {
                if item.get_is_food() {
                    self.body.push_front(new_head);
                    self.food_is_eaten = true;
                    return true
                }
                return false
            }
        }

        for (x, y) in self.body.iter() {
            if x == &new_head.0 && y == &new_head.1 {
                return false
            }
        }

        self.body.push_front(new_head);
        self.body.pop_back().unwrap();

        true
    }
}

struct Food {
    is_food: bool,
    pos_x: i32,
    pos_y: i32,
    color: [f32; 4],
}

impl RenderAndUpdate for Food{
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics; 

        let square = graphics::rectangle::square(
            (self.pos_x * 20) as f64, 
            (self.pos_y * 20) as f64, 
            20.0);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(self.color, square, transform, gl);
        })
    }

    fn update(&mut self) {
        let new_pos_x = rand::thread_rng().gen_range(0, 34);
        let new_pos_y = rand::thread_rng().gen_range(0, 19);

        self.pos_x = new_pos_x;
        self.pos_y = new_pos_y;
    }

    fn get_pos(&self) -> Vec<i32> {
        vec![self.pos_x, self.pos_y]
    }

    fn get_is_food(&self) -> bool {
        self.is_food
    }

    fn change_color(&mut self) {
        if self.color[0] == 1.0 {
            self.color = [0.0, 0.0, 0.0, 1.0];
        } else {
            self.color = [1.0, 1.0, 1.0, 1.0];
        }
    }
}

struct Barrier {
    is_food: bool,
    pos_x: i32,
    pos_y: i32,
    color: [f32; 4],
}

impl RenderAndUpdate for Barrier{
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics; 

        let square = graphics::rectangle::square(
            (self.pos_x * 20) as f64, 
            (self.pos_y * 20) as f64, 
            20.0);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(self.color, square, transform, gl);
        })
    }

    fn update(&mut self) {}

    fn get_pos(&self) -> Vec<i32> {
        vec![self.pos_x, self.pos_y]
    }

    fn get_is_food(&self) -> bool {
        self.is_food
    }

    fn change_color(&mut self) {
        if self.color[0] == 1.0 {
            self.color = [0.0, 0.0, 0.0, 1.0];
        } else {
            self.color = [1.0, 1.0, 1.0, 1.0];
        }
    }
}

fn main() {
    let open_gl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new(
        "Hladkiy Ivan.   Rust.   Snake Game.", 
        [700, 400]
    ).opengl(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game {
        score: 0,
        epileptic_mode_on: false,
        gl: GlGraphics::new(open_gl),
        food_and_barriers: vec![
            Box::new(Food {
                is_food: true,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [1.0, 1.0, 1.0, 1.0]
            }),
            Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            }),
            Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            }),
            Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            }),Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            }),
            Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            }),
            Box::new(Barrier{
                is_food: false,
                pos_x: rand::thread_rng().gen_range(0, 34),
                pos_y: rand::thread_rng().gen_range(0, 19),
                color: [0.0, 0.0, 0.0, 1.0]
            })
        ],
        snake: Snake { 
            body: LinkedList::from_iter((vec![(0, 0), (0,1)]).into_iter()),
            direction: enums::Directions::Right,
            food_is_eaten: false,
            color: [1.0, 0.0, 0.0, 1.0]
        },
        background_color: [0.0, 1.0, 0.0, 1.0]
    };

    let mut events = Events::new(EventSettings::new()).ups(8);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }

    println!("Nice try! Your score: {}", game.score);
}
