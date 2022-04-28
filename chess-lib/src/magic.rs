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

    pub fn generate(rook_magics: [(u64, u32); 64], bishop_magics: [(u64, u32); 64]) -> MagicBitBoards {
        let mut rooks: Vec<Magic> = Vec::with_capacity(64);
        let mut bishops: Vec<Magic> = Vec::with_capacity(64);
        let mut kings: Vec<BitBoard> = Vec::with_capacity(64);
        let mut knights: Vec<BitBoard> = Vec::with_capacity(64);
        let mut boards_cache: HashMap<BitBoard, Vec<BitBoard>> = HashMap::new();

        for c in 0..64 {
            let coord = BitCoord(1 << c);
            let rook_moves_map = generate_moves(coord, rook_mask(coord), rook_moves);
            let bishop_moves_map = generate_moves(coord, bishop_mask(coord), bishop_moves);

            let (rook_magic, rook_bits) = rook_magics[coord.0.trailing_zeros() as usize];
            rooks.push(Magic::generate(
                    rook_magic,
                    rook_mask(coord),
                    &rook_moves_map,
                    &mut boards_cache,
                    1 << rook_bits,
            ).expect(&format!("Rook magic {} is valid", coord.0.trailing_zeros())));

            let (bishop_magic, bishop_bits) = bishop_magics[coord.0.trailing_zeros() as usize];
            bishops.push(Magic::generate(
                    bishop_magic,
                    bishop_mask(coord),
                    &bishop_moves_map,
                    &mut boards_cache,
                    1 << bishop_bits,
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

    pub fn shift(&self) -> u32 {
        self.shift
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

            let mut success = true;
            for o in occupancies.iter() {
                let index = Magic::index(*o, mask, magic, shift);
                let moves = all_moves.get(o).expect("Missing some moves");

                let current = table[index];
                if current != BitBoard::EMPTY && current != *moves {
                    success = false;
                    break;
                }

                table[index] = *moves;
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
    pub const ROOK_MAGIC: [(u64, u32); 64] = [
        (0xb76a2db6b51796a8, 13),  // 0[8192]
        (0x78c003e007100142, 11),  // 1[2048]
        (0xd6b9fe01f4dd9d9a, 12),  // 2[4096]
        (0x57f0009727effb29, 12),  // 3[4096]
        (0xecfff1a1c80de300, 12),  // 4[4096]
        (0x34e00f121491fbf8, 12),  // 5[4096]
        (0xacc00dc593418156, 12),  // 6[4096]
        (0x5000959a495452d5, 13),  // 7[8192]
        (0x71ba80076182400c, 11),  // 8[2048]
        (0x9daf002740008705, 10),  // 9[1024]
        (0x5405736e66d8905a, 11),  // 10[2048]
        (0x9cd21642f7a2bfc0, 11),  // 11[2048]
        (0xced60e538fa9f17d, 11),  // 12[2048]
        (0xc77c8db95d4825eb, 11),  // 13[2048]
        (0x9252008dfff7ff24, 10),  // 14[1024]
        (0xa07a000184030e62, 11),  // 15[2048]
        (0x563be107f08c4dff, 12),  // 16[4096]
        (0x715e8d1733ce20b7, 11),  // 17[2048]
        (0x78f58b8e37fed5aa, 11),  // 18[2048]
        (0x32cdcd20a50d20e3, 11),  // 19[2048]
        (0x159f50f0f10e8361, 11),  // 20[2048]
        (0x67067200090600aa, 11),  // 21[2048]
        (0x734b5c000330182e, 10),  // 22[1024]
        (0x4e1072001581d70c, 11),  // 23[2048]
        (0x7d7da4e38bc5e6cd, 12),  // 24[4096]
        (0xaf590eba201956f1, 11),  // 25[2048]
        (0xde6bb30100200541, 10),  // 26[1024]
        (0x177119e1c71718d7, 11),  // 27[2048]
        (0xb42c768a6fd72590, 11),  // 28[2048]
        (0x41bd7a7d82d0db0b, 11),  // 29[2048]
        (0xcccfa814002aa710, 10),  // 30[1024]
        (0x5bf2f65a0000e6ac, 11),  // 31[2048]
        (0xf0d69c97632ecb07, 12),  // 32[4096]
        (0x1a750980c2002203, 10),  // 33[1024]
        (0xe0dcd44ef10f8402, 11),  // 34[2048]
        (0x268408f883f8b000, 11),  // 35[2048]
        (0x45b585a05f4f4f7c, 11),  // 36[2048]
        (0x94292c7dfab7ed48, 11),  // 37[2048]
        (0x09c582172c002890, 10),  // 38[1024]
        (0x7e0ea8a72e0005a4, 11),  // 39[2048]
        (0x6ba0894003a68006, 11),  // 40[2048] 
        (0xc6fe9a9f9d5e0e9b, 11),  // 41[2048]
        (0xe1b8d0a5cc2f59e3, 11),  // 42[2048]
        (0xc62fc20060da0010, 10),  // 43[1024]
        (0x2fe7405dcce8bf20, 11),  // 44[2048]
        (0xefb5ffdbaeddfff0, 11),  // 45[2048]
        (0x4009e50a106c0058, 10),  // 46[1024]
        (0x9a723b2b745a0004, 11),  // 47[2048]
        (0x75ffff5e85b94450, 11),  // 48[2048]
        (0xf67fffba7e7e12f0, 10),  // 49[1024]
        (0x1eaed4200109c100, 10),  // 50[1024]
        (0x840ba100f0014b00, 10),  // 51[1024]
        (0x95ffff77515d5200, 10),  // 52[1024]
        (0x2127ff4bc8415110, 11),  // 53[2048]
        (0xe7eaf01207b81c00, 10),  // 54[1024]
        (0xaa18e84117940600, 11),  // 55[2048]
        (0x66fffee689ce02fe, 12),  // 56[4096]
        (0x1bfffce87017d1ca, 11),  // 57[2048]
        (0xa565ffb3e00d80be, 11),  // 58[2048]
        (0x947fff083092008e, 11),  // 59[2048]
        (0x353fffaee5ffbdba, 11),  // 60[2048]
        (0x3a360008241f307a, 11),  // 61[2048]
        (0x56bc6b9002183114, 11),  // 62[2048]
        (0xe7ce270084182c36, 12),  // 63[4096]
    ];

    pub const BISHOP_MAGIC: [(u64, u32); 64] = [
        (0xc9e0b32f7efb37ff, 6),  // 0[64]
        (0xbcc5efc5767ff79a, 4),  // 1[16]
        (0xbdf044b22bf68725, 5),  // 2[32]
        (0xcf7825efcd929f29, 5),  // 3[32]
        (0x798c0c206fafba0b, 5),  // 4[32]
        (0xdd220a8ffcf4be24, 5),  // 5[32]
        (0x9c50a28a537f9f27, 4),  // 6[16]
        (0x378bba1d8ef5ffe6, 5),  // 7[32]
        (0x85faadad9b78efff, 4),  // 8[16]
        (0x3be609029afd97f6, 4),  // 9[16]
        (0xf9fdd869ffce07ce, 5),  // 10[32]
        (0x942a3114058191bf, 5),  // 11[32]
        (0xd9b87c1420629674, 5),  // 12[32]
        (0xe238de02301c578f, 5),  // 13[32]
        (0x04644b251bf87f8a, 4),  // 14[16]
        (0x44e2b6d5376dfec6, 4),  // 15[16]
        (0x86107b60600e779c, 5),  // 16[32]
        (0xe1e61142652177fd, 5),  // 17[32]
        (0x52c802b00e85a209, 7),  // 18[128]
        (0xf878010904530057, 7),  // 19[128]
        (0xb6a700e090402215, 7),  // 20[128]
        (0x1396001a4113a032, 7),  // 21[128]
        (0xed17054a14ff60dc, 5),  // 22[32]
        (0x4907041dcb27b00c, 5),  // 23[32]
        (0x7810c0237618b1aa, 5),  // 24[32]
        (0xf1ff5e89387e3c06, 5),  // 25[32]
        (0x382b7002480687e1, 7),  // 26[128]
        (0x718da08cd046c6b6, 10),  // 27[1024]
        (0xd3af7877c05e2061, 10),  // 28[1024]
        (0xbd7f85004e00d206, 7),  // 29[128]
        (0xdfa50c183452180e, 5),  // 30[32]
        (0x1496034cf4e10825, 5),  // 31[32]
        (0xf0a920a036d0349f, 5),  // 32[32]
        (0x74762fb0ab170610, 5),  // 33[32]
        (0xe5d1b24805d00783, 7),  // 34[128]
        (0x58cdffaa517fea00, 10),  // 35[1024]
        (0x78a8ee38e3954c79, 10),  // 36[1024]
        (0x3a4d0b5200b5010d, 7),  // 37[128]
        (0x0484158a09c1980d, 5),  // 38[32]
        (0x436f2606823a04ce, 5),  // 39[32]
        (0xce3dc3dba34cc051, 5),  // 40[32]
        (0x98aff3e5953b8829, 5),  // 41[32]
        (0x9523ff9710015901, 7),  // 42[128]
        (0x7a14ad6018009d05, 7),  // 43[128]
        (0x1be07a4602013c10, 7),  // 44[128]
        (0x06fe22054d020201, 7),  // 45[128]
        (0xf57ee9cce33a0942, 5),  // 46[32]
        (0x741fb5826e5d0a00, 4),  // 47[16]
        (0x84bff89913cd4d69, 4),  // 48[16]
        (0x108ffc98c4cd2780, 4),  // 49[16]
        (0x66b9fff61ef084b3, 5),  // 50[32]
        (0xe5ff84ac20881d3a, 5),  // 51[32]
        (0x86b5dfd06e0217c3, 5),  // 52[32]
        (0xe9bc3d202bf3c905, 5),  // 53[32]
        (0xc9ff7491dcd8cf7e, 4),  // 54[16]
        (0xa0bfe9f514524727, 4),  // 55[16]
        (0xbf47fbeeaa7939fe, 6),  // 56[64]
        (0x508157fe864b7938, 4),  // 57[16]
        (0x2bb52b32ea011010, 5),  // 58[32]
        (0x168cd40c87085806, 5),  // 59[32]
        (0xa7e927aca8306c0d, 5),  // 60[32]
        (0xfcf6d6a8710f2a01, 5),  // 61[32]
        (0x57ed7fb519d8a0ae, 4),  // 62[16]
        (0xadffd1e9127f66b4, 5),  // 63[32]
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
                    match Magic::generate(rand::random::<u64>(), rook_mask(coord), &moves, &mut boards_cache, 1 << 15) {
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
