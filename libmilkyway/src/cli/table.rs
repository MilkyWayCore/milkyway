use colored::Colorize;

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<&str>) -> Table {
        Table {
            headers: headers.into_iter().map(String::from).collect(),
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<&str>) {
        self.rows.push(row.into_iter().map(String::from).collect());
    }

    pub fn display(&self) {
        for header in &self.headers {
            print!("{:<15}", header.bold().underline().blue());
        }
        println!();
        
        for row in &self.rows {
            for cell in row {
                print!("{:<15}", cell.green());
            }
            println!();
        }
    }
}