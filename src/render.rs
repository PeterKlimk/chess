use super::*;
use std::fs::File;
use std::io::Write;

const BOARD_BASE: &str = include_str!("data/board");

pub fn debug_svg (state: &ChessState) {
    let mut board = String::new();
    board.push_str(BOARD_BASE);

    for &kind in Piece::kinds() {
        for pos in state.piece_bb[kind as usize].get_indices() {
            let color = state.color_at(pos).unwrap();
            let x = (pos % 8) * 100 + 10;
            let y = (7 - pos / 8) * 100 + 80;
            match color {
                Color::White => board.push_str(&format!("<text stroke-width=\"1.5\" stroke=\"black\" class=\"piece white\" x=\"{}\" y=\"{}\">{}</text>\n", x, y, kind.render(Color::Black))),
                Color::Black => board.push_str(&format!("<text class=\"piece black\" x=\"{}\" y=\"{}\">{}</text>\n", x, y, kind.render(Color::Black))),
            }
        }
    }
    
    board.push_str("</svg>");
    
    let mut file = File::create("render/render.svg").unwrap();
    file.write_all(board.as_bytes());
}