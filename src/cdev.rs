use gpio_cdev::{Chip, Line, LineRequestFlags};
use std::{thread, time};

const LOW: u8 = 0;
const HIGH: u8 = 1;
const MAX_NUMBER_OF_READINGS: usize = 83;

fn get_line(gpio_number: u32) -> Line {
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    chip.get_line(gpio_number).unwrap()
}

fn do_init(line: &Line) {
    let output = line
        .request(LineRequestFlags::OUTPUT, HIGH, "pull-down")
        .unwrap();
    // https://cdn-shop.adafruit.com/datasheets/Digital+humidity+and+temperature+sensor+AM2302.pdf
    // Step 1: MCU send out start signal to AM2302 and AM2302 send response signal to MCU
    // MCU will pull low data-bus and this process must beyond at least 1~10ms
    // to ensure AM2302 could detect MCU's signal
    output.set_value(LOW).unwrap();
    thread::sleep(time::Duration::from_millis(3));
}

#[derive(Debug, PartialEq)]
enum EvenType {
    RisingEdge,
    FallingEdge,
}

#[derive(Debug)]
struct Event {
    timestamp: time::Instant,
    event_type: EvenType,
}

impl Event {
    pub fn new(timestamp: time::Instant, event_type: EvenType) -> Self {
        Event {
            timestamp,
            event_type,
        }
    }
}

fn events_to_data(events: &[Event]) -> Vec<u8> {
    events
        .windows(2)
        .map(|pair| {
            let prev = pair.get(0).unwrap();
            let next = pair.get(1).unwrap();
            match next.event_type {
                EvenType::FallingEdge => Some(next.timestamp - prev.timestamp),
                EvenType::RisingEdge => None,
            }
        })
        .filter(|&d| d.is_some())
        .map(|elapsed| {
            if elapsed.unwrap().as_micros() > 35 { 1 } else { 0 }
        }).collect()
}

pub fn push_pull(gpio_number: u32) -> Vec<u8> {
    let line = get_line(gpio_number);
    let mut events: Vec<Event> = Vec::with_capacity(MAX_NUMBER_OF_READINGS);
    let contact_time = time::Duration::from_secs(10);
    do_init(&line);
    read_events(&line, &mut events, contact_time);
    events_to_data(&events)
}

fn read_events(line: &Line, events: &mut Vec<Event>, contact_time: time::Duration) {
    let input = line.request(
        LineRequestFlags::INPUT,
        HIGH,
        "read-data").unwrap();

    let mut last_state = input.get_value().unwrap();
    let start = time::Instant::now();

    while start.elapsed() < contact_time {
        let new_state = input.get_value().unwrap();
        if new_state != last_state {
            let timestamp = time::Instant::now();
            let event_type = if last_state == LOW && new_state == HIGH {
                EvenType::RisingEdge
            } else {
                EvenType::FallingEdge
            };
            events.push(Event::new(timestamp, event_type));
            if events.len() >= MAX_NUMBER_OF_READINGS {
                break;
            }
            last_state = new_state;
        }
    }
}
