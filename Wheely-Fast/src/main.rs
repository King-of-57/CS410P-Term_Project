extern crate rand;

use ggez;
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics;
use ggez::nalgebra::{Point2, Vector2};
use ggez::timer;
use ggez::{event, Context, GameResult};
use std::env;
use std::path;

use std::time::{Duration, Instant};

use rand::Rng;

//30x30 grid, not sure how big it should be right now this is just for testing. For not it will be 25x50
const GRID_SIZE: (i16, i16) = (25, 50);
//The number of pixels in each cell on the grid, 17x17
const GRID_CELL_SIZE: (i16, i16) = (17, 17);

//The 'x' position for each lane, for each barrier image to appear center in its' respective lane
const LANE_1: f32 = 68.0;
const LANE_2: f32 = 174.0;
const LANE_3: f32 = 280.0;

//An arbitraury number that changes the speed of the background and barriers to make the game more difficult
//4-10 are good starting points
const DIFFICULTY: u32 = 7;

//Distance inbetween each barrier on the screen
//must be negative
//-200 is a good starting point
const BARRIER_DISTANCE: i32 = -200;

//size of the game screen
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

//Determines how quickly the game should update ,dont want the car to move to quickly across the screen so we can determine,
//the distance it moves every frame.
const UPDATES_PER_SECOND: f32 = 8.0;
const MS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

pub fn get_lane() -> f32 {
    let mut rng = rand::thread_rng();
    let x: i16 = rng.gen_range(0,3);
    if x == 0 {
        return LANE_1;
    }
    else if x == 1 {
        return LANE_2;
    }
    else if x == 2 {
        return LANE_3;
    }
    else {
        0.0
    }
}

struct GameImages {
    car_image: graphics::Image,
    font: graphics::Font,
}

impl GameImages {
    fn new(ctx: &mut Context) -> GameResult<GameImages> {
        let car_image = graphics::Image::new(ctx, "/Car.png")?;
        let font = graphics::Font::new(ctx, "/CommodorePixelized.ttf")?;

        Ok(GameImages {
            car_image,
            font,
        })
    }
}

//The directions that the car can go
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    //I want to us Up for now to stop the car justincase it doesn't stop on its own. I have to see how it runs.
    Up,
}

impl Direction {
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            KeyCode::Up => Some(Direction::Up),
            _ => None,
        }
    }
}

//The location of where the car is on the game grid. used for staying in the lane and crossing finish line
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridLocation {
    //Might only need an x value for now since our car only moves left and right.
    x: i16,
    y: i16,
}

impl GridLocation {
    //used to create new grid location easier when updating the position and initalizing the position. 
    //I am not sure if we need the y-axis since we do not move up and down. We probably don't.
    pub fn new(x: i16, y: i16) -> Self {
        GridLocation {x, y}
    }

    //A helper function that takes a grid position and returns a new one after making a move in a spoecific direction.
    //The car can potentially go off screen right now
    //I am also not sure if I need to implement the y-axis since we do not intend for the car to move up and down right now.
    pub fn new_move(pos: GridLocation, dir: Direction) -> Self {
        match dir {
            Direction::Left => GridLocation::new(pos.x - 17, pos.y),
            Direction::Right => GridLocation::new(pos.x + 17, pos.y),
            //The up direction is used to stop the car from moving. I want to look into a way for it to stop moving on key release.GridLocation
            //maybe it will I have to test
            Direction::Up => GridLocation::new(pos.x, pos.y)
        }
    }
}

//This will fill the grid cell with a rectangle that represents our car at a specific location.
impl From<GridLocation> for graphics::Rect {
    fn from(pos: GridLocation) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32, 
            pos.y as i32 * GRID_CELL_SIZE.1 as i32, 
            GRID_CELL_SIZE.0 as i32, 
            GRID_CELL_SIZE.1 as i32
        )
    }
}

//Helps us convert from i16 to GridLocation
impl From<(i16, i16)> for GridLocation {
    fn from(pos: (i16, i16)) -> Self {
        GridLocation {x: pos.0, y: pos.1}
    }
}

//The car object that will be placed on the grid and manuevered.
struct Car {
    car: GridLocation,
    dir: Direction,
    next_dir: Option<Direction>,
}

impl Car {
    pub fn new(pos: GridLocation) -> Self {
        Car {
            //Initial car position, not moving in any direction.
            car: GridLocation::new(pos.x, pos.y),
            dir: Direction::Up,
            next_dir: None
        }
    }

    //need to implement update function that will be used every time that the game needs to be updated.
    fn update(&mut self) {
        //update the direction from the key board input
        if self.next_dir.is_some(){
            self.dir = self.next_dir.unwrap();
            if self.car.x == 0 && self.dir == Direction::Left {
                self.dir = Direction::Up;
            }
            if self.car.x == GRID_SIZE.0 - 1 && self.dir == Direction::Right {
                self.dir = Direction::Up;
            }
            //if I change this to Up does the car stop?
            self.next_dir = None;
        } else {
            //makes it so the car stops after moving 1 cell in the direction of the last key press.
            self.dir = Direction::Up;
        };

        //Give the car a new position and direction
        let new_car_pos = GridLocation::new_move(self.car, self.dir);
        let new_car = GridLocation::new(new_car_pos.x, new_car_pos.y);

        self.car = new_car;

        //must eventually check for collisions here as well. maybe runnning of road too?
    }
    //Draw car
    fn draw(&self, ctx: &mut Context, pic: &mut GameImages) -> GameResult<()> {
        let image = &pic.car_image;
        let pos = self.car;
        let drawparams = graphics::DrawParam::new()
            .dest(Point2::new(pos.x as f32, pos.y as f32));

        graphics::draw(ctx, image, drawparams)
    }
}

struct MainState {
    pics: GameImages,
    start: graphics::Image, // Start button image
    road: graphics::spritebatch::SpriteBatch, // Sprite batch of the road background image
    car: Car,
    barrier: graphics::spritebatch::SpriteBatch, // Sprite batch of the barrier image
    score: u128,
    start_time: u128, // Time when player begins play
    last_update: Instant,
    play: bool, // false means menu, true means gameplay
}

//add levels, score, stop the car from going off screen
impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        //Initializing all the variables of MainState when a new object is created
        let pics = GameImages::new(ctx)?;
        let start_img = graphics::Image::new(ctx, "/Start_Button.png").unwrap();
        let background = graphics::Image::new(ctx, "/Background.png").unwrap();
        let background_batch = graphics::spritebatch::SpriteBatch::new(background);
        //Put car in the middle bottom section of screen or the cars initial location on the screen.
        let car_pos = (((GRID_SIZE.0 * GRID_CELL_SIZE.0) / 2) -28, (GRID_SIZE.1 * GRID_CELL_SIZE.1) -100).into();
        let barrier_img = graphics::Image::new(ctx, "/Barrier.png").unwrap();
        let blockage = graphics::spritebatch::SpriteBatch::new(barrier_img);

        let mut s = MainState {
            pics,
            start: start_img,
            road: background_batch,
            car: Car::new(car_pos),
            barrier: blockage,
            score: 0,
            start_time: 0,
            last_update: Instant::now(),
            play: false,
        };

        //This generates 450 barriers that are each 'BARRIER_DISTANCE' away from each other and loads them
        //into the barrier variable of MainState
        for x in 0..450 {
            let i = get_lane();
            //Generate a barrier every 'BARRIER_DISTANCE' pixels apart, where x = the nth barrier
            let j = graphics::DrawParam::new()
                .dest(Point2::new(i, (x * BARRIER_DISTANCE) as f32))
                .scale(Vector2::new(1.0, 1.0,))
                .rotation(0.0);
            s.barrier.add(j);
        }

        Ok(s)
    }
}

//implements the EventHandler for the GameState
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.play {
            //check to see if enough time has passed so we can update again.
            if Instant::now() - self.last_update >= Duration::from_millis(MS_PER_UPDATE) {
                self.car.update();
                self.last_update = Instant::now();

                //update score
                let time = timer::time_since_start(_ctx).as_millis();
                self.score = (time - self.start_time) / 64;
            }
        }
        Ok(())
    }

    //render the game
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let pics = &mut self.pics;
        //clear screen, can make the screens background a specific color here.
        graphics::clear(ctx, graphics::BLACK.into());

        //current time in miliseconds
        let time = (timer::duration_to_f64(timer::time_since_start(ctx)) * 1000.0) as u32;
    
        //generates 150 road images and stiches them together to create a scrolling background effect so the user
        //thinks the car is driving on a road
        for x in 0..150 {
            let p = graphics::DrawParam::new()
                //the y postion was calculated to get the images to stich together seemlessly
                .dest(Point2::new(0.0, ((x * -450) + 600) as f32))
                .scale(Vector2::new(1.0, 1.0,))
                .rotation(0.0);
            self.road.add(p);
        }
        let param = graphics::DrawParam::new()
            .dest(Point2::new(0.0, (time / DIFFICULTY) as f32))
            .scale(Vector2::new(1.0, 1.0,))
            .rotation(0.0)
            .offset(Point2::new(0.0, 0.0));

        graphics::draw(ctx, &self.road, param)?;
        self.road.clear();

        //if the game isnt started display the start button
        if !self.play {
            let start_dest = Point2::new(SCREEN_SIZE.0 / 4.0, SCREEN_SIZE.1 / 2.0);
            graphics::draw(ctx, &self.start, graphics::DrawParam::default().dest(start_dest))?;
        }
        //else start generating barriers on the screen
        else {
            let offset_distance = self.start_time as u32 / DIFFICULTY;
            let param2 = graphics::DrawParam::new()
                .dest(Point2::new(0.0, ((time / DIFFICULTY) - offset_distance) as f32))
                .scale(Vector2::new(1.0, 1.0,))
                .rotation(0.0)
                .offset(Point2::new(0.0, 0.0));

            graphics::draw(ctx, &self.barrier, param2)?;
        }

        //draw car
        self.car.draw(ctx, pics)?;

        //draw score
        let score_dest = Point2::new(SCREEN_SIZE.0 / 8.0, 16.0);
        let score_str = format!("Score: {}", self.score);
        let score_display = graphics::Text::new((score_str, pics.font, 30.0));
        graphics::draw(ctx, &score_display, (score_dest, 0.0, graphics::Color::new(1.0, 0.0, 0.0, 1.0)))?; //red

        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context, 
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
        ) {
            if let Some(dir) = Direction::from_keycode(keycode) {
                // ensures direction is not changed  unless in play mode
                // this way the car stays in place even if arrow key is pressed before return
                if self.play {
                    //just make the direction for the next left or right input the same as
                    self.car.next_dir = Some(dir);
                }
            } else { 
                match keycode {
                    KeyCode::Return => {
                        // press return to start game
                        if !self.play {
                            self.play = true;
                            self.start_time = timer::time_since_start(_ctx).as_millis();
                        }
                    }
                    KeyCode::Escape => {
                        // quit app by pressing escape key
                        event::quit(_ctx);
                    }
                    _ => (), // do nothing
                }
            }
        }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Racing Game", "Ryan Campbell")
        .window_setup(ggez::conf::WindowSetup::default().title("Racing"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .add_resource_path(resource_dir)
        .build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, events_loop, state)
}
