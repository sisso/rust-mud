pub struct PlotPoint {
    pub x: f32,
    pub y: f32,
    pub c: String,
}

pub struct PlotCfg {
    pub width: u32,
    pub height: u32,
    pub min_scale: f32,
}

pub fn plot_points(cfg: &PlotCfg, points: &Vec<PlotPoint>) -> Vec<Vec<String>> {
    let mut min_x = -cfg.min_scale;
    let mut max_x = cfg.min_scale ;
    let mut min_y = -cfg.min_scale ;
    let mut max_y = cfg.min_scale ;

    for point in points.iter() {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }

    let scale_x = (max_x - min_x) / cfg.width as f32;
    let scale_y = (max_y - min_y) / cfg.height as f32;


    let mut buffer = vec![];

    // draw empty map
    for i in 0..cfg.height {
        let mut b = Vec::new();
        for j in 0..cfg.width {
            b.push(".".to_string());
        }
        buffer.push(b)
    }

//    println!("Ranges X: {}/{} Y: {}/{}\nScale {}x{}", min_x, max_x, min_y, max_y, scale_x, scale_y);

    // draw objects
    for p in points.iter() {
        let mut x = ((p.x - min_x) / scale_x).round() as usize;
        let mut y = ((p.y - min_y) / scale_y).round() as usize;

//        println!("{} {} = {} {}", p.x, p.y, x, y);

        if x >= cfg.width as usize{
            x = cfg.width as usize - 1;
        }

        if y >= cfg.height as usize{
            y = cfg.height as usize - 1;
        }

        let line = buffer.get_mut(y).unwrap();
        line[x] = p.c.to_string();
    }

    buffer
}

pub fn append_right(buffer: &mut Vec<String>, mut other: Vec<String>) {
    let extra_lines = buffer.len().max(other.len());
    for i in buffer.len()..extra_lines {
        buffer.push("".to_string());
    }

    let buffer_max = buffer.iter().map(|i| i.len()).max().unwrap();
    for i in 0..other.len() {
        let size = buffer_max - buffer[i].len() + 1;
        let append = String::from_utf8(vec![b' '; size]).unwrap();
        buffer[i].push_str(append.as_str());
        buffer[i].push_str(other[i].as_str());
    }
}

pub fn mkstring(buffer: Vec<Vec<String>>) -> String {
    buffer.into_iter().map(|i| i.join("")).collect::<Vec<String>>().join("\n")
}

/// search in labels giving an inputs and return all matches, sorted by score
///
/// if a full match is found, no fuzy query is used
pub fn search_label(input: &str, labels: &Vec<&str>) -> Vec<usize> {
    labels.iter().enumerate().filter_map(|(i,s)| {
        if s.eq_ignore_ascii_case(input) {
            Some(i)
        } else {
            None
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use crate::utils::text::{plot_points, PlotCfg, PlotPoint, mkstring, append_right, search_label};

    const CFG: PlotCfg = PlotCfg {
        width: 6,
        height: 6,
        min_scale: 10.0,
    };

    #[test]
    pub fn test_search_label() {
        fn test_one(input: &str, strings: Vec<&str>, expected: Vec<usize>) {
            let result = search_label(input, &strings);
            assert_eq!(result, expected);
        }

        test_one("", vec![], vec![]);
        test_one("", vec!["one"], vec![]);
        test_one("one", vec!["one", "two"], vec![0]);
        test_one("two", vec!["one", "two"], vec![1]);
        test_one("one a", vec!["one a", "one b"], vec![0]);
//        test_one("one", vec!["one a", "one b"], vec![0, 1]);
//        test_one("one", vec!["ONE a", "ONE b"], vec![0, 1]);
//        test_one("one", vec!["ONE a", "ONE"], vec![1]);
    }

    #[test]
    pub fn test_plot_points_empty() {
        let buffer = plot_points(&CFG, &vec![]);

        assert_eq!(mkstring(buffer), r#"......
......
......
......
......
......"#)
    }

    #[test]
    pub fn test_plot_points_one_point() {
        let buffer = plot_points(&CFG, &vec![
            PlotPoint {
                x: 0.0,
                y: 0.0,
                c: 'X'.to_string()
            }
        ]);

        assert_eq!(mkstring(buffer), r#"......
......
......
...X..
......
......"#)
    }

    #[test]
    pub fn test_plot_points_extremes() {
        let buffer = plot_points(&CFG, &vec![
            PlotPoint {
                x: 0.0,
                y: 0.0,
                c: '0'.to_string()
            },
            PlotPoint {
                x: -10.0,
                y: -10.0,
                c: '1'.to_string()
            },
            PlotPoint {
                x: 10.0,
                y: -10.0,
                c: '2'.to_string()
            },
            PlotPoint {
                x: 10.0,
                y: 10.0,
                c: '3'.to_string()
            },
            PlotPoint {
                x: -10.0,
                y: 10.0,
                c: '4'.to_string()
            },
        ]);

        assert_eq!(mkstring(buffer), r#"1....2
......
......
...0..
......
4....3"#)
    }

    #[test]
    fn test_append_right() {
        let mut buffer = vec![
            "....".to_string(),
            "...".to_string(),
            "..".to_string(),
            "....".to_string(),
        ];

        append_right(&mut buffer, vec![
            "1 - One".to_string(),
            "2 - two".to_string(),
        ]);

        assert_eq!(".... 1 - One\n\
            ...  2 - two\n\
            ..\n\
            ....",
            buffer.join("\n")
        )
    }
}
