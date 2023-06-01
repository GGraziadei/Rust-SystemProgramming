use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/*
Buffer circolare di strutture SensorData salvati su File
*/
#[repr(C)]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
//PartialEq: sono presenti dei campi float
pub struct SensorData {
    seq: u32, // sequenza letture
    values: [f32; 10],
    timestamp: u32
}

impl  SensorData {

    pub fn random_measurements() -> [f32; 10] {
        let mut array: [f32; 10] = [0.;10];
        for i in 0..10{
            array[i] = rand::random::<f32>() * 100.;
        }
        array
    }

    pub fn new() ->  impl FnMut([f32; 10]) -> Self {
        let mut seq = 0; //Closure: sequence id
        move |values| {
            seq += 1;
            let result = Self{
                seq,
                values,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u32
            };

            if seq >= 8 { //Suppongo la presenza di 8 differenti sensori
                seq = 0;
            }

            result
        }
    }

    pub fn get_seq(&self) -> u32 {
        self.seq
    }

    pub fn get_values(&self) -> [f32 ; 10] {self.values}

}

/*
Implementazione buffer circolare Cabodi - Camurati
Puntatori e strututre dati dinamiche
Pagina 182
 */
#[derive( Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub struct CircularBuffer {
    tail: usize,
    head: usize,
    counter: u8,
    buffer: [SensorData; 20]
}

impl Default for CircularBuffer{
    fn default() -> Self {
        Self{
            tail: 0,
            head: 20,
            counter: 0,
            buffer: [SensorData::default(); 20] //Default value for buffer
        }
    }
}

impl CircularBuffer{

    /*
        Consizione empty: l'elemento tail sovrapposto a cursore head
    */
    pub fn empty(&self) -> bool {
        //self.head % self.buffer.len() == self.tail
        self.counter == 0
    }

    /*
        Consizione full: l'elemento tail precede l'elemento head
    */
    pub fn full(&self) -> bool {
        //(self.tail + 1) % self.buffer.len() == self.head
        //+1 in quanto l'ultimo elemento del vettore non può essere utilizzato
        self.counter == (self.buffer.len() ) as u8
    }

    pub fn store(&mut self , sensor_data : SensorData ) -> Option<usize> {

        if self.full(){
            return None;
        }
        let index = self.tail;
        self.buffer[self.tail] = sensor_data;
        self.tail = ( index + 1 ) % self.buffer.len();
        self.counter += 1;
        Some(index)
    }

    pub fn read_from_index(&mut self , index : &usize ) -> Box<SensorData> {
        if *index >= self.buffer.len(){
            panic!("Index out of bound");
        }
        let sensor_data = Box::new(self.buffer[*index ]);
        self.buffer[*index] = SensorData::default();
        sensor_data
    }

    pub fn read(&mut self ) -> Option<Box<SensorData>> {

        if self.empty() {
            return None;
        }

        self.head = self.head % self.buffer.len();
        let sensor_data = Box::new(self.buffer[self.head ]);
        self.buffer[self.head  ] = SensorData::default();
        self.head += 1;
        self.counter -= 1;
        Some(sensor_data)
    }

}


