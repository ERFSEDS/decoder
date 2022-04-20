use std::error::Error as StdError;
use std::fmt::Write;
use thiserror::Error;

use novafc_data_format::{BarometerData, Data, GyroData, HighGAccelerometerData, Message};

fn main() {
    let bytes = base64::decode(BASE64).unwrap();
    let mut messages = vec![];
    let _ = read_page(&bytes, &mut messages);
    let json = serde_json::to_string(&messages).unwrap();
    println!("JSON: {}", json);
}

#[derive(Error, Debug)]
enum Error {
    #[error("Unexpected {expected}, got {got} at index {index}")]
    Unexpected { expected: u8, got: u8, index: usize },

    #[error("Buffer underflow")]
    BufferUnderflow,
}

type Result<T> = std::result::Result<T, Error>;

fn read_page(bytes: &[u8], messages: &mut Vec<Message>) -> Result<()> {
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
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::HighGAccelerometerData(HighGAccelerometerData { x: x, y, z }),
                    });
                    println!("Found accel data {} {} {}", x, y, z);
                }
                b'B' => {
                    let temp = d.read_little_i32()?;
                    let pressure = d.read_little_i32()?;
                    println!("Found pressure {} {}", temp, pressure);
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::BarometerData(BarometerData {
                            temprature: temp as u32,
                            pressure: pressure as u32,
                        }),
                    });
                }
                b'G' => {
                    let x = d.read_little_i16()?;
                    let y = d.read_little_i16()?;
                    let z = d.read_little_i16()?;
                    println!("Found gyro data {} {} {}", x, y, z);
                    messages.push(Message {
                        ticks_since_last_message: 1,
                        data: Data::GyroData(GyroData { x, y, z }),
                    });
                }
                c => {
                    println!("Two of {} ({}) found! Bad", c as char, c);
                }
            },
            Ok(None) => {
                println!("Ignoring different char");
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

static BASE64: &str = "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";
