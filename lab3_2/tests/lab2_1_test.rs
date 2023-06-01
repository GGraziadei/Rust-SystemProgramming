use std::fs::File;
use std::io::{Read, Write};
use bincode::Error;
use lab2_1::{ CircularBuffer, SensorData};

#[test]
fn sensor_data_sequence_generation(){
    //Il generatore deve essere dichiarato come mutabile al fine di incrementare l'indice di seq
    let mut sensor_data_generator = SensorData::new();

    let sensor_data_1 = sensor_data_generator([0.,0.,0.,0.,0.,0.,0.,0.,0.,0.]);
    assert_eq!(1, sensor_data_1.get_seq());
    let sensor_data_2 = sensor_data_generator([0.,0.,0.,0.,0.,0.,0.,0.,0.,0.]);
    assert_eq!(2, sensor_data_2.get_seq());
    let sensor_data_3 = sensor_data_generator([0.,0.,0.,0.,0.,0.,0.,0.,0.,0.]);
    assert_eq!(3, sensor_data_3.get_seq());
}

#[test]
fn serialize_deserialize_struct(){

    let buffer_file_name = "BUFFER_TEST.bin";
    let mut buffer = CircularBuffer::default();

    println!("{:?}", buffer);
    let mut buffer_file = match File::create(buffer_file_name ) {
        Ok(file) => { file }
        Err(_) => { panic!("Impossibile aprire il file"); }
    };

    let mut bytes : Vec<u8> = bincode::serialize(&buffer).unwrap();
    buffer_file.write_all(&bytes).unwrap();

    buffer_file.read(&mut bytes);
    let buffer_read : CircularBuffer = bincode::deserialize(&bytes).unwrap();

    assert_eq!(buffer_read, buffer);
}


#[test]
fn serialize_deserialize_struct_d1(){

    let mut sensor_data_generator = SensorData::new();

    let buffer_file_name = "BUFFER_TEST.bin";
    let mut buffer = CircularBuffer::default();

    let sensor_data_delta = sensor_data_generator([0.,1.,54.43,0.,0.,0.,0.,0.,-0.42,0.]);
    let index = buffer.store(sensor_data_delta.clone()).unwrap();

    println!("{:?}", buffer);
    let mut buffer_file = match File::create(buffer_file_name ) {
        Ok(file) => { file }
        Err(_) => { panic!("Impossibile aprire il file"); }
    };

    let mut bytes : Vec<u8> = bincode::serialize(&buffer).unwrap();
    buffer_file.write_all(&bytes).unwrap();

    buffer_file.read(&mut bytes);
    let buffer_read : CircularBuffer = bincode::deserialize(&bytes).unwrap();

    assert_eq!(buffer_read, buffer);

    let sensor_data_read = *buffer.read_from_index(&index);

    assert_eq!(sensor_data_read, sensor_data_delta);
}

#[test]
fn serialize_deserialize_struct_store(){

    let mut sensor_data_generator = SensorData::new();

    let buffer_file_name = "BUFFER_TEST.bin";
    let mut buffer = CircularBuffer::default();

    let sensor_data_delta = sensor_data_generator([0.,1.,54.43,0.,0.,0.,0.,0.,-0.42,0.]);
    let index = buffer.store(sensor_data_delta.clone()).unwrap();

    println!("{:?}", buffer);
    let mut buffer_file = match File::create(buffer_file_name ) {
        Ok(file) => { file }
        Err(_) => { panic!("Impossibile aprire il file"); }
    };

    let mut bytes : Vec<u8> = bincode::serialize(&buffer).unwrap();
    buffer_file.write_all(&bytes).unwrap();

    buffer_file.read(&mut bytes);
    let buffer_read : CircularBuffer = bincode::deserialize(&bytes).unwrap();

    assert_eq!(buffer_read, buffer);

    let sensor_data_read = *buffer.read().unwrap();

    println!("{:?}", buffer);

    assert_eq!(sensor_data_read, sensor_data_delta);
}

#[test]
fn sensor_data_sequence_generation_random(){
    //Il generatore deve essere dichiarato come mutabile al fine di incrementare l'indice di seq
    let mut sensor_data_generator = SensorData::new();

    let sensor_data_1 = sensor_data_generator(SensorData::random_measurements());
    assert_eq!(1, sensor_data_1.get_seq());
    println!("{:?}", sensor_data_1);
    let sensor_data_2 = sensor_data_generator(SensorData::random_measurements());
    assert_eq!(2, sensor_data_2.get_seq());
    println!("{:?}", sensor_data_2);
    let sensor_data_3 = sensor_data_generator(SensorData::random_measurements());
    assert_eq!(3, sensor_data_3.get_seq());
    println!("{:?}", sensor_data_3);
}

#[test]
fn buffer_life_cycle(){
    //Il generatore deve essere dichiarato come mutabile al fine di incrementare l'indice di seq
    let mut sensor_data_generator = SensorData::new();

    let mut buffer = CircularBuffer::default();
    assert_eq!(false, buffer.full());
    assert_eq!(true, buffer.empty());
    let mut c = 0;
    for _ in 0..25 {
        assert_eq!(false, buffer.full());
        buffer.store(sensor_data_generator(SensorData::random_measurements()));
        if buffer.full(){
            break;
        }
        c += 1;
        assert_eq!(false, buffer.empty() );
    }

    assert_eq!(true, buffer.full());
    assert_eq!(false, buffer.empty());
    assert_eq!(c , 19 );
}