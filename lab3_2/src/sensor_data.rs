use std::time::{SystemTime, UNIX_EPOCH};

#[repr(C)]
#[derive(Debug, Default, PartialEq, Copy, Clone)]
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

    pub fn get_seq(&self) -> u32 { self.seq }

    pub fn get_values(&self) -> [f32 ; 10] { self.values }

}