use termion;

pub fn debug(location: &str, msg: &str) {
    println!("{color}DEBUG {location} - {msg}{reset}",
             color=termion::color::Fg(termion::color::Green),
             location=location,
             msg=msg,
             reset=termion::color::Fg(termion::color::Reset));
}

pub fn info(location: &str, msg: &str) {
    println!("{color}INFO {location} - {msg}{reset}",
             color=termion::color::Fg(termion::color::White),
             location=location,
             msg=msg,
             reset=termion::color::Fg(termion::color::Reset));
}

pub fn warn(location: &str, msg: &str) {
    println!("{color}WARN {location} - {msg}{reset}",
             color=termion::color::Fg(termion::color::Yellow),
             location=location,
             msg=msg,
             reset=termion::color::Fg(termion::color::Reset));
}
