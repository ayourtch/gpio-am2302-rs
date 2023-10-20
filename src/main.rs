mod am2302;
mod binutils;
mod cdev;

use am2302::Reading;
use cdev::push_pull;
use std::sync::{Arc, Mutex};
use std::{thread, time};

fn try_read(gpio_number: u32) -> Option<Reading> {
    let mut final_result = None;
    let all_data = push_pull(gpio_number);
    if all_data.len() < 40 {
        println!("Saad, read not enough data");
        return final_result;
    }
    for data in all_data.windows(40) {
        let result = Reading::from_binary_vector(&data);
        match result {
            Ok(reading) => {
                final_result = Some(reading);
                break;
            }
            Err(e) => {
                println!("Error: {:?}", e)
            }
        }
    }
    final_result
}

fn main() {
    let gpio_number = 4; // GPIO4  (7)
    let sleep_time = time::Duration::from_secs(1);
    let shared_data = Arc::new(Mutex::new((0.0f32, 0.0f32, time::Instant::now())));

    let writer_shared_data = Arc::clone(&shared_data);
    let writer_handle = thread::spawn(move || {
        let mut count = 0.0;
        loop {
            match try_read(gpio_number) {
                Some(reading) => {
                    println!("Reading: {:?}", reading);
                    // Lock the Mutex and update the float values.
                    if let Ok((mut locked_data)) = writer_shared_data.lock() {
                        let now = time::Instant::now();
                        *locked_data = (reading.temperature, reading.humidity, now);
                        println!("Writer updated values to: {:?}", locked_data);
                    } else {
                        println!("writer not lock!");
                    }
                }
                None => println!("Unable to get the data"),
            }
            println!(
                "Sleeping for another {:?}, to be sure that device is ready",
                sleep_time
            );
            thread::sleep(sleep_time);
        }
    });

    let reader_shared_data = Arc::clone(&shared_data);
    loop {
        {
            let locked_data = reader_shared_data.lock().unwrap();
            println!("Reader got values: {:?}", locked_data);
        }
        thread::sleep(sleep_time);
    }
}
