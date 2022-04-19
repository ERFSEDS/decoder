use std::error::Error as StdError;
use std::fmt::Write;
use thiserror::Error;

fn main() {
    let bytes = hex::decode(HEX).unwrap();
    read_page(&bytes).unwrap();
}

#[derive(Error, Debug)]
enum Error {
    #[error("Unexpected {expected}, got {got} at index {index}")]
    Unexpected { expected: u8, got: u8, index: usize },

    #[error("Buffer underflow")]
    BufferUnderflow,
}

type Result<T> = std::result::Result<T, Error>;

fn read_page(bytes: &[u8]) -> Result<()> {
    for &b in bytes.iter().take(32) {
        match b {
            32..=127 => print!("{}", b as char),
            _ => print!("\\x{}", b),
        }
        print!(", ");
    }
    let mut d = Decoder {
        data: bytes[..128].to_owned(),
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
                    println!("Found accel data {} {} {}", x, y, z);
                }
                b'B' => {
                    let temp = d.read_little_i32()?;
                    let pressure = d.read_little_i32()?;
                    println!("Found pressure {} {}", temp, pressure);
                }
                b'G' => {
                    let x = d.read_little_i16()?;
                    let y = d.read_little_i16()?;
                    let z = d.read_little_i16()?;
                    println!("Found gyro data {} {} {}", x, y, z);
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

static HEX: &str = "4E4F564142421AE006E8D10414173DC173F164747892BFF61CF742421AE005A8D10414173DC173F164747642FBF6EAF642421AE005A8D10414173DC173F1647473F234F7C0F642421AE006E8D104141FEEBE91E184747F4170F782F642421AE006E8D104141FEEBE91E184747CB1B7F766F642421AE006E8D104141FEEBE91E184747741F3F73CF642421AE00828D104141FEEBE91E18474727135F81DF642421AE006E8D104141E91089C851B4747EC067F8FCF542421AE006E8D104141E91089C851B474781097F8F9F542421AE006E8D104141E91089C851B4747270DEF8D9F542421AE006E8D104141E91089C851B4747D8FF7F9D1F542421AE006E8D1041411E128FECB1E474776FF25F9C9F542421AE006E8D1041411E128FECB1E4747F8FE39F9C9F542421AE006E8D1041411E128FECB1E4747B1FE4AF9AFF542421AE006E8D1041411E128FECB1E47473DFE52F9B7F542421AE006E8D1041418C1181FA7204747DBFD50F9BEF542421AE006E8D1041418C1181FA720474788FD44F99FF542421AE005A8D1041418C1181FA72047472DFD26F9B1F542421BE00728D1041418C1181FA7204747C2FCFFF8C8F542421BE00728D1041418C1181FA72047476DFCE2F8C5F542421BE005E8D10414170FEEE9C20474722FCA1F8D6F542421BE00728D10414170FEEE9C204747B6FB64F8D0F542421BE005E8D10414170FEEE9C20474760FB26F8D5F542421AE005A8D10414170FEEE9C2047471DFBE9F7E2F542421BE00728D104141FECE85204747BCFA90F7E6F542421BE00728D104141FECE852047477CFA64F7D0F542421BE005E8D104141FECE8520474721FADFF6F9F542421BE00858D104141FECE85204747E7F9A2F6AF642421BE005E8D104141D4DB4D4B214747AAF95CF6FFF542421BE00728D104141D4DB4D4B21474772F920F625F642421BE00728D104141D4DB4D4B21474738F9DAF538F642421BE00728D104141D4DB4D4B214747EF9A0F535F642421BE00728D104141D4DB4D4B214747F5F85FF541F642421BE00728D104141C3E22D8A214747EEF840F55CF642421BE00728D104141C3E22D8A214747F3F81BF56AF642421BE00728D104141C3E22D8A2147475F9FFF468F642421BE00728D104141C3E22D8A2147471BF9D1F477F642421BE00728D1041411910A3B4E21474745F9C0F477F642421BE00728D1041411910A3B4E21474788F9C7F48AF642421DE00768D1041411910A3B4E214747CAF9CEF487F642421DE00768D1041411910A3B4E214747FCF9C1F481F642421DE00768D10414141278A2722474755FACEF466F642421BE00858D10414141278A2722474799FAE9F475F642421BE00728D10414141278A27224747E7FA8F567F642421BE00728D10414141278A272247474CFB32F552F642421DE00768D10414144145BA6F2347479FFB69F555F642421BE005E8D10414144145BA6F2347474FC97F53FF642421BE00728D10414144145BA6F2347474CFCE1F541F642421DE00768D10414144145BA6F234747B5FC38F628F642421DE00628D10414144145BA6F2347476FD86F621F642421DE00768D104141E31575A3D23474753FD7F7FFF542421DE00628D104141E31575A3D2347479EFD8BF7F4F542421BE00728D104141E31575A3D234747CBFDCAF7FBF542421DE00768D104141E31575A3D2347475FE74F8D5F542421DE00628D1041415016CDAB220474715FEFFF8E2F542421DE00768D1041415016CDAB220474712FE68F9C1F542421DE00768D1041415016CDAB22047471CFEECF9CAF542421DE00768D1041415016CDAB220474718FE81FAC9F542421DE00768D104141C51585CB91E4747EFEDFFAC9F542421DE00768D104141C51585CB91E4747F3FD56FBCEF542421DE00768D104141C51585CB91E4747EFFDC5FBC9F542421DE00768D104141C51585CB91E4747C8FD1EFCE2F542421DE00628D104141F4141DE551F4747DBFD89FCEFF542421FE00798D104141F4141DE551F47478EFDD5FCFDF542421DE00768D104141F4141DE551F474793FD17FDFEF542421DE00768D104141F4141DE551F47477BFD4EFD17F642421DE00768D104141F4141DE551F474740FD83FD25F642421DE00768D10414150131DFA91F47471EFD93FD2AF642421FE00798D10414150131DFA91F4747FFFCBEFD56F642421DE00768D10414150131DFA91F4747F3FCDFFD53F642421DE00768D10414150131DFA91F4747D2FCD3FD64F642421DE00768D104141851177F661F4747C6FCDEFD6DF642421DE00768D104141851177F661F4747C4FCE4FD65F642421FE00798D104141851177F661F4747C4FC4FE54F642421DE00768D104141851177F661F4747B5FCEFE49F642421DE00768D104141ADFDDD661C4747C3FC38FE28F642421FE00798D104141ADFDDD661C4747C5FC49FE2F642421FE00798D104141ADFDDD661C4747DDFC92FEDBF542421DE00768D104141ADFDDD0";
