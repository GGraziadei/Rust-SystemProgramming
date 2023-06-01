/// author: GG
/// little endian parsing

#[no_std]

use std::env::args;
use clap::{arg, Parser};
use std::{fs, mem};
use std::fs::File;
use std::io::Read;
use std::os::raw::c_float;
use std::ptr::null;
use crate::CData::{MessageStruct, ValueStruct};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// Filename
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    num: i32,
    #[arg(short, long)]
    size: i32,
}

#[ repr(C) ]
#[derive(Default, Debug)]
struct ValueStructT{
    t: i32,
    val: f32,
    timestamp: i32
}

#[ repr(C) ]
#[derive(Default, Debug)]
struct MValueStructT{
    t: i32,
    val: [f32; 10],
    timestamp: i32
}

#[ repr(C) ]
#[derive(Default, Debug)]
struct MessageStructT{
    t: i32,
    val: [char; 21]
}

enum CData{
    MessageStruct(MessageStructT),
    MValueStruct(MValueStructT),
    ValueStruct(ValueStructT)
}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = match File::open(&filename) {
        Ok(file) => file,
        Err(_) => panic!("File not found")
    };
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer.reverse();

    buffer
}

fn byte_4_i32(sub: &mut Vec<u8>) -> i32{
    // byte buffer array
    let mut byte_4 = [0,0,0,0];

    for j in 0..3{ //Scrittura del primo byte
        byte_4[j] = sub.pop().expect("Invalid byte");
    }
    i32::from_le_bytes(byte_4)
}

fn byte_4_f32(sub: &mut Vec<u8>) -> f32{
    // byte buffer array
    let mut byte_4 = [0,0,0,0];

    for j in 0..3{ //Scrittura del primo byte
        byte_4[j] = sub.pop().expect("Invalid byte");
    }
    c_float::from_le_bytes(byte_4)
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let mut bytes : Vec<u8> = get_file_as_byte_vec(&args.file);
    println!("{:?}", bytes);

    //Vec di enum
    let mut c_data_vec = Vec::<CData>::with_capacity(args.num as usize);

    for block in 0..args.num{ //Iterazione sul numero di elementi salvati nel file da leggere
        //Blocco CData sul quale opero
        println!("<Cdata> {:?}", block);
        let offset = block * args.size;
        let mut sub_vec =  Vec::from (&bytes[offset as usize..(offset+args.size) as usize]);

        let type_var = byte_4_i32(&mut sub_vec);

        match type_var {
            0 => {

                let t = byte_4_i32(&mut sub_vec);
                let val: f32 = byte_4_f32(&mut sub_vec);
                let timestamp = byte_4_i32(&mut sub_vec);

                let value_struct = ValueStructT {
                    t,
                    val,
                    timestamp,
                };

                println!("<ValueStruct> {:?}", value_struct);

            },
            1 => {
                println!("<MValueStruct> ");
            },
            2 => {
                println!("<MessageStruct> ");
            }
            _ => {}
        }
    }
}
