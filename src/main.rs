use std::error::Error as StdError;
use std::fmt::Write;
use std::io::Read;
use thiserror::Error;

use novafc_data_format::{BarometerData, Data, GyroData, HighGAccelerometerData, Message};

fn main() {
    let mut input = vec![];
    let data = std::io::stdin().read_to_end(&mut input);
    let input = String::from_utf8(input).unwrap();
    let mut messages = vec![];
    let mut page = 64;
    let mut pressures = vec![];
    for (i, line) in input.lines().enumerate() {
        //Sikp short lines
        if line.len() > 500 {
            let bytes = base64::decode(line).unwrap();
            let before = messages.len();
            match read_page(&bytes, &mut messages, &mut pressures) {
                Ok(()) => {}
                Err(Error::BufferUnderflow) => {}
                Err(e) => Err(e).unwrap(),
            }
            println!("len {}", messages.len() - before);
            page += 1;
        }
        if page > 1868 {
            break;
        }
    }
    let g_load: Vec<_> = messages
        .iter()
        .filter_map(|s| match &s.data {
            Data::HighGAccelerometerData(d) => {
                let squared = d.x * d.x + d.y * d.y + d.z * d.z;
                Some(squared.sqrt())
            }
            _ => None,
        })
        .collect();

    println!(
        "Max g load {}",
        g_load
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    );
    println!(
        "Min g load {}",
        g_load
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    );

    println!(
        "Min pressure {}",
        pressures
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    );
    println!(
        "Max pressure {}",
        pressures
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    );

    //let json = serde_json::to_string(&messages).unwrap();
    //println!("{}", json);
}

#[derive(Error, Debug)]
enum Error {
    #[error("Unexpected {expected}, got {got} at index {index}")]
    Unexpected { expected: u8, got: u8, index: usize },

    #[error("Buffer underflow")]
    BufferUnderflow,
}

type Result<T> = std::result::Result<T, Error>;

fn read_page(bytes: &[u8], messages: &mut Vec<Message>, pressures: &mut Vec<f64>) -> Result<()> {
    let mut d = Decoder {
        data: bytes.to_owned(),
        offset: 0,
    };

    d.expect(b'N')?;
    d.expect(b'O')?;
    d.expect(b'V')?;
    d.expect(b'A')?;
    loop {
        match d.expect_same_chars() {
            Ok(Some(c)) => match c {
                b'A' => {
                    let x = d.read_little_i16()?;
                    let y = d.read_little_i16()?;
                    let z = d.read_little_i16()?;
                    // +-6 g mode
                    let raw_gyro_to_g = 6.0 / i16::MAX as f64;
                    let x = x as f64 * raw_gyro_to_g;
                    let y = y as f64 * raw_gyro_to_g;
                    let z = z as f64 * raw_gyro_to_g;
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::HighGAccelerometerData(HighGAccelerometerData { x, y, z }),
                    });
                    // Looks like accel data is using the +-6g setting, so 1g = a raw value of 5460
                    //println!("Found accel data [{} {} {}]g", x, y, z);
                }
                b'B' => {
                    let raw_temp = d.read_little_i32()?;
                    //The sensor gives pascals already
                    let pressure_pascal = d.read_little_i32()? as f64;
                    let temp_c = raw_temp as f64 / 100.0;
                    let temp_k = temp_c + 273.15;
                    pressures.push(pressure_pascal);
                    //println!("Found pressure {}Pa temp: {}C", pressure_pascal, temp_c);
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::BarometerData(BarometerData {
                            temprature: temp_k,
                            pressure: pressure_pascal,
                        }),
                    });
                }
                b'G' => {
                    let x = d.read_little_i16()?;
                    let y = d.read_little_i16()?;
                    let z = d.read_little_i16()?;
                    // 2000 degree per second mode
                    // map +32767 to +2000 and -32767 to -2000
                    let raw_gyro_to_g = 2000.0 / i16::MAX as f64;
                    let x = x as f64 * raw_gyro_to_g;
                    let y = y as f64 * raw_gyro_to_g;
                    let z = z as f64 * raw_gyro_to_g;
                    //println!("Found gyro data [{} {} {}] degree / second", x, y, z);
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::GyroData(GyroData { x, y, z }),
                    });
                }
                c => {
                    panic!("Two of {} ({}) found! Bad", c as char, c);
                }
            },
            Ok(None) => {
                panic!("Ignoring different char");
            }
            Err(e) => return Ok(()),
        }
    }
}

struct Decoder {
    data: Vec<u8>,
    offset: usize,
}

impl Decoder {
    fn peek(&mut self) -> Result<u8> {
        self.ensure_available(1)?;
        Ok(self.data[self.offset])
    }

    fn next(&mut self) -> Result<u8> {
        self.ensure_available(1)?;
        let offset = self.offset;
        self.offset += 1;
        Ok(self.data[offset])
    }

    fn expect(&mut self, expected: u8) -> Result<()> {
        let next = self.next()?;
        if next == expected {
            Ok(())
        } else {
            Err(Error::Unexpected {
                expected,
                got: next,
                index: self.offset - 1,
            })
        }
    }

    /// Tries to consume the next two bytes if they are the same.
    /// If they are different, Ok(None) is returned and the buffer is advanced by one position
    fn expect_same_chars(&mut self) -> Result<Option<u8>> {
        let first = self.next()?;
        let second = self.peek()?;
        if first == second {
            Ok(Some(self.next()?))
        } else {
            Ok(None)
        }
    }

    fn read_buf<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.ensure_available(N)?;
        let mut data = [0u8; N];
        data.copy_from_slice(&self.data[self.offset..self.offset + N]);
        self.offset += N;
        Ok(data)
    }

    fn read_little_i32(&mut self) -> Result<i32> {
        let bytes: [u8; 4] = self.read_buf()?;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_little_i16(&mut self) -> Result<i16> {
        let bytes: [u8; 2] = self.read_buf()?;
        Ok(i16::from_le_bytes(bytes))
    }

    fn last_offset(&self) -> usize {
        self.offset.saturating_sub(1)
    }

    fn ensure_available(&self, more: usize) -> Result<()> {
        if self.offset + more <= self.data.len() {
            Ok(())
        } else {
            Err(Error::BufferUnderflow)
        }
    }
}
