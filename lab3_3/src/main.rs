use std::fs;
use std::io::{stdin, Write};
use std::process::exit;

use lab2_2::{Dir, File, FileSystem, FileType, Node, MatchResult};

fn main() {

    /*
    //let file_system = FileSystem::from(Dir::new("./src"));
    let mut file_system = FileSystem::from_dir("./src");
    println!("{:?}", file_system);

    file_system.mk_dir("./src/pippo");
    println!("{:?}", file_system);

    file_system.rm_dir("./src/pippo");
    println!("{:?}", file_system);

    let mut f1 = fs::File::create("./src/test.txt").unwrap();
    f1.write_all(b"Ciao Mondo");
    file_system.new_file("./src/test.txt", File::new("./src/test.txt"));
    println!("{:?}", file_system);

    let f2 : &mut File = file_system.get_file("./src/test.txt").unwrap();
    println!("{:?}", f2);

    file_system.rm_file("./src/test.txt");
    println!("{:?}", file_system);

    file_system.mk_dir("./src/pippo");
    fs::File::create("./src/pippo/test.txt");
    file_system.new_file("./src/pippo/test.txt", File::new("./src/pippo/test.txt"));
    println!("{:?}", file_system);
    file_system.rm_file("./src/pippo/test.txt");
    file_system.rm_dir("./src/pippo");

    let queries = ["larger:900", "content:cioa"];
    let result = file_system.search(&queries);
    println!("{:?}", result);
     */

    let mut fs_vec = Vec::<FileSystem>::new();

    loop {
        println!("{:?}" , fs_vec);
        println!("Digita uno dei seguenti comandi o premi invio per chiudere:");
        println!("new");
        println!("mk_dir <path>");
        println!("from_dir <path>");
        println!("rm_dir <path>");
        println!("new_file <path>");
        println!("rm_file <path>");
        println!("find <path>");
        println!("find_r <path>");
        let mut input_string = String::new();
        match stdin().read_line(&mut input_string).ok() {
            None => {
                println!("Something went wrong");
                return;
            }
            Some(_) => {
                let commands: Vec<&str> = input_string.split(" ").collect();
                println!("{:?}" , commands);
                match commands[0].trim_end() {
                    "" => { break; }
                    "0" | "new" => {
                        // fs = Some(FileSystem::new());
                        fs_vec.push(FileSystem::new());
                    }
                    "1" | "mkdir" | "mk_dir" => {
                        if commands.len() > 1 && !commands[1].is_empty() {
                            let dir_name = commands[1].trim_end();
                            fs_vec[0].mk_dir(dir_name);
                        } else {
                            println!("Errore: inserire nome corretto");
                        }
                    }
                    "2" | "cp" | "from_dir" => {
                        if commands.len() == 2 && !commands[1].is_empty() {
                            let path = commands[1].trim_end();
                            let new_fs = FileSystem::from_dir( path);
                            fs_vec.push(new_fs);

                        } else {
                            println!("Errore: inserire nome corretto");
                        }
                    }
                    "3" | "rmdir" | "rm_dir" => {
                        if commands.len() == 2 && !commands[1].is_empty() {
                            let dir_name = commands[1].trim_end();
                            fs_vec[0].rm_dir(dir_name);
                        } else {
                            println!("Errore: inserire nome corretto");
                        }
                    }
                    "4" | "touch" | "new_file" => {
                        if commands.len() >= 3 {
                            let file_name = commands[1].trim_end();
                            let content = commands[2..].join(" ").trim_end().replace("\"", "");
                            let file = File::new(file_name);
                            fs_vec[0].new_file(file_name, file);
                        }
                    }
                    "5" | "rm" | "rm_file" => {
                        if commands.len() == 2 {
                            let file_name = commands[1].trim_end();
                            fs_vec[0].rm_file(file_name);
                        }
                    }
                    "6" | "get_file" => {
                        if commands.len() == 2 {
                            let file_name = commands[1].trim_end();
                            fs_vec[0].get_file(file_name);
                        }
                    }
                    "7" | "find" => {
                        if commands.len() >= 2 {
                            let queries_vec = &commands[1..];
                            match fs_vec[0].search(queries_vec) {
                                Some(v) => {
                                    println!("Queries matchate:");
                                    for q in v.queries.iter() {
                                        println!("{:?}", q);
                                    }
                                    println!("Nodi trovati:");
                                    for n in v.nodes.iter() {
                                        println!("{:?}", n);
                                    }
                                }
                                None => {
                                    println!("Nessun risultato");
                                }
                            }
                        }

                    }
                    "print" => {}
                    "q" | "exit" => { exit(0); }
                    _ => {}
                }
            }
        }
    }
}
