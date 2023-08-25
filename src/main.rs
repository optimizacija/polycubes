// rotation invariance
// hashmap
//

// cc - cube cluster

use std::fmt::{Formatter, Display, self};
use std::hash::Hash;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

use log::{debug, info};

#[derive(Clone, Hash, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Bitfield3D {
    data: Vec<bool>,
    width: isize,
    height: isize,
    depth: isize,
}

impl Bitfield3D {
    pub fn new(width: isize, height: isize, depth: isize) -> Self {
        Self {
            data: vec![false; (width * height * depth) as usize],
            width,
            height,
            depth,
        }
    }

    pub fn set_unchecked(&mut self, x: isize, y: isize, z: isize, value: bool) {
        self.data[((x * self.height + y) * self.depth + z) as usize] = value;
    }


    pub fn get_unchecked(&self, x: isize, y: isize, z: isize) -> bool {
        self.data[((x * self.height + y) * self.depth + z) as usize]
    }

    fn touching_unset_bits(&self) -> impl Iterator<Item = (isize, isize, isize)> + '_ {
        enum State {
            Explore(isize, isize, isize),
            Yield(isize, isize, isize),
        }
        
        struct MyIterator<'a> {
            states: VecDeque<State>,
            visited: HashSet<(isize, isize, isize)>,
            bitfield: &'a Bitfield3D,
        }

        impl<'a> MyIterator<'a> {
            pub fn new(initial: (isize, isize, isize), bitfield: &'a Bitfield3D) -> Self {
                let mut visited = HashSet::new();
                visited.insert(initial);
                
                let mut states = VecDeque::new();
                states.push_back(State::Explore(initial.0, initial.1, initial.2));
                
                Self {
                    states,
                    visited,
                    bitfield,
                }
            }
        }
        

        impl<'a> Iterator for MyIterator<'a> {
            type Item = (isize, isize, isize);

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(state) = self.states.pop_front() {
                    match state {
                        State::Explore(x, y, z) => {
                            let neighbors = [
                                (x - 1, y, z),
                                (x + 1, y, z),
                                (x, y - 1, z),
                                (x, y + 1, z),
                                (x, y, z - 1),
                                (x, y, z + 1),
                            ];

                            for &(nx, ny, nz) in neighbors.iter() {
                                if !self.bitfield.is_inside(nx, ny, nz) || !self.bitfield.get_unchecked(nx, ny, nz) {
                                    self.states.push_back(State::Yield(nx, ny, nz));
                                } else if !self.visited.contains(&(nx, ny, nz)) {
                                    self.visited.insert((nx, ny, nz));
                                    self.states.push_back(State::Explore(nx, ny, nz));
                                }
                            }
                        }
                        State::Yield(x, y, z) => {
                            return Some((x, y, z));
                        }
                    }
                }
                None
            }
        }
        
        let (x, y, z) = self.find_first_set_bit_unchecked();
        MyIterator::new((x, y, z), &self)
    }
            
    fn is_inside(&self, x: isize, y: isize, z: isize) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height && z >= 0 && z < self.depth
    }
    
    fn index_unchecked(&self, x: isize, y: isize, z: isize) -> usize {
        ((x * self.height  + y) * self.depth + z) as usize
    }

    
    fn rotate_x(&self) -> Bitfield3D {
        let mut result = Bitfield3D::new(self.width, self.depth, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    if self.get_unchecked(x, y, z) {
                        let new_y = self.depth - 1 - z;
                        let new_z = y;
                        let index = result.index_unchecked(x, new_y, new_z);
                        result.data[index] = true;
                    }
                }
            }
        }

        result
    }

    fn rotate_y(&self) -> Bitfield3D {
        let mut result = Bitfield3D::new(self.depth, self.height, self.width);

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    if self.get_unchecked(x, y, z) {
                        let new_x = z;
                        let new_z = self.width - 1 - x;
                        let index = result.index_unchecked(new_x, y, new_z);
                        result.data[index] = true;
                    }
                }
            }
        }

        result
    }

    fn rotate_z(&self) -> Bitfield3D {
        let mut result = Bitfield3D::new(self.height, self.width, self.depth);

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    if self.get_unchecked(x, y, z) {
                        let new_x = self.height - 1 - y;
                        let new_y = x;
                        let index = result.index_unchecked(new_x, new_y, z);
                        result.data[index] = true;
                    }
                }
            }
        }

        result
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
    
    fn grow_to_fit(&self, x: isize, y: isize, z: isize) -> Bitfield3D {
        // Increase the dimensions by 2 to account for padding on both sides
        let offset_x = {
            if x < 0 {
                x.abs()
            } else {
                0
            }
        };
        let offset_y = {
            if y < 0 {
                y.abs()
            } else {
                0
            }
        };
        let offset_z = {
            if z < 0 {
                z.abs()
            } else {
                0
            }
        };
        let new_width = {
            if x >= self.width {
                x + 1
            } else {
                self.width + offset_x
            }
        };
        let new_height = {
            if y >= self.height {
                y + 1
            } else {
                self.height + offset_y
            }
        };
        let new_depth = {
            if z >= self.depth {
                z + 1
            } else {
                self.depth + offset_z
            }
        };

        let mut new_data = vec![false; (new_width * new_height * new_depth) as usize];

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    if self.get_unchecked(x, y, z) {
                        // takes offsets into account
                        let index = (((x + offset_x) * new_height  + (y + offset_y)) * new_depth + (z + offset_z)) as usize;
                        
                        // doesn't take into account offsets
                        // let index = ((x * new_height  + y) * new_depth + z) as usize;
                        new_data[index] = true;
                    }
                }
            }
        }
        
        // debug!("len: {}, width: {}, height: {}, depth: {}", new_data.len(), new_width, new_height, new_depth);

        Bitfield3D { 
            data: new_data,
            width: new_width,
            height: new_height,
            depth: new_depth,
        }
    }

    pub fn generate(&self, lookup: &mut HashSet<Bitfield3D>) -> Vec<Bitfield3D> {
        self.touching_unset_bits()
            .filter_map(|(mut x, mut y, mut z)| {
                let mut next = {
                    if !self.is_inside(x, y, z) {
                        let result = self.grow_to_fit(x, y, z);
                        
                        if x < 0 {
                            x = 0;
                        }
                        if y < 0 {
                            y = 0;
                        }
                        if z < 0 {
                            z = 0;
                        }
                        result
                    } else {
                        self.clone()
                    }
                };
                next.set_unchecked(x, y, z, true);

                let canonical = next.create_canonical();
                if !lookup.contains(&canonical) {
                    // TODO: 2 unnecessary clones, make lookup a hash of strings
                    lookup.insert(canonical.clone());
                    Some(canonical.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn find_first_set_bit_unchecked(&self) -> (isize, isize, isize) {
        // basic for loop
        /*
        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    if self.get_unchecked(x, y, z) {
                        return (x, y, z)
                    }
                }
            }
        }
        (0, 0, 0)
        */
        
        // index manipulation
        let first = self.data.iter().position(|&x| x).unwrap();

        let h = self.height as usize;
        let d = self.depth as usize;

        let x = first / (h * d);
        let y = (first % (h * d)) / d;
        let z = first % d;

        (
            x as isize,
            y as isize,
            z as isize,
        )
    }
}

impl Display for Bitfield3D {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ˇˇˇ\n")?;
        for z in 0..self.depth {
            if z != 0 {
                write!(f, "---\n")?;
            }
            for y in 0..self.height {
                for x in 0..self.width {
                    let bit = if self.get_unchecked(x as isize, y as isize, z as isize) {
                        '1'
                    } else {
                        '0'
                    };
                    write!(f, "{}", bit)?;
                }
                write!(f, "\n")?;
            }
        }
        write!(f, "^^^\n")
    }
}

fn main() {
    env_logger::init();
    
    let mut curr_cc: Vec<Bitfield3D> = vec![];
    let mut next_cc: Vec<Bitfield3D> = vec![];
    let mut lookup: HashSet<Bitfield3D> = HashSet::new();
    
    // on first iteration, there is a single block
    curr_cc.push({
        let mut first = Bitfield3D::new(1,1,1);
        first.set_unchecked(0,0,0, true);
        first
    });
    
    for i in 1.. {
        let start = Instant::now();
        
        info!("{}: {}", i, curr_cc.len());
        /*
        for b in curr_cc.iter() {
            debug!("curr_cc\n{}", b);
        }
        */
        
        for cc in curr_cc.iter() {
            next_cc.extend(cc.generate(&mut lookup))
        }
        
        std::mem::swap(&mut curr_cc, &mut next_cc);
        next_cc.clear();
        lookup.clear();
        
        let duration = start.elapsed();
        info!("{}.{:03} seconds", duration.as_secs(), duration.subsec_nanos() / 1_000_000);
    }
}
