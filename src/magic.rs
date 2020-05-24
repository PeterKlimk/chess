use super::BitBoard;
use std::cmp::min;

const MAGIC_ROOKS: [u64; 64] = [
    36033423772491904,2323857820178460676,6953575418995671177,5800645116222767232,3602914904120517120,3530826506039331584,36030996260324736,612491476202422400,
    29977089876705289,2392674750709762,140874928357504,2378463865669288032,1460010738574693424,1127068675080200,5045720449750146560,1820017200552345993,
    148760075017338888,4791672755986944,5914634786208481552,865395365786947584,6956092199235053568,297378862684210176,708085507330056,5233184966038274820,
    2328572120025539200,4538786151139712,1161928983035576352,580964903839401984,2612369877362573376,72621102338151464,7098813223479545986,5764962141304046609,
    5190468957148021888,2305918463203348480,1163337454493638656,2306986503496009729,81143962441614384,1157455892715676160,37198746177538402,2378182217831547202,
    4634555895190159372,1802319528982495232,17592722948224,147846931886506016,562984382496784,72061992118026368,2314886501210783752,4632023139549839396,
    4900067030927172096,110443759023423616,1335880889091154432,146508034618688128,4725410709406748928,38317980295135360,4661968359261184,4830190641549972992,
    2506852087170629635,2450028604691587201,3465062517514963089,90635767277432866,83119780659203843,6009490924839705730,1688854358698010,595179697852481794,
];

const MAGIC_BISHOPS: [u64; 64] = [
    18023198964588608,565183370002432,2269958943801344,73187894138798080,299342044858368,143074487468032,145204288946176,282027148968448,
    4432440328704,2233416810560,1143494273941504,22007420813312,72062009801244672,2203587182592,2207881758720,1101693714944,18014535982121472,
    9007207979024512,580610926055968,73183529379307520,3386497971519504,457401401090048,70385991418880,281477128913408,2256197996513280,
    316659416433152,43991470997536,2814887239778816,145135543263240,1691048885653512,145135568552448,1126037350121984,598203047084544,
    1130306545385984,281754150111232,72059795209191808,1130349492998400,9008299840505856,2252351717245440,2252903620296768,290408575799296,
    1126466909638656,288249102343340288,412719513856,2305851809619510276,598151522418721,1143500687016448,4505800814886944,74835644907520,
    74775514972160,283477278720,2216951808,34630537216,70386058298368,1134764786483200,571754644897792,18693979652096,1116708341760,
    8608843776,9007199259066880,9007199523308032,277059207424,4432540991616,18031992859803776,
];

pub struct MagicCache {
    pub bishop_bits: Vec<u32>,
    pub rook_bits: Vec<u32>,

    pub bishop_masks: Vec<BitBoard>,
    pub rook_masks: Vec<BitBoard>,

    pub rook_cache: Vec<Vec<BitBoard>>,
    pub bishop_cache: Vec<Vec<BitBoard>>,

    pub rook_rays: Vec<BitBoard>,
    pub bishop_rays: Vec<BitBoard>,
}

impl MagicCache {
    pub fn rook_moves(&self, pos: u32, occupancy: BitBoard) -> BitBoard {
        let masked = self.rook_masks[pos as usize] & occupancy;
        let bits = self.rook_bits[pos as usize];
        let key = (masked.0 * MAGIC_ROOKS[pos as usize]) >> (64 - bits);
        
        self.rook_cache[pos as usize][key as usize]
    }

    pub fn bishop_moves(&self, pos: u32, occupancy: BitBoard) -> BitBoard {
        let masked = self.bishop_masks[pos as usize] & occupancy;
        let bits = self.bishop_bits[pos as usize];
        let key = (masked.0 * MAGIC_BISHOPS[pos as usize]) >> (64 - bits);

        self.bishop_cache[pos as usize][key as usize]
    }

    pub fn rook_ray (&self, pos: u32, other: u32) -> BitBoard {
        self.rook_rays[(pos * 64 + other) as usize]
    }

    pub fn bishop_ray (&self, pos: u32, other: u32) -> BitBoard {
        self.bishop_rays[(pos * 64 + other) as usize]
    }

    fn gen_bishop_rays() -> Vec<BitBoard> {
        let mut bishop_rays = vec![BitBoard::new(); 64*64];

        for pos in 0..64 {
            let (x, y) = (pos % 8, pos / 8);
    
            let mut bb = BitBoard::new();
            let (mut x2, mut y2) = (x, y);
            while x2 < 7 && y2 < 7 {
                x2 += 1; y2 += 1;
                let other = x2 + y2 * 8;
                bb = bb.add_pos(other);
                bishop_rays[(pos * 64 + other) as usize] = bb;
            }
    
            let mut bb = BitBoard::new();
            let (mut x2, mut y2) = (x, y);
            while x2 < 7 && y2 > 0 {
                x2 += 1; y2 -= 1;
                let other = x2 + y2 * 8;
                bb = bb.add_pos(other);
                bishop_rays[(pos * 64 + other) as usize] = bb;
            }
    
            let mut bb = BitBoard::new();
            let (mut x2, mut y2) = (x, y);
            while x2 > 0 && y2 > 0 {
                x2 -= 1; y2 -= 1;
                let other = x2 + y2 * 8;
                bb = bb.add_pos(other);
                bishop_rays[(pos * 64 + other) as usize] = bb;
            }
    
            let mut bb = BitBoard::new();
            let (mut x2, mut y2) = (x, y);
            while x2 > 0 && y2 < 7 {
                x2 -= 1; y2 += 1;
                let other = x2 + y2 * 8;
                bb = bb.add_pos(other);
                bishop_rays[(pos * 64 + other) as usize] = bb;
            }
        }

        bishop_rays
    }

    fn gen_rook_rays() -> Vec<BitBoard> {
        let mut rook_rays = vec![BitBoard::new(); 64*64];

        for pos in 0..64 {
            let (x, y) = (pos % 8, pos / 8);

            let mut bb = BitBoard::new();
            for y2 in 0..y { 
                let other = x + y2 * 8;
                bb = bb.add_pos(other);
                rook_rays[(pos * 64 + other) as usize] = bb;
            }

            let mut bb = BitBoard::new();
            for y2 in (y+1)..8 { 
                let other = x + y2 * 8;
                bb = bb.add_pos(other);
                rook_rays[(pos * 64 + other) as usize] = bb;
            }

            let mut bb = BitBoard::new();
            for x2 in 0..x { 
                let other = x2 + y * 8;
                bb = bb.add_pos(other);
                rook_rays[(pos * 64 + other) as usize] = bb;
            }

            let mut bb = BitBoard::new();
            for x2 in (x+1)..8 { 
                let other = x2 + y * 8;
                bb = bb.add_pos(other);
                rook_rays[(pos * 64 + other) as usize] = bb;
            }
        }

        rook_rays
    }

    pub fn new() -> Self {
        let mut rook_bits = Vec::new();
        let mut bishop_bits = Vec::new();

        let mut rook_masks = Vec::new();
        let mut bishop_masks = Vec::new();

        for pos in 0..64 {
            let rook_mask = Self::rook_mask(pos);
            let bishop_mask = Self::bishop_mask(pos);

            rook_bits.push(rook_mask.count());
            bishop_bits.push(bishop_mask.count());

            rook_masks.push(rook_mask);
            bishop_masks.push(bishop_mask);
        }

        let mut rook_cache = Vec::new();
        let mut bishop_cache = Vec::new();

        for pos in 0..64 {
            let rb = rook_bits[pos as usize];
            let bb = bishop_bits[pos as usize];

            let mut crc = vec![BitBoard::new(); 2usize.pow(rb)];
            let mut cbc = vec![BitBoard::new(); 2usize.pow(bb)];

            let possible_rooks = Self::gen_rook(pos);
            let possible_bishops = Self::gen_bishop(pos);

            for rook in possible_rooks {
                let key = (rook.0 * MAGIC_ROOKS[pos as usize]) >> (64 - rb);
                let result = Self::solve_rook(rook, pos);
                crc[key as usize] = result;
            }

            for bishop in possible_bishops {
                let key = (bishop.0 * MAGIC_BISHOPS[pos as usize]) >> (64 - bb);
                let result = Self::solve_bishop(bishop, pos);
                cbc[key as usize] = result;
            }

            rook_cache.push(crc);
            bishop_cache.push(cbc);
        }

        Self {
            rook_cache,
            rook_masks,
            rook_bits,
            bishop_cache,
            bishop_masks,
            bishop_bits,

            rook_rays: Self::gen_rook_rays(),
            bishop_rays: Self::gen_bishop_rays(), 
        }
    }

    pub fn rook_mask (pos: u32) -> BitBoard {
        let mut bb = BitBoard::new();
        let (x, y) = (pos % 8, pos / 8);

        for y2 in 1..y { bb = bb.add_pos(x + y2 * 8); }
        for y2 in (y+1)..7 { bb = bb.add_pos(x + y2 * 8); }
        for x2 in 1..x { bb = bb.add_pos(x2 + y * 8); }
        for x2 in (x+1)..7 { bb = bb.add_pos(x2 + y * 8); }

        bb
    }

    pub fn bishop_mask (pos: u32) -> BitBoard {
        let mut bb = BitBoard::new();

        let x = pos % 8;
        let y = pos / 8;

        let (mut x2, mut y2) = (x, y);
        while x2 < 6 && y2 < 6 {
            x2 += 1; y2 += 1;
            bb = bb.add_pos(x2 + y2 * 8);
        }

        let (mut x2, mut y2) = (x, y);
        while x2 < 6 && y2 > 1 {
            x2 += 1; y2 -= 1;
            bb = bb.add_pos(x2 + y2 * 8);
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 1 && y2 > 1 {
            x2 -= 1; y2 -= 1;
            bb = bb.add_pos(x2 + y2 * 8);
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 1 && y2 < 6 {
            x2 -= 1; y2 += 1;
            bb = bb.add_pos(x2 + y2 * 8);
        }

        bb
    }

    pub fn solve_rook (mask: BitBoard, pos: u32) -> BitBoard {
        let (x, y) = (pos % 8, pos / 8);
        let mut result = BitBoard::new();

        let mut x2 = x;
        while x2 < 7 {
            x2 += 1;
            let new_pos = y * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let mut x2 = x;
        while x2 > 0 {
            x2 -= 1;
            let new_pos = y * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let mut y2 = y;
        while y2 < 7 {
            y2 += 1;
            let new_pos = y2 * 8 + x;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let mut y2 = y;
        while y2 > 0 {
            y2 -= 1;
            let new_pos = y2 * 8 + x;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        result
    }

    pub fn solve_bishop (mask: BitBoard, pos: u32) -> BitBoard {
        let mut result = BitBoard::new();

        let x = pos % 8;
        let y = pos / 8;

        let (mut x2, mut y2) = (x, y);
        while x2 < 7 && y2 < 7 {
            x2 += 1; y2 += 1;
            let new_pos = y2 * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 < 7 && y2 > 0 {
            x2 += 1; y2 -= 1;
            let new_pos = y2 * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 0 && y2 > 0 {
            x2 -= 1; y2 -= 1;
            let new_pos = y2 * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 0 && y2 < 7 {
            x2 -= 1; y2 += 1;
            let new_pos = y2 * 8 + x2;
            result = result.add_pos(new_pos);
            if !mask.empty_at(new_pos) { break; }
        }

        result
    }

    pub fn gen_bishop (pos: u32) -> Vec<BitBoard> {
        let mut perms = vec![BitBoard::new()];

        let x = pos % 8;
        let y = pos / 8;

        let (mut x2, mut y2) = (x, y);
        while x2 < 6 && y2 < 6 {
            x2 += 1; y2 += 1;
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y2 * 8)); }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 < 6 && y2 > 1 {
            x2 += 1; y2 -= 1;
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y2 * 8)); }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 1 && y2 > 1 {
            x2 -= 1; y2 -= 1;
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y2 * 8)); }
        }

        let (mut x2, mut y2) = (x, y);
        while x2 > 1 && y2 < 6 {
            x2 -= 1; y2 += 1;
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y2 * 8)); }
        }

        perms
    }

    pub fn gen_rook (pos: u32) -> Vec<BitBoard> {
        let mut perms = vec![BitBoard::new()];
        let (x, y) = (pos % 8, pos / 8);

        for y2 in 1..y {
            for perm in perms.clone() { perms.push(perm.add_pos(x + y2 * 8)); }
        }
        for y2 in (y+1)..7 {
            for perm in perms.clone() { perms.push(perm.add_pos(x + y2 * 8)); }
        }
        for x2 in 1..x {
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y * 8)); }
        }
        for x2 in (x+1)..7 {
            for perm in perms.clone() { perms.push(perm.add_pos(x2 + y * 8)); }
        }

        perms
    }
}