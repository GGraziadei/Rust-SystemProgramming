extern crate core;
use core::time;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::raw::pid_t;
use lab2_1::{ CircularBuffer, SensorData};
use fork::{daemon, Fork, fork};
use std::{fs, thread};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::thread::sleep;
use std::time::Duration;
use colored::*;
use std::fs::OpenOptions;
use std::process::{Command, exit};
use fcntl::{FcntlLockType, lock_file, unlock_file};
use std::collections::HashMap;
use serde::__private::de::Content::F32;

static BUFFER_FILE : &str = "BUFFER.bin";
static SLEEP_TIME_PRODUCER : Duration = time::Duration::from_millis(1000);
static SLEEP_TIME_CONSUMER : Duration = time::Duration::from_millis(10000);

//Linear search
fn find_max(v : &Vec<f32>) -> f32 {
    let mut max = f32::MIN;
    for val in v {
        if *val > max {
            max = *val;
        }
    }
    max
}
fn find_min(v : &Vec<f32>) -> f32 {
    let mut min = f32::MAX;
    for val in v {
        if *val < min {
            min = *val;
        }
    }
    min
}

fn lock_file_wrapper(file: &File) -> bool {
    match lock_file(file, None, Some(FcntlLockType::Write)) {
        Ok(true) => {
            println!("Lock acuired!");
            return true;
        },
        Ok(false) => {
            println!("Could not acquire lock!");
            thread::sleep(SLEEP_TIME_PRODUCER/10);
            return false;
        },
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn unlock_file_wrapper(file: &File) -> bool {
    match unlock_file(file, None) {
        Ok(true) => {
            println!("Lock successfully released");
            return true;
        },
        Ok(false) => {
            println!("Falied to release lock");
            return false;
        },
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn producer(buffer_file: &mut File){
    let process = "PRODUCER".bold().blue();
    let mut sensor_data_generator = SensorData::new();
    let mut sensor_data = sensor_data_generator(SensorData::random_measurements());
    let mut bytes = bincode::serialize( CircularBuffer::default().borrow() ).unwrap();

    let mut counter = 0;

    loop {
        println!("{} STARTED ", process);
        counter += 1;
        while ! lock_file_wrapper(&buffer_file){}
        //Sezione critica
        buffer_file.read(&mut bytes);
        let mut buffer: CircularBuffer =  bincode::deserialize(&bytes).unwrap();
        match buffer.store(sensor_data) {
            None => {
                println!("{} Buffer is full ! ", process);
            }
            Some(i) => {
                println!("{} : {:?} STORED @ index: {} ", process, sensor_data , i );
                sensor_data = sensor_data_generator(SensorData::random_measurements());
            }
        };
        bytes = bincode::serialize(&buffer).unwrap();

        buffer_file.rewind().unwrap();
        buffer_file.write_all(&bytes).unwrap();
        buffer_file.rewind().unwrap();

        while ! unlock_file_wrapper(&buffer_file){}
        thread::sleep(SLEEP_TIME_PRODUCER);

        if counter == 100 {
            println!("{} EXITED ", process);
            exit(0x0100);
        }
    }
}

fn consumer(buffer_file: &mut File){
    let process = "CONSUMER".bold().red();
    let mut bytes = bincode::serialize( CircularBuffer::default().borrow() ).unwrap();

    let mut counter = 0;
    let mut stats = HashMap::<u32, Vec<f32>>::new();

    loop {
        counter += 1;
        println!("{} STARTED ", process);
        while ! lock_file_wrapper(&buffer_file){}
        //Sezione critica
        buffer_file.read(&mut bytes);
        let mut buffer : CircularBuffer = bincode::deserialize(&bytes).unwrap();

        while !buffer.empty() {
            let sensor_data = *buffer.read().unwrap();
            let mut seq_number = sensor_data.get_seq();
            if stats.contains_key( &seq_number) {
                for value in sensor_data.get_values() {
                    stats.get_mut(&seq_number).unwrap().push(value);
                }
            }else{
                stats.insert(seq_number , sensor_data.get_values().to_vec() );
            }
        }

        bytes = bincode::serialize(&buffer).unwrap();

        buffer_file.rewind().unwrap();
        buffer_file.write_all(&bytes).unwrap();
        buffer_file.rewind().unwrap();

        while ! unlock_file_wrapper(&buffer_file){}

        //Print stats
        for entry in &stats {
            let seq_number = entry.0;
            let mut sensor_data = entry.1.clone();

            let entry_stats = (
                    //sensor_data.min(),
                    find_min(&sensor_data),
                    //sensor_data.max(),
                    find_max(&sensor_data),
                    sensor_data.clone().into_iter().sum::<f32>() / ( sensor_data.len() as f32 )
                );
            println!("{} - stats for SENSOR_{} : (min, max, avg) {:?} ", process, seq_number , entry_stats);
        }

        thread::sleep(SLEEP_TIME_CONSUMER);

        if counter == 10 {
            println!("{} EXITED ", process);
            exit(0x0100);
        }

    }
}

fn main() {

    fs::remove_file(BUFFER_FILE).unwrap();
    let mut buffer_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(BUFFER_FILE).unwrap();

    let buffer = CircularBuffer::default();
    println!("{:?}", buffer);
    let mut bytes : Vec<u8> = bincode::serialize(&buffer).unwrap();
    buffer_file.write_all(&bytes).unwrap();

    //Process manager creates 2 child - it waits the end of them


    //-->Producer
    let producer: pid_t = match fork(){
        Ok(Fork::Child) => {
            producer(&mut buffer_file);
            -1
        },
        Ok(Fork::Parent(pid)) => {
            pid
        },
        Err(e) => {
            panic!("{} - error during process creation " , e);
        }
    };
    println!("process {} producer started ", producer);

    //-->Consumer
    let consumer : pid_t = match fork(){
        Ok(Fork::Child) => {
            consumer(&mut buffer_file);
            -1
        },
        Ok(Fork::Parent(pid)) => {
            pid
        },
        Err(e) => {
            panic!("{} - error during process creation " , e);
        }
    };
    println!("process {} consumer started ", consumer);

    loop {

    }

    /*
    let mut producer = Command::new("producer").spawn().unwrap();
    let mut consumer = Command::new("consumer").spawn().unwrap();

    thread::sleep(SLEEP_TIME_CONSUMER * 10 );
    producer.kill();
    consumer.kill();
    */

}