extern crate piston_window;
use std;
use std::thread;
extern crate rand;
use self::rand::{SeedableRng, Rng, Isaac64Rng};

use world::{World,Point};

use self::piston_window::*;
const WIDTH : f64 = 500.0;
const HEIGHT : f64 = 400.0;

use network::RemoteInformant;
// use network::Diff;

//contains the game's data elements. represents the LOCAL copy


pub fn game_loop<RI : RemoteInformant>(mut ri : RI, player_id : u64) {
    let mut window = init_window();

    //world creates and stores diffs as they pile up
    let mut w = World::new();


    let mut r = Isaac64Rng::from_seed(&[player_id]);

    let protagonist_id = (23 << 8) | player_id;
    w.create_entity(protagonist_id, Point {x : r.gen::<f64>(), y : r.gen::<f64>()}, true);

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
                    w.move_entity_to(protagonist_id, p, true);
                }
            }
        }

        if let Some(_) = e.update_args() {
            let out_from_world = w.exchange_diffs(ri.drain());
            ri.update_all(out_from_world);
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
    for (_, e) in w.entities_iter() {
        let rad = 10.0;
        window.draw_2d(event, |context, graphics| {
                    ellipse(
                        [0.0, 1.0, 0.0, 1.0],
                        [
                            (e.p().x as f64)*WIDTH - rad,
                            (e.p().y as f64)*HEIGHT - rad,
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
