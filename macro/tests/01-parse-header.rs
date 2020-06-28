

use atom_macros::*;
use atom_state::*;

#[derive(Clone)]
struct Pos(f64,f64);

#[state]
fn a_pos() -> Pos {
    Pos(0.,0.)
}

#[state]
fn b_pos() -> Pos {
    Pos(0.,0.)
}

#[reaction]
fn a_b_distance() -> f64 {
    let a = get(a_pos());
    let b = get(b_pos());
    ((a.0-b.0).powi(2) + (a.1-b.1).powi(2)).sqrt()
}

fn main() {
    let a_pos = a_pos(); 
    let b_pos = b_pos();
    let a_b_distance = a_b_distance();

    println!("A is at : {}",  a_pos);
    println!("B is at : {}",  b_pos);
    println!("The distance between them is : {}",  a_b_distance);

    a_pos.update(|s| *s = Pos(4.,5.));
    b_pos.update(|s| *s = Pos(1.,1.));
    println!("moving a to {} and b to {}",  a_pos, b_pos);
    println!("The distance between them is now : {}",  a_b_distance);
    
}


impl std::fmt::Display for Pos
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

// fn use_a_number()-> AtomStateAccess<i32>{
//     atom::<u32>("a_number_state", || 3)
// }

// fn use_add_five()-> AtomStateAccess<i32> {

//     reaction::<u32,_>("add_five", ||{
//         let count = get::<u32>("a_number_state");
//         count + 5
//     })

// }