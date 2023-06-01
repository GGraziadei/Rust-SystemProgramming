use clap::Parser;
use minesweeper::annotate;

/// minesweeper
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// number of rows
    #[arg(short, long)]
    rows: u8,

    /// number of columns
    #[arg(short, long)]
    cols: u8,

    /// minesweeper table
    #[arg(short, long)]
    table: String,
}

fn main() {
    let args = Args::parse();
    let mut input: Vec<&str> = Vec::new();  // vettore di slice di stringhe immutabili
    let mut first: &str = ""; // slice di stringa immutabile
    let mut last = args.table.as_str(); // &args.table é un prestito di stringa in scrittura

    for _ in 0..args.rows {
        (first, last) = last.split_at(args.cols as usize);
        input.push(first);
    }
    let output = annotate(&input);
    for o in output {
        println!("{}",o);
    }

}