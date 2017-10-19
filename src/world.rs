
use std::collections::HashMap;
use network::Diff;

pub type EntityID = u64;


#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x : f64,
    pub y : f64,
}

impl Point {
    // pub const NULL: Point = Point{x:0.0, y:0.0};
}

#[derive(Debug)]
pub struct Entity {
    p : Point,
}

impl Entity {
    pub fn p(&self) -> &Point {
        &self.p
    }
}

pub struct World {
    entities : HashMap<EntityID, Entity>,
    diffs : Vec<Diff>,
}

impl World {

    pub fn exchange_diffs(&mut self, incoming : Vec<Diff>) -> Vec<Diff> {
        self.apply_incoming_diffs(incoming);
        self.give_outgoing_diffs()
    }

    fn apply_incoming_diffs(&mut self, diffs : Vec<Diff>) {
        for d in diffs {
            println!("world catching incoming diff {:?}", &d);
            match d {
                Diff::Creation(e_id, p) => {self.create_entity(e_id, p, false)},
                Diff::Movement(e_id, p) => {self.move_entity_to(e_id, p, false)},
            }
        }
    }

    fn give_outgoing_diffs(&mut self) -> Vec<Diff> {
        self.diffs.drain(..).collect()
    }

    pub fn new() -> World {
        World {
            entities : HashMap::new(),
            diffs : Vec::new(),
        }
    }

    pub fn create_entity(&mut self, id : EntityID, p : Point, generate_diff : bool){
        self.entities.insert(id, Entity{p : p});
        if generate_diff {
            self.diffs.push(Diff::Creation(id, p));
        }
    }

    pub fn move_entity_to(&mut self, id : EntityID, p : Point, generate_diff : bool) {
        if let Some(ref mut entity) = self.entities.get_mut(&id) {
            entity.p = p;
            if generate_diff {
                self.diffs.push(Diff::Movement(id, p));
            }
        } else {
            panic!("NO ENTITY WITH THAT ID, bub.");
        }
    }

    pub fn entities_iter<'a>(&'a self) -> Box<Iterator<Item=(&EntityID, &Entity)> + 'a> {
        Box::new(
            self.entities.iter()
        )
    }
}
