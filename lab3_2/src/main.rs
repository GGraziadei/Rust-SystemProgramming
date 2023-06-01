extern crate core;

use core::time;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use colored::*;

use ring_buffer::CircularBufferError;
use ring_buffer::RingBuffer;
use sensor_data::SensorData;

mod ring_buffer;
mod sensor_data;

const SLEEP_TIME_PRODUCER : Duration = time::Duration::from_millis(1000);
//const SLEEP_TIME_CONSUMER : Duration = time::Duration::from_millis(10000);

const BUFFER_MAX_SIZE : usize = 10;

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

fn producer(buffer : Arc<RingBuffer<SensorData>>){

    let process = "PRODUCER".bold().blue();

    let mut sensor_data_generator = SensorData::new();
    let mut sensor_data = sensor_data_generator(SensorData::random_measurements());

    loop {
        println!("{} STARTED", process);
        let result = buffer.write( sensor_data);
        if result.is_err(){
            println!("Buffer is full! ");
            /*
                QUesta sleep non risulta necessaria. tuttavia la inserisco per fornire il
                tempo al buffer di stdout di stampare correttamente le statistiche per tutti
                i sensori
            */
            //thread::sleep(SLEEP_TIME_PRODUCER);
        }
        sensor_data = sensor_data_generator(SensorData::random_measurements());
    }

}

fn producer_with_sender( sender : SyncSender<SensorData>){

    let process = "PRODUCER".bold().blue();

    let mut sensor_data_generator = SensorData::new();

    loop {
        println!("{} started" , process);
        sender.send(sensor_data_generator(SensorData::random_measurements()) ) ;
    }

}

fn print_stats( stats : &HashMap<u32 , Vec<f32>> ) -> (){
    for entry in stats.iter() {
        let seq_number = entry.0;
        let mut sensor_data = entry.1;
        let entry_stats = (
            //sensor_data.min(),
            find_min(sensor_data),
            //sensor_data.max(),
            find_max(sensor_data),
            //sensor_data.clone().into_iter().sum::<f32>() / ( sensor_data.len() as f32 )
            sensor_data.iter().sum::<f32>() / (sensor_data.len() as f32 )
        );
        println!("=> stats for SENSOR_{} : (min, max, avg) {:?} ", seq_number , entry_stats);
    }
}

fn consumer(buffer : Arc<RingBuffer<SensorData>>){
    let process = "CONSUMER".bold().red();

    let mut stats = HashMap::<u32, Vec<f32>>::new();

    loop {
        println!("{} STARTED ", process);

        let result =  buffer.read();

        if( result.is_some() ){
            let sensor_data = result.unwrap();
            let seq_number = sensor_data.get_seq();
            let data = sensor_data.get_values();

            stats.entry(seq_number)
                .and_modify( | v | { data.iter().for_each( |val| { v.push(val.clone())} )})
                .or_insert(Vec::from(data));

            print_stats(&stats);
        }

        //thread::sleep(SLEEP_TIME_CONSUMER);

    }
}

fn consumer_with_receiver(receiver : Receiver<SensorData>) -> ()
{
    let process = "CONSUMER".bold().red();
    let mut stats = HashMap::<u32, Vec<f32>>::new();
    loop {
        println!("{} STARTED ", process);
        while  let Ok(sensor_data) = receiver.recv() {
            let seq_number = sensor_data.get_seq();
            let data = sensor_data.get_values();

            stats.entry(seq_number)
                .and_modify( | v | { data.iter().for_each( |val| { v.push(val.clone())} )})
                .or_insert(Vec::from(data));

            print_stats(&stats);
        }
    }
}

fn main() {

    /*
        Versione1 - Utilizzo del costrutto mutex + cv per sincronizzare producer e consumer

    let mut producer_buffer = Arc::new(RingBuffer::<SensorData>::new());
    let mut consumer_buffer = producer_buffer.clone();
    //Process manager creates 2 child threads - it waits the end of them

    //-->Producer
    let producer = thread::spawn(move || {producer( producer_buffer )});
    //-->Consumer
    let consumer = thread::spawn(move || {consumer(consumer_buffer)});

     */

    /* Versione2 - Utilizzo un channel per permettere la comunicazioen tra producer e consumer
    let (sender, receiver) = std::sync::mpsc::channel::<SensorData>();
    */

    //Versione3 - Utilizzo un sync_channel per limitare la dimensione del buffer
    let (sender , receiver ) = std::sync::mpsc::sync_channel::<SensorData>(BUFFER_MAX_SIZE);

    //-->Producer
    let producer = thread::spawn(move || {producer_with_sender(sender)});
    //-->Consumer
    let consumer = thread::spawn(move || { consumer_with_receiver(receiver)} );

    producer.join();
    consumer.join();
}