pub fn parse_csv(data: &str) -> Vec<Vec<&str>> {
    let lines = data.split_terminator("\n").collect::<Vec<&str>>();
    let mut result = vec![];
    let mut max_columns = 0;

    // collect lines
    for line in lines {
        let mut row = vec![];
        let columns = line.split_terminator(",").collect::<Vec<&str>>();
        for r in columns {
            row.push(r);
        }

        max_columns = max_columns.max(row.len());

        result.push(row);
    }

    // normalize
    for i in 0..result.len() {
        for _ in result[i].len()..max_columns {
            result[i].push("");
        }
    }

    result
}

pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(name: String) -> Table {
        assert!(!name.is_empty());

        Table {
            name: name,
            columns: vec![],
            rows: vec![],
        }
    }

    pub fn get(&self, row: usize, column: &str) -> &str {
        match self.columns.iter().position(|col| col.as_str() == column) {
            Some(index) => self.rows[row][index].as_str(),
            None => panic!("could not find column {:?}", column),
        }
    }
}

pub fn strings_to_table(data: &Vec<Vec<&str>>) -> Result<Vec<Table>, String> {
    let mut result = vec![];
    let mut current = None;
    let mut parse_columns = false;
    for row in data {
        if row[0] == "#Table" {
            if let Some(current) = current.take() {
                result.push(current);
            }

            current = Some(Table::new(row[1].to_string()));
            parse_columns = true;
        } else if let Some(current) = &mut current {
            let mut row_str: Vec<String> = row.iter().map(|s| s.to_string()).collect();
            if parse_columns {
                parse_columns = false;

                // remove leading empty columns
                while row_str[row_str.len() - 1].is_empty() {
                    row_str.pop();
                }
                current.columns = row_str;
            } else {
                current.rows.push(row_str);
            }
        } else {
            return Err("No table header found".to_string());
        }
    }

    if let Some(current) = current.take() {
        result.push(current);
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::csv::{parse_csv, strings_to_table};

    #[test]
    fn test_parse_csv_with_simple() {
        let csv = r"#Table,Title
id,name
0,planet
1,armor";

        let data = parse_csv(csv);
        assert_eq!(data.len(), 4);
        for row in 0..data.len() {
            assert_eq!(data[row].len(), 2);
        }

        assert_eq!("#Table", data[0][0]);
        assert_eq!("Title", data[0][1]);
        assert_eq!("id", data[1][0]);
        assert_eq!("name", data[1][1]);
        assert_eq!("0", data[2][0]);
        assert_eq!("planet", data[2][1]);
        assert_eq!("1", data[3][0]);
        assert_eq!("armor", data[3][1]);
    }

    #[test]
    fn test_parse_csv_with_sample() {
        let csv = r###"#Table,Planets,,,,
code,label,prob weight,breath prob perc,habitability,mining
earth,Earth like,1,0.5,1,0.5
aqua,Aquaworld,1,0.5,1,0.5
jungle,Jungle,1,0.5,1,0.5
plain,Plains,1,0.5,1,0.5
alpine,Alpines,1,0.5,0.25,0.5
desert,Desert,10,0.5,0.25,0.5
barrent,Barrent,20,0,0.1,1
ice,Ice,10,0.5,0.1,1
lava,Lava,10,0,0.1,1
toxic,Toxic,10,0,0.1,1
,,,,,
#Table,Zones,,,,
code,require_breath,min_hab,min_mining,landing,
village,TRUE,0.25,0,0,
landpad,FALSE,0.1,0,2,
port,FALSE,0.25,0,4,
mining station,FALSE,0.1,0.5,0,
outpost,FALSE,0.1,0.5,0,
trade post,FALSE,0.25,0,0,
habitation,TRUE,0.25,0,0,
university,TRUE,0.25,0,0,
factory,FALSE,0.25,0,0,
land_zone,FALSE,0,0,1,
"###;
        let data = parse_csv(csv);
        assert_eq!(data.len(), 25);
    }

    #[test]
    fn test_strings_to_table() {
        let csv = r"#Table,Title
id,name
0,planet
1,armor
,
#Table,Other
id,name,desc
0,,that is a great thing
";

        let data = parse_csv(csv);
        let tables = strings_to_table(&data).unwrap();

        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "Title");
        assert_eq!(tables[0].columns, vec!["id", "name"]);
        assert_eq!(tables[0].rows[0], vec!["0", "planet", ""]);
        assert_eq!(tables[0].rows[1], vec!["1", "armor", ""]);

        assert_eq!(tables[0].get(0, "name"), "planet");
        assert_eq!(tables[0].get(1, "name"), "armor");

        assert_eq!(tables[1].name, "Other");
        assert_eq!(tables[1].columns, vec!["id", "name", "desc"]);
        assert_eq!(tables[1].rows[0], vec!["0", "", "that is a great thing"]);
        assert_eq!(tables[1].get(0, "name"), "");
    }
}