use std::collections::HashMap;
use crate::types::{BitBoard, BitCoord};

pub struct MagicBitBoards {
    rooks: Vec<Magic>,
    bishops: Vec<Magic>,

    kings: Vec<BitBoard>,
    knights: Vec<BitBoard>,
}

impl MagicBitBoards {
    pub fn default() -> MagicBitBoards {
        Self::generate(generated::ROOK_MAGIC, generated::BISHOP_MAGIC)
    }

    pub fn generate(rook_magic: [u64; 64], bishop_magic: [u64; 64]) -> MagicBitBoards {
        let mut rooks: Vec<Magic> = Vec::with_capacity(64);
        let mut bishops: Vec<Magic> = Vec::with_capacity(64);
        let mut kings: Vec<BitBoard> = Vec::with_capacity(64);
        let mut knights: Vec<BitBoard> = Vec::with_capacity(64);
        for c in 0..64 {
            let coord = BitCoord(1 << c);
            let rook_moves_map = generate_moves(coord, rook_mask(coord), rook_moves);
            let bishop_moves_map = generate_moves(coord, bishop_mask(coord), bishop_moves);

            rooks.push(Magic::generate(
                    rook_magic[coord.0.trailing_zeros() as usize],
                    rook_mask(coord),
                    &rook_moves_map,
            ).unwrap());

            bishops.push(Magic::generate(
                    bishop_magic[coord.0.trailing_zeros() as usize],
                    bishop_mask(coord),
                    &bishop_moves_map,
            ).unwrap());

            kings.push(king_moves(coord));
            knights.push(knight_moves(coord));
        }

        MagicBitBoards{rooks, bishops, kings, knights}
    }

    pub fn rook(&self, coord: BitCoord) -> &Magic {
        &self.rooks[coord.0.trailing_zeros() as usize]
    }

    pub fn bishop(&self, coord: BitCoord) -> &Magic {
        &self.bishops[coord.0.trailing_zeros() as usize]
    }

    pub fn king(&self, coord: BitCoord) -> BitBoard {
        self.kings[coord.0.trailing_zeros() as usize]
    }

    pub fn knight(&self, coord: BitCoord) -> BitBoard {
        self.knights[coord.0.trailing_zeros() as usize]
    }
}

pub struct Magic {
    table: Vec<BitBoard>,
    mask: BitBoard,
    magic: u64,
    shift: u32,
}

impl Magic {
    pub fn dummy() -> Magic {
        Magic{
            table: vec![],
            mask: BitBoard::EMPTY,
            magic: 0,
            shift: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.table.len()
    }

    pub fn lookup(&self, bb: BitBoard) -> BitBoard {
        return self.table[Magic::index(bb, self.mask, self.magic, self.shift)];
    }

    pub fn generate(
        magic: u64,
        mask: BitBoard,
        all_moves: &HashMap<BitBoard, BitBoard>,
    ) -> Option<Magic> {
        // 2^10 = 1024 possible masked occupancies.
        // Perfect hashing would fit in 256 cells.
        let occupancies = boards_for_mask(mask);

        for size in 8..=16 {
            let shift = 64 - size;
            let mut table = vec![BitBoard::EMPTY; 1 << size];
            let mut filled = vec![false; 1 << size];

            let mut success = true;
            for o in occupancies.iter() {
                let index = Magic::index(*o, mask, magic, shift);
                let moves = all_moves.get(o).expect("Missing some moves");
                if !filled[index] {
                    table[index] = *moves;
                    filled[index] = true;
                    continue;
                }

                if table[index] != *moves {
                    success = false;
                    break;
                }
            }

            if success {
                return Some(Magic{table, mask, magic, shift});
            }
        }

        None
    }

    fn index(bb: BitBoard, mask: BitBoard, magic: u64, shift: u32) -> usize {
        ((bb.0 & mask.0).wrapping_mul(magic) >> shift) as usize
    }
}

pub fn generate_moves(
    coord: BitCoord, 
    mask: BitBoard,
    movegen: fn (BitCoord, BitBoard) -> BitBoard,
) -> HashMap<BitBoard, BitBoard> {
    let mut moves: HashMap<BitBoard, BitBoard> = HashMap::new();
    for o in boards_for_mask(mask) {
        moves.insert(o, movegen(coord, o));
    }
    moves
}

pub fn rook_mask(coord: BitCoord) -> BitBoard {
    line_mask(coord, 1) | line_mask(coord, -1) | line_mask(coord, 8) | line_mask(coord, -8)
}

pub fn rook_moves(start: BitCoord, occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [1, 8, -1, -8].iter() {
        for coord in Line::new(start, *dir) {
            moves = moves | coord;
            if occupancy & coord != BitBoard::EMPTY {
                break;
            }
        }
    }

    moves
}

pub fn bishop_mask(coord: BitCoord) -> BitBoard {
    line_mask(coord, 9) | line_mask(coord, -9) | line_mask(coord, 7) | line_mask(coord, -7)
}

pub fn bishop_moves(start: BitCoord, occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [7, 9, -7, -9].iter() {
        for coord in Line::new(start, *dir) {
            moves = moves | coord;
            if occupancy & coord != BitBoard::EMPTY {
                break;
            }
        }
    }

    moves
}

pub fn king_moves(start: BitCoord) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [1, -1, 8, -8, 9, -9, 7, -7].iter() {
        match Line::new(start, *dir).next() {
            Some(coord) => {
                moves = moves | coord;
            },
            None => (),
        }
    }

    moves
}

pub fn knight_moves(start: BitCoord) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [17, -17, 15, -15, 10, -10, 6, -6].iter() {
        match Line::new(start, *dir).next() {
            Some(coord) => {
                moves = moves | coord;
            },
            None => (),
        }
    }

    moves
}

struct Line {
    coord: u64,
    shift: i32,
}

impl Line {
    fn new(start: BitCoord, shift: i32) -> Line {
        Line{
            coord: start.0,
            shift,
        }
    }
}

impl Iterator for Line {
    type Item = BitCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let prev = self.coord;

        if self.shift > 0 {
            self.coord <<= self.shift as u32;
        } else {
            self.coord >>= (-self.shift) as u32;
        }

        // Check for board wrapping.
        // Maximum possible taxicab distance is 3 (for knight move).
        let dx = diff(prev.trailing_zeros() % 8, self.coord.trailing_zeros() % 8);
        let dy = diff(prev.trailing_zeros() / 8, self.coord.trailing_zeros() / 8);
        if dx + dy > 3 {
            return None;
        }

        if self.coord == 0 {
            None
        } else {
            Some(BitCoord(self.coord))
        }
    }
}

fn diff(x: u32, y: u32) -> u32 {
    if x > y {
        x - y
    } else {
        y - x
    }
}

fn line_mask(start: BitCoord, shift: i32) -> BitBoard {
    let mut mask = BitBoard::EMPTY;
    let mut last_coord = BitCoord(0);

    for coord in Line::new(start, shift) {
        mask = mask | coord;
        last_coord = coord;
    }

    mask & (!last_coord)
}

fn boards_for_mask(mask: BitBoard) -> Vec<BitBoard> {
    let mut boards = Vec::with_capacity(1024);
    boards.push(BitBoard::EMPTY);
    
    for x in 0..64 {
        let coord = BitCoord(1 << x);
        if mask & coord == BitBoard::EMPTY {
            continue;
        }

        for ix in 0..boards.len() {
            boards.push(boards[ix] | coord);
        }
    }

    boards
}

mod generated {
    pub const ROOK_MAGIC: [u64; 64] = [
        0xfc19fe6fec2cf537,  // 0[16384]
        0xeacaa62d652357e7,  // 1[8192]
        0x8f979281a0211aea,  // 2[16384]
        0x671c700f8e25251b,  // 3[8192]
        0x430cb929d5f8c5cf,  // 4[16384]
        0x79400708fb3f7a67,  // 5[16384]
        0xcca97deec7dca69e,  // 6[8192]
        0x8d12c734ebcecec2,  // 7[16384]
        0xe4c8de4524b57290,  // 8[8192]
        0xb3a5de29ce7d195d,  // 9[4096]
        0x8b3e021c9f262b0e,  // 10[4096]
        0xd91d05ee6b0fa113,  // 11[4096]
        0x67d236a3c6a1e2a3,  // 12[4096]
        0x1c53206a99114ed5,  // 13[4096]
        0x1e51b2f9cc05b305,  // 14[2048]
        0x3fad2a6578db0810,  // 15[8192]
        0xa38e5071f1fb3719,  // 16[8192]
        0xeb9e6fef30c89b2d,  // 17[4096]
        0x1697e7f7d11da04d,  // 18[4096]
        0x7611ee11997a354e,  // 19[4096]
        0xa2b2296bc0b35149,  // 20[4096]
        0x4cd92b03e0424e87,  // 21[4096]
        0xc3991d59fe30161c,  // 22[4096]
        0xd88c26fe9c2c6516,  // 23[4096]
        0xd20b418466a3c471,  // 24[8192]
        0x505f816e6743d000,  // 25[4096]
        0x07c72b1f75f95f2f,  // 26[4096]
        0x5771db16d2d3bb4f,  // 27[4096]
        0x38b5f99b58e6515a,  // 28[4096]
        0x8a72dca523400e31,  // 29[4096]
        0xc3f848612b835628,  // 30[4096]
        0xadf924cff49d5f5c,  // 31[4096]
        0xaaa77ff76522e098,  // 32[8192]
        0xc4379ce185f9a4a6,  // 33[4096]
        0x346951ee8ca0d39b,  // 34[4096]
        0x61435d185ee10616,  // 35[4096]
        0x8aa360af7c27f512,  // 36[4096]
        0x77609c69819bd8eb,  // 37[4096]
        0x18efee8681af030e,  // 38[4096]
        0x1386aeda72da63b5,  // 39[4096]
        0xeade55a59f364336,  // 40[4096]
        0x899288f8694e97eb,  // 41[4096]
        0x0dcb50c586597110,  // 42[4096]
        0x90de7c97919b4943,  // 43[4096]
        0x35452ef808c0f1ee,  // 44[4096]
        0x5e34044a59f577d2,  // 45[4096]
        0xdfa39102fe335b9b,  // 46[2048]
        0x5f8486f8cdd68837,  // 47[4096]
        0x2825a8f0a44bb2c2,  // 48[8192]
        0xab2e8a092fc1f15c,  // 49[2048]
        0xfc6740cb73e47507,  // 50[4096]
        0x5fb9bd36f86e79bd,  // 51[2048]
        0xf88bb4ef15e277f0,  // 52[4096]
        0x2127ff4bc8415110,  // 53[2048]
        0xb4a7965983831f3c,  // 54[2048]
        0xf06468ad013996c0,  // 55[4096]
        0xa2f999725d5ebf7a,  // 56[8192]
        0x64ad9ba3331931ce,  // 57[4096]
        0xa55b0586001c71be,  // 58[4096]
        0x70890bc3c1d892ce,  // 59[4096]
        0x35838af8effa74f2,  // 60[8192]
        0x514886983a434ff3,  // 61[8192]
        0x6e6c44b573523dcc,  // 62[4096]
        0x037f859713fc9e2a,  // 63[8192]
    ];

    pub const BISHOP_MAGIC: [u64; 64] = [
        0x2b37884ab6607661,  // 0[256]
        0xf2cd889ec6a9c81e,  // 1[256]
        0xabee7df5cb3794c2,  // 2[256]
        0xf9878d4b4658fc3d,  // 3[256]
        0x1c6ae81d1e1dba94,  // 4[256]
        0x53e7464af1763b00,  // 5[256]
        0x36ce5b74d02f55f8,  // 6[256]
        0xde1b9fbdb734217b,  // 7[256]
        0x0d32b4a42ce5d95d,  // 8[256]
        0xad03884b003e73b2,  // 9[256]
        0xe163799d724b7108,  // 10[256]
        0x7e2316b38260826b,  // 11[256]
        0x7e0168bee116276b,  // 12[256]
        0x898ea1a6f5ef1c26,  // 13[256]
        0x0e40ee533af10702,  // 14[256]
        0xffd5b2405e084c3c,  // 15[256]
        0x90029c75a3c9936a,  // 16[256]
        0x390c41d646750ff8,  // 17[256]
        0xf64cfec68c44afb1,  // 18[256]
        0xaef8045cf83ec705,  // 19[256]
        0x0f0baa520d926f53,  // 20[512]
        0x96a6ffa7fbeb0b5c,  // 21[256]
        0xaa0fbf0b6559a284,  // 22[256]
        0xc19ba18c3b136e8e,  // 23[256]
        0x849678e0857b8678,  // 24[256]
        0xe92851b524c3e5ef,  // 25[256]
        0x3295a5aff4fc17f8,  // 26[256]
        0x61540c70bc720b77,  // 27[4096]
        0xf7f213fff0ca213c,  // 28[2048]
        0xd74a79f63fac428e,  // 29[256]
        0x9f445422f60d14f6,  // 30[256]
        0x7625be0598a36418,  // 31[256]
        0x80f8604a44fb3cc6,  // 32[256]
        0xfb2165468c7a2cd2,  // 33[256]
        0x7ef3aeadfb5bfa12,  // 34[256]
        0xef6f54cd3b3b5288,  // 35[2048]
        0xb8509ef6174b0abd,  // 36[4096]
        0xfcd9217b22a84107,  // 37[256]
        0x2f4fd5406390fa74,  // 38[256]
        0x5f3d0112dadbbe30,  // 39[256]
        0x571f504b69b5cb85,  // 40[256]
        0x9e61752b04a1734c,  // 41[256]
        0x473e4ce818e6ece1,  // 42[256]
        0xa58c2367f762fb27,  // 43[256]
        0xaab79cfb1cad195e,  // 44[256]
        0xc9e3898417ab70a9,  // 45[256]
        0x32b4e5582b435c5b,  // 46[256]
        0xea24b2f93e5f0089,  // 47[256]
        0x8d9a150b5bcc8476,  // 48[256]
        0xcbcd01d98964983c,  // 49[256]
        0xfe60a7831a6e4146,  // 50[256]
        0x6ab7f3b27d215dd6,  // 51[256]
        0xed282405a3b8f28d,  // 52[256]
        0xe79cc209fd70b7a3,  // 53[256]
        0x55f0eb1fb2fa323e,  // 54[256]
        0xed3e4e85a0b0888b,  // 55[256]
        0x949810abb19c47fc,  // 56[256]
        0x6487dcba652389ba,  // 57[256]
        0x1b9780785f4f714a,  // 58[256]
        0x11d0a56b4cbaa168,  // 59[256]
        0xa424afb235fc83c4,  // 60[256]
        0xd1769edf01cf0094,  // 61[256]
        0x5029a8a9e99cbe54,  // 62[256]
        0x7fc97500c80410b4,  // 63[256]
    ];
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::magic::*;
    use crate::types::{BitBoard};

    #[test]
    fn test_rook_mask() {
        let actual = rook_mask(BitCoord(0x00_40_00_00_00_00_00_00));
        let expected = BitBoard(0x00_3E_40_40_40_40_40_00);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bishop_mask() {
        let actual = bishop_mask(BitCoord(0x00_00_20_00_00_00_00_00));
        let expected = BitBoard(0x00_50_00_50_08_04_02_00);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_boards_for_mask() {
        let actual = boards_for_mask(BitBoard(0b101));
        let expected = vec![BitBoard(0b000), BitBoard(0b001), BitBoard(0b100), BitBoard(0b101)];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_rook_moves() {
        let actual = rook_moves(BitCoord(0x00_40_00_00_00_00_00_00), BitBoard(0x00_00_00_40_00_00_00_00));
        let expected = BitBoard(0x40_BF_40_40_00_00_00_00);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_generate_rook() {
        let mut rng = rand::thread_rng();
        // Test a few random configurations.
        for _ in 0..100 {
            let coord = BitCoord(1 << (rng.gen_range(0..64)));
            let moves = generate_moves(coord, rook_mask(coord), rook_moves);
            let magic: Magic = {
                let mut mm: Option<Magic> = None;
                while mm.is_none() {
                    match Magic::generate(rand::random::<u64>(), rook_mask(coord), &moves) {
                        Some(m) => {
                            mm = Some(m);
                        },
                        None => continue,
                    }
                }
                mm.unwrap()
            };
            let board = BitBoard(rand::random::<u64>());
            let actual_moves = rook_moves(coord, board);
            let magic_moves = magic.lookup(board);
            if actual_moves != magic_moves {
                println!("Occupancy: {}", board);
                println!("Actual: {}", actual_moves);
                println!("Magic: {}", magic_moves);
                panic!("Test failed");
            }
        }
    }

    #[test]
    fn test_generated_rook_magic() {
        let mut rng = rand::thread_rng();
        let magic_bbs = MagicBitBoards::default();

        for _ in 0..100_000 {
            let coord = BitCoord(1 << (rng.gen_range(0..64)));
            let board = BitBoard(rng.gen::<u64>());
            let actual_moves = rook_moves(coord, board);
            let magic_moves = magic_bbs.rook(coord).lookup(board);

            if actual_moves != magic_moves {
                println!("Actual: {}", actual_moves);
                println!("Magic: {}", magic_moves);
                panic!("Test failed");
            }
        }
    }

    #[test]
    fn test_generated_bishop_magic() {
        let mut rng = rand::thread_rng();
        let magic_bbs = MagicBitBoards::default();

        for _ in 0..100_000 {
            let coord = BitCoord(1 << (rng.gen_range(0..64)));
            let board = BitBoard(rng.gen::<u64>());
            let actual_moves = bishop_moves(coord, board);
            let magic_moves = magic_bbs.bishop(coord).lookup(board);
            assert_eq!(actual_moves, magic_moves);
        }
    }
}
