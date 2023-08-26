use std::fmt::{Formatter, Display, self};
use std::hash::Hash;
use std::collections::HashSet;
use std::time::Instant;

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Bitfield3D {
    data: Vec<bool>,
    width: isize,
    height: isize,
    depth: isize,
}

impl Hash for Bitfield3D {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
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
        (-1..=self.width).flat_map(move |x| {
            (-1..=self.height).flat_map(move |y| {
                (-1..=self.depth).filter_map(move |z| {
                    if (!self.is_inside(x, y, z) || !self.get_unchecked(x, y, z)) && self.has_set_neighbor(x, y, z) {
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
        x >= 0 && x < self.width && y >= 0 && y < self.height && z >= 0 && z < self.depth
    }
    
    fn index_unchecked(&self, x: isize, y: isize, z: isize) -> usize {
        ((x * self.height  + y) * self.depth + z) as usize
    }

    
    fn rotate_x(&self, buffer: &mut Bitfield3D) {
        buffer.width = self.width;
        buffer.height = self.depth;
        buffer.depth = self.height;

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    let new_y = self.depth - 1 - z;
                    let new_z = y;
                    let index = buffer.index_unchecked(x, new_y, new_z);
                    buffer.data[index] = self.get_unchecked(x, y, z);
                }
            }
        }
    }

    fn rotate_y(&self, buffer: &mut Bitfield3D) {
        buffer.width = self.depth;
        buffer.height = self.height;
        buffer.depth = self.width;

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    let new_x = z;
                    let new_z = self.width - 1 - x;
                    let index = buffer.index_unchecked(new_x, y, new_z);
                    buffer.data[index] = self.get_unchecked(x, y, z);
                }
            }
        }
    }

    fn rotate_z(&self, buffer: &mut Bitfield3D) {
        buffer.width = self.height;
        buffer.height = self.width;
        buffer.depth = self.depth;

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    let new_x = self.height - 1 - y;
                    let new_y = x;
                    let index = buffer.index_unchecked(new_x, new_y, z);
                    buffer.data[index] = self.get_unchecked(x, y, z);
                }
            }
        }
    }
    

    fn create_canonical(&self) -> Bitfield3D {
        let mut result = self.clone();
        let mut rotator = self.clone();
        let mut buffer = self.clone();
        
        // Convert to raw pointers
        let result_ptr = &mut result as *mut Bitfield3D;
        let mut rotator_ptr = &mut rotator as *mut Bitfield3D;
        let mut buffer_ptr = &mut buffer as *mut Bitfield3D;

        // TODO: this makes 32 rotations, but only 24 are needed
        for _x in 0..2 {
            for _y in 0..4 {
                for _z in 0..4 {
                    unsafe {
                        if *rotator_ptr < *result_ptr {
                            *result_ptr = (*rotator_ptr).clone();
                        }
                        (*rotator_ptr).rotate_z(&mut *buffer_ptr);
                    }
                    std::mem::swap(&mut rotator_ptr, &mut buffer_ptr);
                }
                unsafe {
                    (*rotator_ptr).rotate_y(&mut *buffer_ptr);
                }
                std::mem::swap(&mut rotator_ptr, &mut buffer_ptr);
            }
            unsafe {
                (*rotator_ptr).rotate_x(&mut *buffer_ptr);
            }
            std::mem::swap(&mut rotator_ptr, &mut buffer_ptr);
        }
        
        unsafe {
            (*result_ptr).clone()
        }
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

    pub fn generate(&self, new_polycubes: &mut HashSet<Bitfield3D>) {
        for (mut x, mut y, mut z) in self.touching_unset_bits() {
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
            new_polycubes.insert(canonical);
        }
    }

}

impl Display for Bitfield3D {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}\n", std::iter::repeat('ˇ').take(self.width as usize).collect::<String>())?;
        for z in 0..self.depth {
            if z != 0 {
                write!(f, "{}\n", std::iter::repeat('-').take(self.width as usize).collect::<String>())?;
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
        write!(f, "{}\n", std::iter::repeat('^').take(self.width as usize).collect::<String>())
    }
}

fn main() {
    let mut curr_polycubes: HashSet<Bitfield3D> = HashSet::new();
    let mut new_polycubes: HashSet<Bitfield3D> = HashSet::new();
    
    // on first iteration, there is a single block
    curr_polycubes.insert({
        let mut first = Bitfield3D::new(1,1,1);
        first.set_unchecked(0,0,0, true);
        first
    });
    println!("1: 1");
    
    for i in 2.. {
        let start = Instant::now();
        
        for polycube in curr_polycubes.iter() {
            polycube.generate(&mut new_polycubes)
        }
        
        std::mem::swap(&mut curr_polycubes, &mut new_polycubes);
        new_polycubes.clear();
        
        let duration = start.elapsed();
        println!("{}: {}, nano: {} | human: {}.{:03} seconds", i, curr_polycubes.len(), duration.as_nanos(), duration.as_secs(), duration.subsec_nanos() / 1_000_000);
    }
}
