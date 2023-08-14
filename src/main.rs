// rotation invariance
// hashmap
//

// cc - cube cluster

use std::hash::Hash;
use std::collections::HashSet;

static mut WIDTH: isize = 3;
static mut HEIGHT: isize = 3;
static mut DEPTH: isize = 3;

fn step_dimensions() {
    unsafe {
        WIDTH += 2;
        HEIGHT += 2;
        DEPTH += 2;
    }
}

fn width() -> isize {
    unsafe { WIDTH }
}

fn height() -> isize {
    unsafe { HEIGHT }
}

fn depth() -> isize{
    unsafe { DEPTH }
}


#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Bitfield3D {
    data: Vec<bool>,
}


impl Bitfield3D {
    pub fn new() -> Self {
        Self {
            data: vec![false; (width() * height() * depth()) as usize],
        }
    }

    pub fn set_unchecked(&mut self, x: isize, y: isize, z: isize, value: bool) {
        self.data[((x * height() + y) * depth() + z) as usize] = value;
    }


    pub fn get_unchecked(&self, x: isize, y: isize, z: isize) -> bool {
        self.data[((x * height() + y) * depth() + z) as usize]
    }

    fn touching_unset_bits(&self) -> impl Iterator<Item = (isize, isize, isize)> + '_ {
        (0..width()).flat_map(move |x| {
            (0..height()).flat_map(move |y| {
                (0..depth()).filter_map(move |z| {
                    if !self.get_unchecked(x, y, z) && self.has_set_neighbor(x, y, z) {
                        Some((x, y, z))
                    } else {
                        None
                    }
                })
            })
        })
    }
            
    fn has_set_neighbor(&self, x: isize, y: isize, z: isize) -> bool {
        let neighbors = [
            (x - 1, y, z),
            (x + 1, y, z),
            (x, y - 1, z),
            (x, y + 1, z),
            (x, y, z - 1),
            (x, y, z + 1),
        ];

        neighbors.iter().any(|&(nx, ny, nz)| {
            self.is_inside(nx, ny, nz) && self.get_unchecked(nx, ny, nz)
        })
    }
    
    fn is_inside(&self, x: isize, y: isize, z: isize) -> bool {
        x >= 0 && x < width() && y >= 0 && y < height() && z >= 0 && z < depth()
    }
    
    fn index_unchecked(&self, x: isize, y: isize, z: isize) -> usize {
        ((x * unsafe { HEIGHT } + y) * unsafe { DEPTH } + z) as usize
    }

    
    fn rotate_x(&self) -> Bitfield3D {
        let mut new_data = vec![false; self.data.len()];

        for x in 0..width() {
            for y in 0..height() {
                for z in 0..depth() {
                    if self.get_unchecked(x, y, z) {
                        let new_y = depth() - 1 - z;
                        let new_z = y;
                        let index = self.index_unchecked(x, new_y, new_z);
                        new_data[index] = true;
                    }
                }
            }
        }

        Bitfield3D { data: new_data }
    }

    fn rotate_y(&self) -> Bitfield3D {
        let mut new_data = vec![false; self.data.len()];

        for x in 0..width() {
            for y in 0..height() {
                for z in 0..depth() {
                    if self.get_unchecked(x, y, z) {
                        let new_x = z;
                        let new_z = width() - 1 - x;
                        let index = self.index_unchecked(new_x, y, new_z);
                        new_data[index] = true;
                    }
                }
            }
        }

        Bitfield3D { data: new_data }
    }

    fn rotate_z(&self) -> Bitfield3D {
        let mut new_data = vec![false; self.data.len()];

        for x in 0..width() {
            for y in 0..height() {
                for z in 0..depth() {
                    if self.get_unchecked(x, y, z) {
                        let new_x = height() - 1 - y;
                        let new_y = x;
                        let index = self.index_unchecked(new_x, new_y, z);
                        new_data[index] = true;
                    }
                }
            }
        }

        Bitfield3D { data: new_data }
    }
    
    fn rotations(&self) -> impl Iterator<Item = Bitfield3D> + '_ {
        (0..4).flat_map(move |x| {
            (0..4).flat_map(move |y| {
                (0..4).map(move |z| {
                    self.clone()
                        .rotate_times_x(x)
                        .rotate_times_y(y)
                        .rotate_times_z(z)
                })
            })
        })
    }

    fn rotate_times_x(&self, times: usize) -> Bitfield3D {
        (0..times).fold(self.clone(), |acc, _| acc.rotate_x())
    }

    fn rotate_times_y(&self, times: usize) -> Bitfield3D {
        (0..times).fold(self.clone(), |acc, _| acc.rotate_y())
    }

    fn rotate_times_z(&self, times: usize) -> Bitfield3D {
        (0..times).fold(self.clone(), |acc, _| acc.rotate_z())
    }
    
    fn create_canonical(&self) -> Bitfield3D {
        self.rotations().fold(self.clone(), |acc, rotation| {
            if rotation < acc { 
                rotation 
            } else { 
                acc 
            }
        })
    }
    
    fn grow(&self) -> Bitfield3D {
        // Increase the dimensions by 2 to account for padding on both sides
        let new_width = width() + 2;
        let new_height = height() + 2;
        let new_depth = depth() + 2;

        let mut new_data = vec![false; (new_width * new_height * new_depth) as usize];

        for x in 0..width() {
            for y in 0..height() {
                for z in 0..depth() {
                    if self.get_unchecked(x, y, z) {
                        // Translate to center the original data
                        let new_x = x + 1;
                        let new_y = y + 1;
                        let new_z = z + 1;

                        let index = self.index_unchecked(new_x, new_y, new_z);
                        new_data[index] = true;
                    }
                }
            }
        }

        Bitfield3D { data: new_data }
    }

    pub fn generate(&self, lookup: &mut HashSet<Bitfield3D>) -> Vec<Bitfield3D> {
        let mut next = self.grow();
        self.touching_unset_bits()
            .filter_map(|(x, y, z)| {
                let xx = x + 1;
                let yy = y + 1;
                let zz = z + 1;
                next.set_unchecked(xx, yy, zz, true);

                // Use a scope to limit borrowing duration of self for cloning
                let result = {
                    let canonical = next.create_canonical();
                    if !lookup.contains(&canonical) {
                         // TODO: 2 unnecessary clones, make lookup a hash of strings
                        lookup.insert(canonical.clone());
                        Some(canonical.clone())
                    } else {
                        None
                    }
                };

                // Reset the bit to its original state
                next.set_unchecked(xx, yy, zz, false);
                
                result
            })
            .collect()
    }
    
}

fn main() {
    let mut curr_cc: Vec<Bitfield3D> = vec![];
    let mut next_cc: Vec<Bitfield3D> = vec![];
    let mut lookup: HashSet<Bitfield3D> = HashSet::new();
    
    // on first iteration, there is a single block
    curr_cc.push({
        let mut first = Bitfield3D::new();
        first.set_unchecked(1,1,1, true);
        first
    });
    
    for i in 1.. {
        println!("{}: {}", i, curr_cc.len());
        println!("{:?}", curr_cc);
        if i == 2{
            break;
        }
        
        for cc in curr_cc.iter() {
            next_cc.extend(cc.generate(&mut lookup))
        }
        
        std::mem::swap(&mut curr_cc, &mut next_cc);
        next_cc.clear();
        lookup.clear();
        step_dimensions();
    }
}
