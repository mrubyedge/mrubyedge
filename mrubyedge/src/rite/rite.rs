extern crate simple_endian;

use super::Error;
use super::binfmt::*;

use core::ffi::CStr;
use core::mem;
use std::ffi::CString;

use super::marker::*;

use simple_endian::{u16be, u32be};

#[derive(Debug, Clone)]
pub enum PoolValue {
    Str(CString),    // IREP_TT_STR = 0 (need free)
    Int32(i32),      // IREP_TT_INT32 = 1
    SStr(CString),   // IREP_TT_SSTR = 2 (static)
    Int64(i64),      // IREP_TT_INT64 = 3
    Float(f64),      // IREP_TT_FLOAT = 5
    BigInt(Vec<u8>), // IREP_TT_BIGINT = 7 (not yet fully supported)
}

#[derive(Debug, Default)]
pub struct Rite<'a> {
    pub binary_header: RiteBinaryHeader,
    pub irep_header: SectionIrepHeader,
    pub irep: Vec<Irep<'a>>,
    pub lvar: Option<LVar>,
}

#[derive(Debug)]
pub struct Irep<'a> {
    pub header: IrepRecord,
    pub insn: &'a [u8],
    pub plen: usize,
    pub pool: Vec<PoolValue>,
    pub slen: usize,
    pub syms: Vec<CString>,
    pub catch_handlers: Vec<CatchHandler>,
}

impl Irep<'_> {
    pub fn nlocals(&self) -> usize {
        be16_to_u16(self.header.nlocals) as usize
    }

    pub fn nregs(&self) -> usize {
        be16_to_u16(self.header.nregs) as usize
    }

    pub fn rlen(&self) -> usize {
        be16_to_u16(self.header.rlen) as usize
    }

    pub fn clen(&self) -> usize {
        be16_to_u16(self.header.clen) as usize
    }
}

#[derive(Debug)]
pub struct CatchHandler {
    pub type_: u8,
    pub start: usize,
    pub end: usize,
    pub target: usize,
}

#[derive(Debug)]
pub struct LVar {
    pub header: SectionMiscHeader,
}

pub fn load<'a>(src: &'a [u8]) -> Result<Rite<'a>, Error> {
    let mut rite = Rite::default();

    let mut size = src.len();
    let mut head = src;
    let binheader_size = mem::size_of::<RiteBinaryHeader>();
    if size < binheader_size {
        dbg!(size < binheader_size);
        return Err(Error::TooShort);
    }
    let binary_header = RiteBinaryHeader::from_bytes(&head[0..binheader_size])?;
    rite.binary_header = binary_header;
    size -= binheader_size;
    head = &head[binheader_size..];

    // let binsize: u32 = be32_to_u32(rite.binary_header.size);

    let irep_header_size = mem::size_of::<SectionIrepHeader>();
    if size < irep_header_size {
        dbg!(size, irep_header_size, size < irep_header_size);
        return Err(Error::TooShort);
    }

    while let Some(chrs) = peek4(head) {
        match chrs {
            IREP => {
                let (cur, irep_header, irep) = section_irep_1(head)?;
                rite.irep_header = irep_header;
                rite.irep = irep;
                head = &head[cur..];
            }
            LVAR => {
                let (cur, lvar) = section_lvar(head)?;
                rite.lvar = Some(lvar);
                head = &head[cur..];
            }
            DBG => {
                let cur = section_skip(head)?;
                head = &head[cur..];
            }
            END => {
                let cur = section_end(head)?;
                head = &head[cur..];
            }
            _ => {
                eprintln!("{:?}", chrs);
                eprint!("{:?}", head);
                return Err(Error::InvalidFormat);
            }
        }
    }

    Ok(rite)
}

pub fn section_irep_1(head: &[u8]) -> Result<(usize, SectionIrepHeader, Vec<Irep<'_>>), Error> {
    let mut cur = 0;

    let irep_header_size = mem::size_of::<SectionIrepHeader>();
    let irep_header = SectionIrepHeader::from_bytes(&head[cur..irep_header_size])?;
    let irep_size = be32_to_u32(irep_header.size) as usize;
    if head.len() < irep_size {
        dbg!((head.len(), irep_size, head.len() < irep_size));
        return Err(Error::TooShort);
    }
    cur += irep_header_size;

    let mut ireps: Vec<Irep> = Vec::new();

    while cur < irep_size {
        let mut pool = Vec::<PoolValue>::new();
        let mut syms = Vec::<CString>::new();

        let start_cur = cur;
        // insn
        let record_size = mem::size_of::<IrepRecord>();
        let irep_record = IrepRecord::from_bytes(&head[cur..cur + record_size])?;
        let irep_rec_size = be32_to_u32(irep_record.size) as usize;
        let ilen = be32_to_u32(irep_record.ilen) as usize;
        cur += record_size;

        let insns = &head[cur..cur + ilen];

        cur += ilen;

        let mut catch_handlers = Vec::<CatchHandler>::new();
        let clen = be16_to_u16(irep_record.clen) as usize;
        if clen > 0 {
            for _ in 0..clen {
                let value = CatchHandler {
                    type_: head[cur],
                    start: be32_to_u32([head[cur + 1], head[cur + 2], head[cur + 3], head[cur + 4]])
                        as usize,
                    end: be32_to_u32([head[cur + 5], head[cur + 6], head[cur + 7], head[cur + 8]])
                        as usize,
                    target: be32_to_u32([
                        head[cur + 9],
                        head[cur + 10],
                        head[cur + 11],
                        head[cur + 12],
                    ]) as usize,
                };
                catch_handlers.push(value);
                cur += mem::size_of::<IrepCatchHandler>();
            }
        }

        // pool
        let data = &head[cur..cur + 2];
        let plen = be16_to_u16([data[0], data[1]]) as usize;
        cur += 2;

        for _ in 0..plen {
            let typ = head[cur];
            cur += 1;
            match typ {
                0 => {
                    // IREP_TT_STR: String (need free)
                    let data = &head[cur..cur + 2];
                    let strlen = be16_to_u16([data[0], data[1]]) as usize + 1;
                    cur += 2;
                    let strval = CStr::from_bytes_with_nul(&head[cur..cur + strlen])
                        .or(Err(Error::InvalidFormat))?;
                    pool.push(PoolValue::Str(strval.to_owned()));
                    cur += strlen;
                }
                1 => {
                    // IREP_TT_INT32: 32-bit integer
                    let data = &head[cur..cur + 4];
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(data);
                    let intval = i32::from_be_bytes(bytes);
                    pool.push(PoolValue::Int32(intval));
                    cur += 4;
                }
                2 => {
                    // IREP_TT_SSTR: Static string
                    let data = &head[cur..cur + 2];
                    let strlen = be16_to_u16([data[0], data[1]]) as usize + 1;
                    cur += 2;
                    let strval = CStr::from_bytes_with_nul(&head[cur..cur + strlen])
                        .or(Err(Error::InvalidFormat))?;
                    pool.push(PoolValue::SStr(strval.to_owned()));
                    cur += strlen;
                }
                3 => {
                    // IREP_TT_INT64: 64-bit integer
                    let data = &head[cur..cur + 8];
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(data);
                    let intval = i64::from_le_bytes(bytes);
                    pool.push(PoolValue::Int64(intval));
                    cur += 8;
                }
                5 => {
                    // IREP_TT_FLOAT: Float (double/float)
                    let data = &head[cur..cur + 8];
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(data);
                    let floatval = f64::from_le_bytes(bytes);
                    pool.push(PoolValue::Float(floatval));
                    cur += 8;
                }
                7 => {
                    // IREP_TT_BIGINT: Big integer (not yet fully supported)
                    let data = &head[cur..cur + 2];
                    let bigint_len = be16_to_u16([data[0], data[1]]) as usize;
                    cur += 2;
                    let bigint_data = head[cur..cur + bigint_len].to_vec();
                    pool.push(PoolValue::BigInt(bigint_data));
                    cur += bigint_len;
                }
                v => {
                    return Err(Error::UnknownPoolType(v));
                }
            }
        }

        // syms
        let data = &head[cur..cur + 2];
        let slen = be16_to_u16([data[0], data[1]]) as usize;
        cur += 2;
        for _ in 0..slen {
            let data = &head[cur..cur + 2];
            let symlen = be16_to_u16([data[0], data[1]]) as usize + 1;
            cur += 2;
            let symval = CStr::from_bytes_with_nul(&head[cur..cur + symlen])
                .or(Err(Error::InvalidFormat))?;
            syms.push(symval.to_owned());
            cur += symlen;
        }

        cur = start_cur + irep_rec_size;

        let irep = Irep {
            header: irep_record,
            insn: insns,
            plen,
            pool,
            slen,
            syms,
            catch_handlers,
        };
        ireps.push(irep);
    }

    Ok((irep_size, irep_header, ireps))
}

pub fn section_end(head: &[u8]) -> Result<usize, Error> {
    let header = SectionMiscHeader::from_bytes(head)?;
    // eprintln!("end section detected");
    Ok(be32_to_u32(header.size) as usize)
}

pub fn section_lvar(head: &[u8]) -> Result<(usize, LVar), Error> {
    let header = SectionMiscHeader::from_bytes(head)?;
    let lvar = LVar { header };
    Ok((be32_to_u32(lvar.header.size) as usize, lvar))
}

pub fn section_skip(head: &[u8]) -> Result<usize, Error> {
    let header = SectionMiscHeader::from_bytes(head)?;
    // eprintln!("skipped section {:?}", header.ident.as_ascii());
    Ok(be32_to_u32(header.size) as usize)
}

pub fn peek4(src: &[u8]) -> Option<[char; 4]> {
    if src.len() < 4 {
        // EoD
        return None;
    }
    if let [a, b, c, d] = src[0..4] {
        let a = char::from_u32(a as u32).unwrap();
        let b = char::from_u32(b as u32).unwrap();
        let c = char::from_u32(c as u32).unwrap();
        let d = char::from_u32(d as u32).unwrap();
        Some([a, b, c, d])
    } else {
        None
    }
}

pub fn be32_to_u32(be32: [u8; 4]) -> u32 {
    let binsize_be = unsafe { mem::transmute::<[u8; 4], u32be>(be32) };
    let binsize: u32 = binsize_be.into();
    binsize
}

pub fn be16_to_u16(be16: [u8; 2]) -> u16 {
    let binsize_be = unsafe { mem::transmute::<[u8; 2], u16be>(be16) };
    let binsize: u16 = binsize_be.into();
    binsize
}
