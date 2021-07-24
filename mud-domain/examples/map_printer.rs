extern crate rand;

trait Rooms {
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn is_portal(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> bool;
    fn id(&self, x: usize, y: usize) -> usize;
}

fn print(rooms: &dyn Rooms) {
    let empty = "..";
    let portal_v = "||";
    let portal_h = "==";

    let mut buffer = String::new();
    for y in 0..rooms.height() {
        for x in 0..rooms.width() {
            let portal_n = if y == 0 {
                false
            } else {
                rooms.is_portal(x, y, x, y - 1)
            };

            buffer.push_str(empty);
            if portal_n {
                buffer.push_str(portal_v);
            } else {
                buffer.push_str(empty);
            }
        }

        buffer.push_str(empty);
        buffer.push_str("\n");

        for x in 0..rooms.width() {
            let portal_w = if x == 0 {
                false
            } else {
                rooms.is_portal(x, y, x - 1, y)
            };

            if portal_w {
                buffer.push_str(portal_h);
            } else {
                buffer.push_str(empty);
            }

            let id = rooms.id(x, y);
            let str = format!("{:02}", id);
            buffer.push_str(str.as_str());
        }

        buffer.push_str(empty);
        buffer.push_str("\n");
    }

    for _x in 0..(rooms.width() * 2 + 1) {
        buffer.push_str(empty);
    }

    buffer.push_str("\n");
    println!("{}", buffer);
}

fn main() {
    struct RoomsImpl;

    impl Rooms for RoomsImpl {
        fn height(&self) -> usize {
            3
        }

        fn width(&self) -> usize {
            3
        }

        fn is_portal(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> bool {
            let portals = vec![
                (0, 0, 1, 0),
                (0, 0, 0, 1),
                (1, 1, 1, 0),
                (0, 1, 0, 2),
                (0, 2, 1, 2),
                (1, 2, 2, 2),
                (2, 2, 2, 1),
                (2, 1, 2, 0),
            ];

            portals.contains(&(x0, y0, x1, y1)) || portals.contains(&(x1, y1, x0, y0))
        }

        fn id(&self, x: usize, y: usize) -> usize {
            x + y * 3
        }
    }

    let rooms = RoomsImpl {};
    print(&rooms);
}
