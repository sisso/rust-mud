use termion;

fn main() {
    let color_red = termion::color::Fg(termion::color::Red);
    println!(
        "{}Red{} b",
        color_red,
        termion::color::Fg(termion::color::Reset)
    );
    println!(
        "{}Red{} a",
        termion::color::Fg(termion::color::Green),
        termion::color::Fg(termion::color::Reset)
    );
    println!(
        "{}Red{} c",
        termion::color::Fg(termion::color::Blue),
        termion::color::Fg(termion::color::Reset)
    );
    println!("No color");
}
