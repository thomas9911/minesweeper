use rand::prelude::*;

#[derive(Debug)]
pub struct Tiles {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Tiles {
    pub fn new(width: usize, height: usize) -> Tiles {
        let mut tiles: Vec<Vec<Tile>> = vec![];
        for i in 0..width {
            let mut row_of_tiles: Vec<Tile> = vec![];
            for j in 0..height {
                row_of_tiles.push(Tile::new(i, j))
            }
            tiles.push(row_of_tiles)
        }
        Tiles {
            width: width,
            height: height,
            tiles: tiles,
        }
    }

    fn is_legit(&mut self, x: usize, y: usize) -> bool {
        (x < self.width)
            & (y < self.height)
            & self.tiles.get(x).is_some()
            & self.tiles.get(y).is_some()
    }

    pub fn set_mines(&mut self, amount_of_mines: u32) {
        if amount_of_mines > (self.width * self.height) as u32 {
            panic!("what are you doing?!");
        }
        let mut thread_rng = thread_rng();
        let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
        let mut mines_placed: u32 = 0;

        // let seed = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16]; // byte array
        // let mut rng = SmallRng::from_seed(seed);
        while mines_placed < amount_of_mines {
            let x_choose: usize = rng.gen_range(0, self.width);
            let y_choose: usize = rng.gen_range(0, self.height);
            // println!("{} {}", x_choose, y_choose);
            // println!("{:?}", self.tiles[x_choose][y_choose]);

            if !self.tiles[x_choose][y_choose].mine {
                self.tiles[x_choose][y_choose].set_as_mine();
                mines_placed += 1;
            }
        }
        self.calculate_mines_around();
    }

    pub fn reveal_around(&mut self, x: u32, y: u32) {
        self.tiles[x as usize][y as usize].revealed = true;
        if (self.tiles[x as usize][y as usize].mines_around == 0)
            & (self.tiles[x as usize][y as usize].flow_revealed == false)
        {
            self.tiles[x as usize][y as usize].flow_revealed = true;
            for p in -1..2 {
                for q in -1..2 {
                    if !((p == 0) & (q == 0)) {
                        let a = (x as i32 + p) as usize;
                        let b = (y as i32 + q) as usize;
                        if self.is_legit(a, b) {
                            self.reveal_around(a as u32, b as u32)
                        }
                    }
                }
            }
        }
    }

    fn calculate_mines_around(&mut self) {
        for i in 0..self.width {
            for j in 0..self.height {
                let mut count: u32 = 0;
                for p in -1..2 {
                    for q in -1..2 {
                        if !((p == 0) & (q == 0)) {
                            let a = (i as i32 + p) as usize;
                            let b = (j as i32 + q) as usize;
                            if (a < self.width)
                                & (b < self.height)
                                & self.tiles.get(a).is_some()
                                & self.tiles.get(b).is_some()
                            {
                                if self.tiles[a][b].mine {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                self.tiles[i][j].set_mines_around(count);
                // println!("{} {}: {}", i, j, count);
            }
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub mine: bool,
    pub flagged: bool,
    pub mines_around: u32,
    pub revealed: bool,
    flow_revealed: bool,
}

impl Tile {
    pub fn new(x: usize, y: usize) -> Tile {
        Tile {
            x: x,
            y: y,
            mine: false,
            flagged: false,
            mines_around: 0,
            revealed: false,
            flow_revealed: false,
        }
    }

    pub fn set_as_mine(&mut self) {
        self.mine = true;
    }

    pub fn set_mines_around(&mut self, mines_around: u32) {
        self.mines_around = mines_around;
    }

    pub fn reveal(&mut self) {
        self.revealed = true;
    }

    pub fn flip_flag(&mut self) {
        match self.flagged {
            true => self.flagged = false,
            false => self.flagged = true,
        };
    }
}
