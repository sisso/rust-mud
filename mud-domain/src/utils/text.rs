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

    println!("Ranges X: {}/{} Y: {}/{}\nScale {}x{}", min_x, max_x, min_y, max_y, scale_x, scale_y);

    // draw objects
    for p in points.iter() {
        let mut x = ((p.x - min_x) / scale_x).round() as usize;
        let mut y = ((p.y - min_y) / scale_y).round() as usize;

        println!("{} {} = {} {}", p.x, p.y, x, y);

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

pub fn mkstring(buffer: Vec<Vec<String>>) -> String {
    buffer.into_iter().map(|i| i.join("")).collect::<Vec<String>>().join("\n")
}

#[cfg(test)]
mod tests {
    use crate::utils::text::{plot_points, PlotCfg, PlotPoint, mkstring};

    const CFG: PlotCfg = PlotCfg {
        width: 6,
        height: 6,
        min_scale: 10.0,
    };

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

}
