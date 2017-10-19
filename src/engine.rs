extern crate piston_window;
use std;
use std::thread;
extern crate rand;
use self::rand::{SeedableRng, Rng, Isaac64Rng};

use self::piston_window::*;
const WIDTH : f64 = 500.0;
const HEIGHT : f64 = 400.0;

use network::RemoteInformant;
use network::Diff;

//contains the game's data elements. represents the LOCAL copy
struct World {
    entities : Vec<Entity>,
}
impl World {
    fn new() -> World {
        World {
            entities : Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Entity {
    id : EntityID,
    p : Point,
}

pub type EntityID = u64;

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Point {
    x : f64,
    y : f64,
}
impl Point {
    pub const NULL: Point = Point{x:0.0, y:0.0};
}

pub fn game_loop<RI : RemoteInformant>(mut ri : RI, player_id : u64) {
    let mut window = init_window();
    // let two_secs = std::time::Duration::from_millis(2_000_000);
    let mut w = World::new();


    let mut r = Isaac64Rng::from_seed(&[player_id]);

    let ent_id = (23 << 8) | player_id;
    let mine = Entity {
        id : ent_id,
        p : Point {x : r.gen::<f64>(), y : r.gen::<f64>()},
    };
    ri.update(Diff::Creation(ent_id, mine.p));


    println!("{:?}", &mine);
    w.entities.push(mine);

    //TODO translate between screen pos and world pos
    let mut mouse_at : Option<[f64 ; 2]> = None;

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            window.draw_2d(&e, | _ , graphics| clear([0.0; 4], graphics));
            render_entities(&w, &e, &mut window);
        }
        if let Some(z) = e.mouse_cursor_args() {
            mouse_at = Some(z);
        }
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if let Some(m) = mouse_at {
                    let p = Point {x:m[0]/WIDTH, y:m[1]/HEIGHT};
                    for ent in w.entities.iter_mut(){
                        ent.p = p;
                        ri.update(Diff::Movement(ent.id, p));
                    }
                }
            }
        }

        if let Some(ud) = e.update_args() {
            for diff in ri.drain() {
                println!(">> updating {:?}", &diff);
            }
        }

        /*
        //     calls here to change some game code or another would generate a search etc.
        //     ultimately, any mutation to the game state would ALSO have to inform `ri`
        //     this informant would then simply ensure the Diff gets pushed to the outgoing Vec
        //
        //     behind the scenes, on another thread,
        //     */
    }
}

fn render_entities(w : &World, event : &Event, window : &mut PistonWindow) {
    for e in w.entities.iter() {
        let rad = 10.0;
        window.draw_2d(event, |context, graphics| {
                    ellipse(
                        [0.0, 1.0, 0.0, 1.0],
                        [
                            (e.p.x as f64)*WIDTH - rad,
                            (e.p.y as f64)*HEIGHT - rad,
                            rad*2.0,
                            rad*2.0
                        ],
                        context.transform,
                        graphics
                  );
              }
        );
    }
}

fn init_window() -> PistonWindow {
    let mut window: PistonWindow = WindowSettings::new("Multiplayer", ((WIDTH) as u32, (HEIGHT) as u32))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    let event_settings = EventSettings {
        max_fps: 32,
        ups: 32,
        ups_reset: 2,
        swap_buffers: true,
        bench_mode: false,
        lazy: false,
    };
    window.set_event_settings(event_settings);
    window
}
