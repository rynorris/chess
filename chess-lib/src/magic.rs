use crate::types::{BitBoard, BitCoord};

pub struct MagicBitBoards {
    rooks: Vec<Magic>,
    bishops: Vec<Magic>,
}

impl MagicBitBoards {
    pub fn default() -> MagicBitBoards {
        Self::generate(generated::ROOK_MAGIC, generated::BISHOP_MAGIC)
    }

    pub fn generate(rook_magic: [u64; 64], bishop_magic: [u64; 64]) -> MagicBitBoards {
        let mut rooks: Vec<Magic> = Vec::with_capacity(64);
        let mut bishops: Vec<Magic> = Vec::with_capacity(64);
        for c in 0..64 {
            let coord = BitCoord(1 << c);
            rooks.push(Magic::generate_rook(rook_magic[coord.0.trailing_zeros() as usize], BitCoord(1 << c)).unwrap());
            bishops.push(Magic::generate_bishop(bishop_magic[coord.0.trailing_zeros() as usize], BitCoord(1 << c)).unwrap());
        }

        MagicBitBoards{rooks, bishops}
    }

    pub fn rook(&self, coord: BitCoord) -> &Magic {
        &self.rooks[coord.0.trailing_zeros() as usize]
    }

    pub fn bishop(&self, coord: BitCoord) -> &Magic {
        &self.bishops[coord.0.trailing_zeros() as usize]
    }
}

pub struct Magic {
    table: Vec<BitBoard>,
    mask: BitBoard,
    magic: u64,
    shift: u32,
}

const INNER_MASK: u64 = 0x00_7e_7e_7e_7e_7e_7e_00;

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

    pub fn generate_rook(magic: u64, coord: BitCoord) -> Option<Magic> {
        Self::generate(magic, coord, rook_mask, rook_moves)
    }

    pub fn generate_bishop(magic: u64, coord: BitCoord) -> Option<Magic> {
        Self::generate(magic, coord, bishop_mask, bishop_moves)
    }

    pub fn generate(
        magic: u64,
        coord: BitCoord,
        maskgen: fn (BitCoord) -> BitBoard,
        movegen: fn (BitCoord, BitBoard) -> BitBoard,
    ) -> Option<Magic> {
        // 2^10 = 1024 possible masked occupancies.
        // Perfect hashing would fit in 256 cells.
        let mask = maskgen(coord);
        let occupancies = boards_for_mask(mask);

        for size in 8..=12 {
            let shift = 64 - size;
            let mut table = vec![BitBoard::EMPTY; 1 << size];
            let mut filled = vec![false; 1 << size];

            let mut success = true;
            for o in occupancies.iter() {
                let index = Magic::index(*o, mask, magic, shift);
                let moves = movegen(coord, *o);
                if !filled[index] {
                    table[index] = moves;
                    filled[index] = true;
                    continue;
                }

                if table[index] != moves {
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

fn rook_mask(coord: BitCoord) -> BitBoard {
    line_mask(coord, 1) | line_mask(coord, 64 - 1) | line_mask(coord, 8) | line_mask(coord, 64 - 8)
}

fn rook_moves(start: BitCoord, occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [1, 8, 64 - 1, 64 - 8].iter() {
        for coord in Line::new(start, *dir) {
            moves = moves | coord;
            if occupancy & coord != BitBoard::EMPTY {
                break;
            }
        }
    }

    moves
}

fn bishop_mask(coord: BitCoord) -> BitBoard {
    line_mask(coord, 9) | line_mask(coord, 64 - 9) | line_mask(coord, 7) | line_mask(coord, 64 - 7)
}

fn bishop_moves(start: BitCoord, occupancy: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    for dir in [7, 9, 64 - 7, 64 - 9].iter() {
        for coord in Line::new(start, *dir) {
            moves = moves | coord;
            if occupancy & coord != BitBoard::EMPTY {
                break;
            }
        }
    }

    moves
}

struct Line {
    coord: u64,
    shift: u32,
}

impl Line {
    fn new(start: BitCoord, shift: u32) -> Line {
        Line{
            coord: start.0,
            shift,
        }
    }
}

impl Iterator for Line {
    type Item = BitCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coord == 0 {
            return None
        }

        self.coord = self.coord.rotate_left(self.shift);
        let nxt = Some(BitCoord(self.coord));
        if self.coord & INNER_MASK == 0 {
            self.coord = 0;
        }

        nxt
    }
}

fn line_mask(start: BitCoord, shift: u32) -> BitBoard {
    let mut mask = BitBoard::EMPTY;

    for coord in Line::new(start, shift) {
        mask = mask | coord;
    }

    mask & INNER_MASK
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
        7318730101365882109,
        13918138860097688298,
        6379511917137240160,
        584281744042479507,
        11880579848240401801,
        17798197070762577132,
        12772047277938945535,
        9243312960803485918,
        17910046109851947197,
        9973819516298166262,
        1152650646648270620,
        14840627584551836772,
        4868154247761511827,
        1972335820218734390,
        1778447406079143747,
        7032058808221398894,
        11170074723843556258,
        8903634066881534747,
        3390273346794418842,
        6636205248622942332,
        8802335086591514653,
        7732074800356010537,
        450170845675430662,
        6144194463625738060,
        15888356658330194585,
        9059145848138251668,
        4844433483735269350,
        14946840612690504417,
        2131333900901757982,
        671095440335332289,
        9987902867935040492,
        12610880065274861585,
        15928797870679920171,
        1283803048829032067,
        13164050679477890988,
        9835329599541273977,
        15037127446014982656,
        16220026149555165702,
        17165476585373934831,
        12265476125666619434,
        13666028341609107392,
        16770166924131862512,
        3685174461418508320,
        1946238055934477918,
        10461804219591862510,
        6719379877049348101,
        11661599357987567151,
        16039539961344729171,
        8055227416381383454,
        8946141372254262784,
        7370412134493463916,
        9599474656535008030,
        8649207445176075074,
        2699080145021749930,
        16929401376011328526,
        6190393392494847991,
        11948877292500961264,
        16351966850438144150,
        16605046368466837003,
        9356260781071982750,
        6822520162779630907,
        14263076139933141526,
        12306394020589150851,
        14409863329953669300,
    ];

    pub const BISHOP_MAGIC: [u64; 64] = [
        7151652312763427893,
        13194642546697694465,
        3833508239341365118,
        9885562817151386972,
        9508817358002745117,
        11183632502371359776,
        15142031059668215335,
        13968335907739081718,
        11988344797868136150,
        5214163953010587096,
        6377142112234490225,
        16755594444978376714,
        4848653307751312232,
        10478702003886345970,
        13508685031193448700,
        5207038863346759135,
        12779767202687574107,
        9512062205417579917,
        7112032746381020256,
        6885438255439953972,
        2170611184150955259,
        4822792460493774831,
        4471899643903503153,
        12997112633536717752,
        18077497145854903955,
        5247931104766456056,
        1694348443751498204,
        9337014659516865789,
        9499126976041960450,
        15287810882428362738,
        3046165162433805900,
        15071778646520856232,
        6483640912386336550,
        1001012655074942660,
        4555950479159770914,
        17625598162253448301,
        7320906560376894470,
        15365110956074090659,
        10126895450170147095,
        3154981859374535098,
        8746970699480683512,
        15852289914160663875,
        13063910906638725380,
        12400351887215294376,
        12650356564085134615,
        2437975998214627067,
        736081946407015041,
        16342253143245812172,
        9677898851406001201,
        12258150507668859047,
        9265883073088409987,
        5894571724841352974,
        11597166303844373211,
        8613296875433344844,
        8298246179503732561,
        8908073261140267094,
        7066802352621490035,
        8091333618441291437,
        11730155112728241686,
        8957394586775416599,
        11172342059769289420,
        14019179725634323730,
        4310311938154446224,
        8595409343331003240,
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
        let coord = BitCoord(0x00_40_00_00_00_00_00_00);

        let magic: Magic = {
            let mut mm: Option<Magic> = None;
            while mm.is_none() {
                match Magic::generate_rook(rand::random::<u64>(), coord) {
                    Some(m) => {
                        mm = Some(m);
                    },
                    None => continue,
                }
            }
            mm.unwrap()
        };

        // Test a few random boards.
        for _ in 0..1000 {
            let board = BitBoard(rand::random::<u64>());
            let actual_moves = rook_moves(coord, board);
            let magic_moves = magic.lookup(board);
            assert_eq!(actual_moves, magic_moves);
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
            assert_eq!(actual_moves, magic_moves);
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
