use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::fmt;
use std::char;
use std::io::{self, BufRead};

use rand::Rng;

use lazy_static::lazy_static;

const PLAYER_COUNT: usize = 2;
const PIECE_TYPE_COUNT: usize = 6;

mod render;
mod magic;

use magic::MagicCache;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn from_letter(c: char) -> Option<Self> {
        match c {
            'w' => Some(Color::White),
            'b' => Some(Color::Black),
            _ => None,
        }
    }
}
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Piece {
    Pawn,
    Bishop,
    King,
    Queen,
    Rook,
    Knight
}

impl Piece {
    pub fn kinds() -> &'static [Piece] {
        const PIECES: [Piece; 6] = [
            Piece::Pawn, 
            Piece::Bishop, 
            Piece::King, 
            Piece::Queen, 
            Piece::Rook, 
            Piece::Knight
        ];

        &PIECES
    }

    pub fn from_letter(c: char) -> Option<Self> {
        match c {
            'k' => Some(Piece::King),
            'q' => Some(Piece::Queen),
            'n' => Some(Piece::Knight),
            'p' => Some(Piece::Pawn),
            'b' => Some(Piece::Bishop),
            'r' => Some(Piece::Rook),
            _ => None,
        }
    }

    pub fn render(&self, color: Color) -> char {
        match color {
            Color::White => {
                match self {
                    Piece::King => '♔',
                    Piece::Queen => '♕',
                    Piece::Rook => '♖',
                    Piece::Bishop => '♗',
                    Piece::Knight => '♘',
                    Piece::Pawn => '♙',
                }
            }

            Color::Black => {
                match self {
                    Piece::King => '♚',
                    Piece::Queen => '♛',
                    Piece::Rook => '♜',
                    Piece::Bishop => '♝',
                    Piece::Knight => '♞',
                    Piece::Pawn => '♟',
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct BitBoard(u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut n = self.0;
        let mut rows = Vec::new();

        for _ in 0..8 {
            let mut row = Vec::new();
            for _ in 0..8 {
                row.push(char::from_digit((n % 2) as u32, 10).unwrap());
                n = n / 2;
            }
            rows.push(row.iter().collect::<String>());
        }

        for row in rows.iter().rev() {
            write!(f, "{}", row)?;
            write!(f, "\n")?;
        }

        Ok(())
    }
}

struct IndexIterator {
    curr: u64,
    pos: u32,
}

impl Iterator for IndexIterator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let trail = self.curr.trailing_zeros() + 1;
        self.pos += trail;

        if self.pos >= 65 {
            None
        } else {
            self.curr >>= trail;
            Some(self.pos - 1)
        }
    }
}

impl BitBoard {
    fn new_empty() -> Self {
        Self(0)
    }

    fn empty_at (&self, pos: u32) -> bool {
        (*self & Self::from_pos(pos)).is_empty()
    }

    fn add_pos (&self, pos: u32) -> Self {
        *self | Self::from_pos(pos)
    }

    fn clear_pos(&self, pos: u32) -> Self {
        *self & Self::from_pos(pos).invert()
    }

    fn is_empty (&self) -> bool {
        self.0 == 0
    }

    fn not_empty(&self) -> bool {
        self.0 != 0
    }

    fn count(&self) -> u32 {
        self.0.count_ones()
    }

    fn invert(&self) -> Self {
        Self(!self.0)
    }

    fn from_pos (pos: u32) -> Self {
        Self(1 << pos)
    }

    fn get_indices (&self) -> IndexIterator {
        IndexIterator {
            pos: 0,
            curr: self.0,
        }
    }

    fn solo_pos (&self) -> u32 {
        self.0.trailing_zeros()
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0)
    }
}

pub struct ChessState {
    pub active: Color,
    pub piece_bb: [BitBoard; PIECE_TYPE_COUNT],
    pub player_bb: [BitBoard; PLAYER_COUNT],
    pub castle_ks: [bool; PLAYER_COUNT],
    pub castle_qs: [bool; PLAYER_COUNT],
    pub en_passant: Option<BitBoard>,
    pub move_rule: u32,
}

struct Cache {
    knight_moves: Vec<BitBoard>,
    king_moves: Vec<BitBoard>,
}

impl Cache {
    fn new () -> Cache {
        let mut knight_moves = Vec::new();
        for pos in 0..64 {
            let x = pos % 8;
            let y = pos / 8;
            
            let mut bb = BitBoard::new_empty();

            if x >= 2 {
                if y < 7 { bb = bb.add_pos((y + 1) * 8 + (x - 2)); }
                if y > 0 { bb = bb.add_pos((y - 1) * 8 + (x - 2)); }
            }

            if x <= 5 {
                if y < 7 { bb = bb.add_pos((y + 1) * 8 + (x + 2)); }
                if y > 0 { bb = bb.add_pos((y - 1) * 8 + (x + 2)); }
            }

            if y <= 5 {
                if x < 7 { bb = bb.add_pos((y + 2) * 8 + (x + 1)); }
                if x > 0 { bb = bb.add_pos((y + 2) * 8 + (x - 1)); }
            }

            if y >= 2 {
                if x < 7 { bb = bb.add_pos((y - 2) * 8 + (x + 1)); }
                if x > 0 { bb = bb.add_pos((y - 2) * 8 + (x - 1)); }
            }

            knight_moves.push(bb);
        }

        let mut king_moves = Vec::new();
        for pos in 0..64 {
            let x = pos % 8;
            let y = pos / 8;

            let mut bb = BitBoard::new_empty();
            if x > 0 {
                bb = bb.add_pos (pos - 1);

                if y > 0 {
                    bb = bb.add_pos (pos - 1 - 8);
                }

                if y < 7 {
                    bb = bb.add_pos(pos - 1 + 8)
                }
            }

            if x < 7 {
                bb = bb.add_pos (pos + 1);

                if y > 0 {
                    bb = bb.add_pos (pos + 1 - 8);
                }

                if y < 7 {
                    bb = bb.add_pos (pos + 1 + 8);
                }
            }

            if y > 0 {
                bb = bb.add_pos (pos - 8);
            }

            if y < 7 {
                bb = bb.add_pos (pos + 8);
            }

            king_moves.push(bb);
        }

        Cache { king_moves, knight_moves }
    }

    fn knight_moves (&self, pos: u32) -> BitBoard {
        self.knight_moves[pos as usize]
    }

    fn king_moves(&self, pos: u32) -> BitBoard {
        self.king_moves[pos as usize]
    }
}

lazy_static! {
    static ref cache: Cache = Cache::new();
    static ref magic_cache: MagicCache = MagicCache::new();
}

impl ChessState {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    fn from_fen (fen: &str) -> Self {
        let mut player_bb = [BitBoard::new_empty(); PLAYER_COUNT];
        let mut piece_bb = [BitBoard::new_empty(); PIECE_TYPE_COUNT];        

        let mut chars = fen.chars();
        let mut i = 0;

        loop {
            let c = chars.next().expect("Invalid FEN.");

            if c == '/' {
                continue;
            } else if c == ' ' {
                break;
            } else if c.is_ascii_digit() {
                i += c.to_digit(10).unwrap();
                continue;
            }

            let piece = Piece::from_letter(
                c.to_ascii_lowercase())
                .expect("Invalid FEN.");
            
            let color = if c.is_uppercase() { Color::White } else { Color::Black };

            let pos = 8 * (8 - (i / 8) - 1) + i % 8;

            let pos_bb = BitBoard::from_pos(pos);

            player_bb[color as usize] |= pos_bb;
            piece_bb[piece as usize] |= pos_bb;
            i += 1;
        }

        let active = match chars.next().expect("Invalid FEN.") {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!("Invalid FEN."),
        };

        chars.next().expect("Invalid FEN.");

        let mut castle_ks = [false; PLAYER_COUNT];
        let mut castle_qs = [false; PLAYER_COUNT];

        loop {
            let c = chars.next().expect("Invalid FEN.");
            match c {
                'k' => castle_ks[Color::Black as usize] = true,
                'K' => castle_ks[Color::White as usize] = true,
                'q' => castle_qs[Color::Black as usize] = true,
                'Q' => castle_qs[Color::White as usize] = true,
                '-' => continue,
                ' '=> break,
                _ => panic!("Invalid FEN."),
            }
        }

        let c = chars.next().expect("Invalid FEN.");
        let en_passant = match c {
            '-' => {
                None
            }

            r => {
                let f = chars.next().expect("Invalid FEN.");
                Some(BitBoard::from_pos(algebra_to_pos(r, f)))
            },
        };

        chars.next().expect("Invalid FEN.");

        let move_rule = chars.take_while(|&c| c != ' ')
            .collect::<String>()
            .parse::<u32>()
            .expect("Invalid FEN.");

        Self {
            active,
            piece_bb,
            player_bb,
            castle_ks,
            castle_qs,
            en_passant,
            move_rule
        }
    } 

    fn color_at (&self, pos: u32) -> Option<Color> {
        if !(self.player_bb[Color::White as usize].empty_at(pos)) {
            Some(Color::White)
        } else if !(self.player_bb[Color::Black as usize].empty_at(pos)) {
            Some(Color::Black)
        } else {
            None
        }
    }

    fn legal_moves (&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let targetable = self.player_bb[self.active as usize].invert();
        let occupied = self.player_bb[0] | self.player_bb[1];
        let player = self.player_bb[self.active as usize];
        let enemy = self.player_bb[self.active.opposite() as usize];
        
        //KNIGHT MOVES
        let bb = self.piece_bb[Piece::Knight as usize] & player;

        for index in bb.get_indices() {
            for target in (cache.knight_moves(index) & targetable).get_indices() {
                moves.push(Move::new(Piece::Knight, index, target));
            }
        }

        //PAWN MOVES
        let double_row = match self.active {
            Color::White => 1,
            Color::Black => 6,
        };

        let end_row = match self.active {
            Color::White => 7,
            Color::Black => 0,
        };

        let bb = self.piece_bb[Piece::Pawn as usize] & player;

        for index in bb.get_indices() {
            let y = index / 8;
            let x = index % 8;

            if y != end_row {

                //left attack
                if x != 0 {
                    let new_pos = match self.active {
                        Color::White => index + 8 - 1,
                        Color::Black => index - 8 - 1,
                    };

                    if !enemy.empty_at(new_pos) {
                        moves.push(Move::new(Piece::Pawn, index, new_pos));
                    }
                }

                //right attack
                if x != 7 {
                    let new_pos = match self.active {
                        Color::White => index + 8 + 1,
                        Color::Black => index - 8 + 1,
                    };

                    if !enemy.empty_at(new_pos) {
                        moves.push(Move::new(Piece::Pawn, index, new_pos));
                    }
                }

                let new_pos = match self.active {
                    Color::White => index + 8,
                    Color::Black => index - 8,
                };

                //move and double move
                if occupied.empty_at(new_pos) {
                    moves.push(Move::new(Piece::Pawn, index, new_pos));

                    if y == double_row {
                        let double_pos = match self.active {
                            Color::White => index + 16,
                            Color::Black => index - 16,
                        };

                        if occupied.empty_at(double_pos) {
                            moves.push(Move::new(Piece::Pawn, index, double_pos));
                        }
                    }
                }
            }
        }

        //BISHOP MOVES
        let bb = self.piece_bb[Piece::Bishop as usize] & player;
        for index in bb.get_indices() {
            let possible = magic_cache.bishop_moves(index, occupied);
            for target in (possible & targetable).get_indices() {
                moves.push(Move::new(Piece::Bishop, index, target));
            }
        }

        //QUEEN MOVES
        let bb = self.piece_bb[Piece::Queen as usize] & player;
        for index in bb.get_indices() {
            let possible = magic_cache.bishop_moves(index, occupied) | magic_cache.rook_moves(index, occupied);
            for target in (possible & targetable).get_indices() {
                moves.push(Move::new(Piece::Queen, index, target));
            }
        }

        //ROOK MOVES
        let bb = self.piece_bb[Piece::Rook as usize] & player;
        for index in bb.get_indices() {
            let possible = magic_cache.rook_moves(index, occupied);
            for target in (possible & targetable).get_indices() {
                moves.push(Move::new(Piece::Rook, index, target));
            }
        }

        //KING MOVES
        let bb = self.piece_bb[Piece::King as usize] & player;
        let king_pos = bb.solo_pos();

        let possible = cache.king_moves(king_pos) & targetable;
        for target in possible.get_indices() {
            moves.push(Move::new(Piece::King, king_pos, target));
        }

        moves
    }

    fn apply_move (&mut self, action: Move) {
        self.player_bb[self.active.opposite() as usize] = self.player_bb[self.active.opposite() as usize].clear_pos(action.dest);
        for &piece in Piece::kinds() {
            println!("{}", self.piece_bb[piece as usize].empty_at(action.dest));
            self.piece_bb[piece as usize] = self.piece_bb[piece as usize].clear_pos(action.dest);
            println!("{}", self.piece_bb[piece as usize].empty_at(action.dest));
        }

        self.player_bb[self.active as usize] = self.player_bb[self.active as usize]
            .clear_pos(action.origin).add_pos(action.dest);
        self.piece_bb[action.piece as usize] = self.piece_bb[action.piece as usize]
            .clear_pos(action.origin).add_pos(action.dest);
            
        self.active = self.active.opposite();
    }
}

#[derive(Copy, Clone)]
struct Move {
    piece: Piece,
    origin: u32,
    dest: u32,
}


impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {} -> {}", self.piece, pos_to_algebra(self.origin), pos_to_algebra(self.dest))
    }
}

impl Move {
    fn new(piece: Piece, origin: u32, dest: u32) -> Self {
        Self { piece, origin, dest }
    }
}

impl fmt::Display for ChessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut board = [' '; 64];

        for pos in 0..64 {
            let x = pos % 8;
            let y = pos / 8;
            if x % 2 != y % 2 {
                board[pos] = '■';
            } else {
                board[pos] = '⮻';
            }
        }

        for &kind in Piece::kinds() {
            for pos in self.piece_bb[kind as usize].get_indices() {
                let color = self.color_at(pos).unwrap();
                board[pos as usize] = kind.render(color);
            }
        }

        for chunk in board.chunks(8).rev() {
            writeln!(f, "{}", chunk.iter().collect::<String>())?;
        }
        Ok(())
    }
}

fn algebra_to_pos(rank: char, file: char) -> u32 {
    let rank_bin = match rank {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => panic!("Invalid position.") 
    };

    let file_bin = file.to_digit(10).expect("Invalid position.") - 1;

    file_bin * 8 + rank_bin
}

fn pos_to_algebra(pos: u32) -> String {
    let x = pos % 8;
    let y = pos / 8;

    let mut algebra = String::with_capacity(2);

    algebra.push(match x {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => unreachable!(),
    });

    algebra.push(match y {
        0 => '1',
        1 => '2',
        2 => '3',
        3 => '4',
        4 => '5',
        5 => '6',
        6 => '7',
        7 => '8',
        _ => panic!("Invalid pos."),
    });

    algebra
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut state = ChessState::default();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        let moves = state.legal_moves();
        for (i, action) in moves.iter().enumerate() {
            println!("{}: {}", i, action);
        }

        render::debug_svg(&state);

        let input = lines.next().unwrap().unwrap();
        let target_move = if input == "" {
            rng.gen_range(0, moves.len())
        } else {
            input.parse::<usize>().unwrap()
        };
        
        state.apply_move(moves[target_move]);
    }
}
