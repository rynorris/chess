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
        let mut boards_cache: HashMap<BitBoard, Vec<BitBoard>> = HashMap::new();

        for c in 0..64 {
            let coord = BitCoord(1 << c);
            let rook_moves_map = generate_moves(coord, rook_mask(coord), rook_moves);
            let bishop_moves_map = generate_moves(coord, bishop_mask(coord), bishop_moves);

            rooks.push(Magic::generate(
                    rook_magic[coord.0.trailing_zeros() as usize],
                    rook_mask(coord),
                    &rook_moves_map,
                    &mut boards_cache,
                    8192,
            ).expect(&format!("Rook magic {} is valid", coord.0.trailing_zeros())));

            bishops.push(Magic::generate(
                    bishop_magic[coord.0.trailing_zeros() as usize],
                    bishop_mask(coord),
                    &bishop_moves_map,
                    &mut boards_cache,
                    1024,
            ).expect(&format!("Bishop magic {} is valid", coord.0.trailing_zeros())));

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

#[derive(Clone)]
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

    pub fn magic(&self) -> u64 {
        self.magic
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
        boards_cache: &mut HashMap<BitBoard, Vec<BitBoard>>,
        max_size: usize,
    ) -> Option<Magic> {
        // 2^10 = 1024 possible masked occupancies.
        // Perfect hashing would fit in 256 cells.
        if !boards_cache.contains_key(&mask) {
            boards_cache.insert(mask, boards_for_mask(mask));
        }

        let occupancies = boards_cache.get(&mask).unwrap();

        let mut width: u32 = 4;
        loop {
            let shift = 64 - width;
            let size: usize = 1 << width;

            if size > max_size {
                break;
            }

            let mut table = vec![BitBoard::EMPTY; size];
            let mut filled = vec![false; size];

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

            width += 1;
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

pub struct Line {
    coord: u64,
    shift: i32,
}

impl Line {
    pub fn new(start: BitCoord, shift: i32) -> Line {
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
        0xb76a2db6b51796a8,  // 0[8192]
        0xdd4bff3eb7476d18,  // 1[4096]
        0xd6b9fe01f4dd9d9a,  // 2[4096]
        0x57f0009727effb29,  // 3[4096]
        0xaca716f7f2c66cec,  // 4[8192]
        0x34e00f121491fbf8,  // 5[4096]
        0xacc00dc593418156,  // 6[4096]
        0x5000959a495452d5,  // 7[8192]
        0x936dffeb83ca00fc,  // 8[4096]
        0x07cdae127e71dd1a,  // 9[2048]
        0x5405736e66d8905a,  // 10[2048]
        0x9cd21642f7a2bfc0,  // 11[2048]
        0xced60e538fa9f17d,  // 12[2048]
        0xc77c8db95d4825eb,  // 13[2048]
        0x1e51b2f9cc05b305,  // 14[2048]
        0xa07a000184030e62,  // 15[2048]
        0x563be107f08c4dff,  // 16[4096]
        0x715e8d1733ce20b7,  // 17[2048]
        0x78f58b8e37fed5aa,  // 18[2048]
        0x32cdcd20a50d20e3,  // 19[2048]
        0x159f50f0f10e8361,  // 20[2048]
        0x67067200090600aa,  // 21[2048]
        0x254b888726442aa0,  // 22[2048]
        0x4e1072001581d70c,  // 23[2048]
        0x7d7da4e38bc5e6cd,  // 24[4096]
        0xaf590eba201956f1,  // 25[2048]
        0x3e1bf7823af9bdef,  // 26[2048]
        0x177119e1c71718d7,  // 27[2048]
        0xb42c768a6fd72590,  // 28[2048]
        0x41bd7a7d82d0db0b,  // 29[2048]
        0xcccfa814002aa710,  // 30[1024]
        0x5bf2f65a0000e6ac,  // 31[2048]
        0xf0d69c97632ecb07,  // 32[4096]
        0xb90760c5b515e61c,  // 33[2048]
        0xe0dcd44ef10f8402,  // 34[2048]
        0x268408f883f8b000,  // 35[2048]
        0x45b585a05f4f4f7c,  // 36[2048]
        0x94292c7dfab7ed48,  // 37[2048]
        0x71568bc15ebb66a3,  // 38[2048]
        0x7e0ea8a72e0005a4,  // 39[2048]
        0x6ba0894003a68006,  // 40[2048] 
        0xc6fe9a9f9d5e0e9b,  // 41[2048]
        0xe1b8d0a5cc2f59e3,  // 42[2048]
        0xc1612a7fdd20022a,  // 43[2048]
        0x2fe7405dcce8bf20,  // 44[2048]
        0xefb5ffdbaeddfff0,  // 45[2048]
        0xdfa39102fe335b9b,  // 46[2048]
        0x9a723b2b745a0004,  // 47[2048]
        0x98f5474730719abf,  // 48[4096]
        0xf67fffba7e7e12f0,  // 49[1024]
        0x29ef5020814028a0,  // 50[2048]
        0x840ba100f0014b00,  // 51[1024]
        0x38dd435e50d731e5,  // 52[2048]
        0x2127ff4bc8415110,  // 53[2048]
        0xb4a7965983831f3c,  // 54[2048]
        0xaa18e84117940600,  // 55[2048]
        0x66fffee689ce02fe,  // 56[4096]
        0x1bfffce87017d1ca,  // 57[2048]
        0xa565ffb3e00d80be,  // 58[2048]
        0x947fff083092008e,  // 59[2048]
        0x37df1a00342e2f3e,  // 60[4096]
        0x15fa2465c6f1ffe2,  // 61[4096]
        0x56bc6b9002183114,  // 62[2048]
        0xe7ce270084182c36,  // 63[4096]
    ];

    pub const BISHOP_MAGIC: [u64; 64] = [
        0xc9e0b32f7efb37ff,  // 0[64]
        0xbcc5efc5767ff79a,  // 1[16]
        0xbdf044b22bf68725,  // 2[32]
        0xcf7825efcd929f29,  // 3[32]
        0x798c0c206fafba0b,  // 4[32]
        0xdd220a8ffcf4be24,  // 5[32]
        0x9c50a28a537f9f27,  // 6[16]
        0x378bba1d8ef5ffe6,  // 7[32]
        0x85faadad9b78efff,  // 8[16]
        0x3be609029afd97f6,  // 9[16]
        0xf9fdd869ffce07ce,  // 10[32]
        0x942a3114058191bf,  // 11[32]
        0xd9b87c1420629674,  // 12[32]
        0xe238de02301c578f,  // 13[32]
        0x04644b251bf87f8a,  // 14[16]
        0x44e2b6d5376dfec6,  // 15[16]
        0x86107b60600e779c,  // 16[32]
        0xe1e61142652177fd,  // 17[32]
        0x52c802b00e85a209,  // 18[128]
        0xf878010904530057,  // 19[128]
        0xb6a700e090402215,  // 20[128]
        0x1396001a4113a032,  // 21[128]
        0xed17054a14ff60dc,  // 22[32]
        0x4907041dcb27b00c,  // 23[32]
        0x7810c0237618b1aa,  // 24[32]
        0xf1ff5e89387e3c06,  // 25[32]
        0x382b7002480687e1,  // 26[128]
        0x718da08cd046c6b6,  // 27[1024]
        0xd3af7877c05e2061,  // 28[1024]
        0xbd7f85004e00d206,  // 29[128]
        0xdfa50c183452180e,  // 30[32]
        0x1496034cf4e10825,  // 31[32]
        0xf0a920a036d0349f,  // 32[32]
        0x74762fb0ab170610,  // 33[32]
        0xe5d1b24805d00783,  // 34[128]
        0x58cdffaa517fea00,  // 35[1024]
        0x78a8ee38e3954c79,  // 36[1024]
        0x3a4d0b5200b5010d,  // 37[128]
        0x0484158a09c1980d,  // 38[32]
        0x436f2606823a04ce,  // 39[32]
        0xce3dc3dba34cc051,  // 40[32]
        0x98aff3e5953b8829,  // 41[32]
        0x9523ff9710015901,  // 42[128]
        0x7a14ad6018009d05,  // 43[128]
        0x1be07a4602013c10,  // 44[128]
        0x06fe22054d020201,  // 45[128]
        0xf57ee9cce33a0942,  // 46[32]
        0x741fb5826e5d0a00,  // 47[16]
        0x84bff89913cd4d69,  // 48[16]
        0x108ffc98c4cd2780,  // 49[16]
        0x66b9fff61ef084b3,  // 50[32]
        0xe5ff84ac20881d3a,  // 51[32]
        0x86b5dfd06e0217c3,  // 52[32]
        0xe9bc3d202bf3c905,  // 53[32]
        0xc9ff7491dcd8cf7e,  // 54[16]
        0xa0bfe9f514524727,  // 55[16]
        0xbf47fbeeaa7939fe,  // 56[64]
        0x508157fe864b7938,  // 57[16]
        0x2bb52b32ea011010,  // 58[32]
        0x168cd40c87085806,  // 59[32]
        0xa7e927aca8306c0d,  // 60[32]
        0xfcf6d6a8710f2a01,  // 61[32]
        0x57ed7fb519d8a0ae,  // 62[16]
        0xadffd1e9127f66b4,  // 63[32]
    ];
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;
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
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        // Test a few random configurations.
        for _ in 0..100 {
            let coord = BitCoord(1 << (rng.gen_range(0..64)));
            let moves = generate_moves(coord, rook_mask(coord), rook_moves);
            let mut boards_cache: HashMap<BitBoard, Vec<BitBoard>> = HashMap::new();

            let magic: Magic = {
                let mut mm: Option<Magic> = None;
                while mm.is_none() {
                    match Magic::generate(rand::random::<u64>(), rook_mask(coord), &moves, &mut boards_cache, 8192) {
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
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
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
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
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
