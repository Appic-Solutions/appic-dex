use ethnum::i256;
use minicbor;
use minicbor::data::{IanaTag, Tag};
use minicbor::decode::{Decoder, Error};
use minicbor::encode::{Encoder, Write};

const I32_MAX: i256 = i256::new(i32::MAX as i128);
const I64_MAX: i256 = i256::new(i64::MAX as i128);

pub fn decode<Ctx>(d: &mut Decoder<'_>, _ctx: &mut Ctx) -> Result<i256, Error> {
    let pos = d.position();
    match d.i64() {
        Ok(n) => return Ok(i256::from(n)),
        Err(e) if e.is_type_mismatch() => {
            d.set_position(pos);
        }
        Err(e) => return Err(e),
    }

    let tag: Tag = d.tag()?;
    if tag != Tag::from(IanaTag::PosBignum) {
        return Err(Error::message(
            "failed to parse i256: expected a PosBignum tag",
        ));
    }
    let bytes = d.bytes()?;
    if bytes.len() > 32 {
        return Err(Error::message(format!(
            "failed to parse i256: expected at most 32 bytes, got: {}",
            bytes.len()
        )));
    }
    let mut be_bytes = [0u8; 32];
    be_bytes[32 - bytes.len()..32].copy_from_slice(bytes);
    Ok(i256::from_be_bytes(be_bytes))
}

pub fn encode<Ctx, W: Write>(
    v: &i256,
    e: &mut Encoder<W>,
    _ctx: &mut Ctx,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    if v <= &I32_MAX {
        e.i32(v.as_i32())?;
    } else if v <= &I64_MAX {
        e.i64(v.as_i64())?;
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
