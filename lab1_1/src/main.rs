use std::env::args;
use clap::{arg, Parser};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// String to sluglify
    #[arg(short, long)]
    s: String,
}

const SUBS_I: &str = "àáâäæãåāăąçćčđďèéêëēėęěğǵḧîïíīįìıİłḿñńǹňôöòóœøōõőṕŕřßśšşșťțûüùúūǘůűųẃẍÿýžźż";
const SUBS_O: &str = "aaaaaaaaaacccddeeeeeeeegghiiiiiiiilmnnnnoooooooooprrsssssttuuuuuuuuuwxyyzzz";
const TRIMMER: char = '-';

fn find_custom(c: char) -> Option<i32> {
    //Custom find
    let mut counter = 0;
    for c_tmp in SUBS_I.chars() {
        if c_tmp == c {
            return Some(counter);
        }
        counter += 1;
    }
    None
}
fn conv(c: char) -> char {
    //Using the find() function is not allowed because the function retrive the index of first byte
    //of the char. In rust characters have not the same representation.
    // SUBS_O[SUBS_I.find(c)]
    let index = find_custom(c).unwrap() as usize;
    SUBS_O.chars().nth(index).unwrap()
}

/*
Un possibile algoritmo è il seguente:
- tutti i caratteri accentati vengono convertiti nell’equivalente non accentato
- tutto viene convertito in minuscolo
- ogni carattere che non sia in [a-z][0-9] viene convertito in “-”
- due “-” consecutivi non sono ammessi, solo il primo viene tenuto
- un “-” finale non è ammesso a meno che non sia l’unico carattere nella stringa
*/
pub fn slugify(s: &str) -> String {
    if s.len() == 0 {
        return  s.to_string();
    }
    //Allocating in heap the memory needed
    let mut result = String::with_capacity(s.len());
    let mut trimmer = true;
    for c in s.to_lowercase().chars() {

        let c_tmp = match c {
            c if c.is_ascii_alphanumeric() => { trimmer = true; Some(c) },
            c if SUBS_I.contains(c) => { trimmer = true; Some(conv(c)) },
            _ => if trimmer { trimmer = false; Some(TRIMMER) } else { None  }
        };
        if c_tmp.is_some() {
            result.push(c_tmp.unwrap());
        }
    }

    //Remove last '-'
    let end_test = result.pop().expect("Char not found");
    if end_test != '-'{
        result.push(end_test);
    }

    println!("{}", result);

    result
}

fn main(){
    let args = Args::parse();
    slugify(&args.s);
}

#[cfg(test)]
mod tests {
    use crate::slugify;

    //stringa con più di una parola
    #[test]
    fn t1() {
        let stringa = "Stringa con più di una parola";
        assert_eq!(slugify(stringa), "stringa-con-piu-di-una-parola");
    }

    //stringa con caratteri accentati
    #[test]
    fn t2() {
        let stringa = "ùùù";
        assert_eq!(slugify(stringa), "uuu");
    }

    //string vuota
    #[test]
    fn t3() {
        let stringa = "";
        assert_eq!(slugify(stringa), "");
    }

    //stringa con più spazi
    #[test]
    fn t4() {
        let stringa = "    ";
        assert_eq!(slugify(stringa), "");
    }

}
