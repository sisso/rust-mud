const COLOR_RESET: &str = "\x1B[0m";

fn fg(fg: &str) -> String {
    format!("\x1B[38;5;{}m", fg)
}

fn bg(bg: &str) -> String {
    format!("\x1B[48;5;{}m", bg)
}

fn main() {
    for i in 0..256 {
        print!("{}{}{}\t", fg(&format!("{}", i)), i, COLOR_RESET);
        if i % 10 == 0 {
            println!()
        }
    }

    for i in 0..256 {
        print!("{}{}{}\t", bg(&format!("{}", i)), i, COLOR_RESET);
        if i % 10 == 0 {
            println!()
        }
    }
}
