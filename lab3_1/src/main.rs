extern crate core;
use std::collections::LinkedList;
use std::thread;
use std::time::Instant;
use clap::Parser;
use itertools::Itertools;

use crate::operations::{OP, OpError};

mod operations;

const TUPLE_LEN: usize = 5;
const RESULT_EXPECTED : i32 = 10;

/// lab3_1
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// number of rows
    #[clap(short, long, value_delimiter=',' )]
    numbers: Vec<u8>
}

fn permutation_r (numbers : &Vec<u8>, v : &mut [u8; TUPLE_LEN], index : usize, results : &mut LinkedList<[u8; TUPLE_LEN]> ) {

    //Condizione di terminazione, nuova permutazione generata
    if index == TUPLE_LEN {
        results.push_back(v.clone());
        return;
    }

    for n in numbers.iter(){
        v[index] = n.clone();
        permutation_r(numbers, v, index + 1, results);
    }
}


fn op_permutation_r (operations : &Vec<operations::OP>, v : &mut [operations::OP ; 4], index : usize, results : &mut LinkedList<[operations::OP ; 4]> ) {

    //Condizione di terminazione, nuova permutazione generata
    if index == ( TUPLE_LEN - 1 ) {
        results.push_back(v.clone());
        return;
    }

    for n in operations.iter(){
        v[index] = n.clone();
        op_permutation_r(operations, v, index + 1, results);
    }
}



fn permutations(numbers : &Vec<u8>) -> Option<LinkedList<[u8; TUPLE_LEN]>> {
    let mut result = LinkedList::<[u8; TUPLE_LEN]>::new();
    let mut v : [u8; TUPLE_LEN] = [0;TUPLE_LEN];
    let mut index = 0;
    /*
        Disposizioni con ripetizione |n|^K ovvero numbers.lenght^5
        Ogni possibile soluzione contiene 5 elementi
    */

    permutation_r(numbers, &mut v, 0, &mut result);

    if result.is_empty(){
        return None;
    }

    Some(result)
}

fn op_permutations(operations : &Vec<operations::OP>) -> Option<LinkedList<[operations::OP ; 4]>> {
    let mut result = LinkedList::<[operations::OP ; 4]>::new() ;
    let mut v   = [operations::OP::SOMMA; TUPLE_LEN - 1];
    let mut index = 0;

    op_permutation_r(operations, &mut v, 0, &mut result);

    if result.is_empty(){
        return None;
    }

    Some(result)
}


fn check(v : &[u8; TUPLE_LEN] , op: &[operations::OP ; 4]) -> bool {
    let mut result : i32 = v[0].clone() as i32;
    let v_slice = &v[1..v.len()];

    for i in 0..op.len() {
        let var = v_slice[i] as i32;
        result = op[i].op(&result , &var).unwrap();
    }

    result == RESULT_EXPECTED
}

fn check_wrapper(tuples : &[(&[u8; 5], &[OP; 4])]) ->() {

    let mut r1 = Vec::<(&[u8;5] , &[operations::OP;4])>::new();
    for t in tuples.iter(){
        if check(&t.0, &t.1){
            r1.push(*t);
        }
    }
}

fn check_wrapper_vec(tuples :  Vec<&(&[u8;5] , &[operations::OP;4])> ) ->() {

    let mut r1 = Vec::<(&[u8;5] , &[operations::OP;4])>::new();
    for t in tuples.into_iter() {
        if check(&t.0, &t.1){
            r1.push(*t);
        }
    }

}

fn spawn_threads( num_threads : i32 , tuples : &Vec<(&[u8;5] , &[operations::OP;4])> ) -> (){
    thread::scope(|s| {
        for set in tuples.chunks((800000 / num_threads) as usize){
            s.spawn(move || {check_wrapper(set ) });
        }
    });
}

fn spawn_threads_interleaved( num_threads : i32 , tuples : &Vec<(&[u8;5] , &[operations::OP;4])> ) -> (){
    thread::scope(|s| {
        let mut powerset =  Vec::< Vec<&(&[u8;5] , &[operations::OP;4])> >::with_capacity(num_threads as usize);
        let mut index : usize = 0 ;

        let start = Instant::now();

        for i  in 0..num_threads {
            powerset.push(Vec::<& (&[u8;5] , &[operations::OP;4])>::with_capacity(tuples.len() / num_threads  as usize));
        }

        for t in tuples.iter() {
            powerset[index].push(t);
            index = ( index + 1 ) % num_threads as usize ;
        }

        let elapsed = start.elapsed();
        println!("Tempo impoiegato per la generazione dei chunck: {:?}" , elapsed);

        for v in powerset {
            s.spawn(move || {check_wrapper_vec(v ) });
        }
    });
}

fn main() {
    let args = Args::parse();
    let numbers = args.numbers;

    let permutations = permutations(&numbers).unwrap();
    println!("Permutations list<> : ");

    let mut operations = Vec::<operations::OP>::with_capacity(4);
    operations.push(operations::OP::SOMMA);
    operations.push(operations::OP::DIFFERENZA);
    operations.push(operations::OP::PRODOTTO);
    operations.push(operations::OP::DIVISIONE);

    println!("Op available <> : {:?} " , operations );

    let op = op_permutations(&operations).unwrap();

    /*
        [disposizioni con ripetizione input] = 5^5
        [disposizione con ripetizione operazioni] = 4^4
        space_len = [dimensione dello spazio da esplorare]
            = [disposizioni con ripetizione input] * [disposizione con ripetizione operazioni]
    */

    let mut space_len = 0;
    let mut tuples = Vec::<(&[u8;5] , &[operations::OP;4])>::new();

    for p in permutations.iter(){
        for o in op.iter(){
            space_len += 1;
            tuples.push((&p,&o));
        }
    }
    assert_eq!(space_len, 800000);

    let mut start = Instant::now();
    let mut r1 = Vec::<(&[u8;5] , &[operations::OP;4])>::new();
    for t in tuples.iter(){
        if check(&t.0, &t.1){
            r1.push(*t);
        }
    }
    let mut duration = start.elapsed();
    println!("Duration 1 thread : {:?}", duration );


    /*
        test 2 thread (attualmente eseguito su un dispositivo con 2 core fisici)
    */
    println!("------THREADS_SEQ-----");

    start = Instant::now();
    spawn_threads(2, &tuples);
    duration = start.elapsed();
    println!("Duration 2 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads(3, &tuples);
    duration = start.elapsed();
    println!("Duration 3 thread : {:?}", duration );


    start = Instant::now();
    spawn_threads(4, &tuples);
    duration = start.elapsed();
    println!("Duration 4 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads(10, &tuples);
    duration = start.elapsed();
    println!("Duration 10 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads(20, &tuples);
    duration = start.elapsed();
    println!("Duration 20 thread : {:?}", duration );

    println!("-----THREADS_SHUFFLE-----");

    start = Instant::now();
    spawn_threads_interleaved(1, &tuples);
    duration = start.elapsed();
    println!("Duration 1 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads_interleaved(2, &tuples);
    duration = start.elapsed();
    println!("Duration 2 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads_interleaved(4, &tuples);
    duration = start.elapsed();
    println!("Duration 4 thread : {:?}", duration );

    start = Instant::now();
    spawn_threads_interleaved(8, &tuples);
    duration = start.elapsed();
    println!("Duration 8 thread : {:?}", duration );

}
