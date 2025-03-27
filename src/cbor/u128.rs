use minicbor;
use minicbor::data::{IanaTag, Tag};
use minicbor::decode::{Decoder, Error};
use minicbor::encode::{Encoder, Write};

const U32_MAX: u128 = u32::MAX as u128;
const U64_MAX: u128 = u64::MAX as u128;

pub fn decode<Ctx>(d: &mut Decoder<'_>, _ctx: &mut Ctx) -> Result<u128, Error> {
    let pos = d.position();
    match d.u64() {
        Ok(n) => return Ok(u128::from(n)),
        Err(e) if e.is_type_mismatch() => {
            d.set_position(pos);
        }
        Err(e) => return Err(e),
    }

    let tag: Tag = d.tag()?;
    if tag != Tag::from(IanaTag::PosBignum) {
        return Err(Error::message(
            "failed to parse u256: expected a PosBignum tag",
        ));
    }
    let bytes = d.bytes()?;
    if bytes.len() > 16 {
        return Err(Error::message(format!(
            "failed to parse i128: expected at most 16 bytes, got: {}",
            bytes.len()
        )));
    }
    let mut be_bytes = [0u8; 16];
    be_bytes[16 - bytes.len()..16].copy_from_slice(bytes);
    Ok(u128::from_be_bytes(be_bytes))
}

pub fn encode<Ctx, W: Write>(
    v: &u128,
    e: &mut Encoder<W>,
    _ctx: &mut Ctx,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    if v <= &U32_MAX {
        e.u32(*v as u32)?;
    } else if v <= &U64_MAX {
        e.u64(*v as u64)?;
    } else {
        let be_bytes = v.to_be_bytes();
        let non_zero_pos = be_bytes
            .iter()
            .position(|x| *x != 0)
            .unwrap_or(be_bytes.len());
        e.tag(Tag::from(IanaTag::PosBignum))?
            .bytes(&be_bytes[non_zero_pos..])?;
    }
    Ok(())
}
