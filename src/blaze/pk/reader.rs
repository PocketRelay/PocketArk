//! Buffer reading wrapper to provided a way to easily read data from
//! packet buffers provides easy functions for all the different tdf types

use super::{
    codec::{Decodable, ValueType},
    error::{DecodeError, DecodeResult},
    tag::{Tag, Tagged, TdfType},
    types::{TdfMap, UNION_UNSET},
};
use std::borrow::Cow;

/// Buffered readable implementation. Allows reading through the
/// underlying slice using a cursor and with a position that can
/// be saved using the marker. Provides functions for reading
/// certain data types in the Blaze format
pub struct TdfReader<'a> {
    /// The underlying buffer to read from
    pub buffer: &'a [u8],
    /// The cursor position on the buffer. The cursor should not be set
    /// to any arbitry values should only be set to previously know values
    pub cursor: usize,
}

/// Macro for implementing VarInt decoding for a specific number type
/// to prevent allocating for a u64 for every other number type
macro_rules! impl_decode_var {
    ($ty:ty, $reader:ident) => {{
        let first: u8 = $reader.read_byte()?;
        let mut result: $ty = (first & 63) as $ty;
        if first < 128 {
            return Ok(result);
        }
        let mut shift: u8 = 6;
        let mut byte: u8;
        loop {
            byte = $reader.read_byte()?;
            result |= ((byte & 127) as $ty) << shift;
            if byte < 128 {
                break;
            }
            shift += 7;
        }
        Ok(result)
    }};
}

impl<'a> TdfReader<'a> {
    /// Creates a new reader over the provided slice of bytes with
    /// the default cursor position at zero
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    /// Takes a single byte from the underlying buffer moving
    /// the cursor over by one. Will return UnexpectedEof error
    /// if there are no bytes left
    pub fn read_byte(&mut self) -> DecodeResult<u8> {
        self.expect_length(1)?;
        let byte: u8 = self.buffer[self.cursor];
        self.cursor += 1;
        Ok(byte)
    }

    /// Attempts to take four bytes from the underlying buffer moving
    /// the cursor over 4 bytes. This is used when decoding tags and
    /// taking floats as they both require 4 bytes. Will return an
    /// UnexpectedEof error if there is not 4 bytes after the cursor
    fn read_byte_4(&mut self) -> DecodeResult<[u8; 4]> {
        // Ensure we have the required number of bytes
        self.expect_length(4)?;
        // Alocate and copy the bytes from the buffer
        let mut bytes: [u8; 4] = [0u8; 4];
        bytes.copy_from_slice(&self.buffer[self.cursor..self.cursor + 4]);
        // Move the cursor
        self.cursor += 4;
        Ok(bytes)
    }

    /// Takes a slice of the provided length from the portion of the
    /// buffer that is after the cursor position
    ///
    /// `length` The length of the slice to take
    pub fn read_slice(&mut self, length: usize) -> DecodeResult<&[u8]> {
        // Ensure we have the required number of bytes
        if self.cursor + length > self.buffer.len() {
            return Err(DecodeError::UnexpectedEof {
                cursor: self.cursor,
                wanted: length,
                remaining: self.len(),
            });
        }
        let slice: &[u8] = &self.buffer[self.cursor..self.cursor + length];
        self.cursor += length;
        Ok(slice)
    }

    /// Takes a float value from the buffer which moves the
    /// cursor over by 4 bytes
    pub fn read_f32(&mut self) -> DecodeResult<f32> {
        let bytes: [u8; 4] = self.read_byte_4()?;
        Ok(f32::from_be_bytes(bytes))
    }

    /// Attempts to ensure the next length exists past the cursor
    /// will return an UnexpectedEof if there is not enough bytes
    ///
    /// `length` The length to expect
    fn expect_length(&self, length: usize) -> DecodeResult<()> {
        if self.cursor + length > self.buffer.len() {
            Err(DecodeError::UnexpectedEof {
                cursor: self.cursor,
                wanted: length,
                remaining: self.len(),
            })
        } else {
            Ok(())
        }
    }

    /// Returns the remaining length left after the cursor
    pub fn len(&self) -> usize {
        self.buffer.len() - self.cursor
    }

    /// Returns if there is nothing left after the cursor
    pub fn is_empty(&self) -> bool {
        self.cursor >= self.buffer.len()
    }

    /// Decodes a u8 value using the VarInt encoding
    pub fn read_u8(&mut self) -> DecodeResult<u8> {
        let first: u8 = self.read_byte()?;
        let mut result: u8 = first & 63;
        // Values less than 128 are already complete and don't need more reading
        if first < 128 {
            return Ok(result);
        }

        let byte: u8 = self.read_byte()?;
        result |= (byte & 127) << 6;

        // Consume remaining unused VarInt data. We only wanted a u8
        if byte >= 128 {
            self.skip_var_int();
        }
        Ok(result)
    }

    /// Decodes a u16 value using hte VarInt encoding. This uses
    /// the impl_decode_var macro so its implementation is the
    /// same as others
    pub fn read_u16(&mut self) -> DecodeResult<u16> {
        impl_decode_var!(u16, self)
    }

    /// Decodes a u32 value using hte VarInt encoding. This uses
    /// the impl_decode_var macro so its implementation is the
    /// same as others
    pub fn read_u32(&mut self) -> DecodeResult<u32> {
        impl_decode_var!(u32, self)
    }

    /// Decodes a u64 value using hte VarInt encoding. This uses
    /// the impl_decode_var macro so its implementation is the
    /// same as others
    pub fn read_u64(&mut self) -> DecodeResult<u64> {
        impl_decode_var!(u64, self)
    }

    /// Decodes a u64 value using hte VarInt encoding. This uses
    /// the impl_decode_var macro so its implementation is the
    /// same as others
    pub fn read_usize(&mut self) -> DecodeResult<usize> {
        impl_decode_var!(usize, self)
    }

    /// Reads a blob from the buffer. The blob is a slice prefixed
    /// by a length value
    pub fn read_blob(&mut self) -> DecodeResult<&[u8]> {
        let length: usize = self.read_usize()?;
        let bytes: &[u8] = self.read_slice(length)?;
        Ok(bytes)
    }

    /// Reads a string from the underlying buffer
    pub fn read_string(&mut self) -> DecodeResult<String> {
        let bytes: &[u8] = self.read_blob()?;
        let text: Cow<str> = String::from_utf8_lossy(bytes);
        let mut text: String = text.to_string();
        // Remove null terminator
        text.pop();
        Ok(text)
    }

    /// Reads a boolean value this is encoded using the
    /// var int encoding
    pub fn read_bool(&mut self) -> DecodeResult<bool> {
        Ok(matches!(self.read_u8()?, 1))
    }

    /// Reads a map from the underlying buffer
    pub fn read_map<K: Decodable + ValueType, V: Decodable + ValueType>(
        &mut self,
    ) -> DecodeResult<TdfMap<K, V>> {
        let length: usize = self.read_map_header(K::value_type(), V::value_type())?;
        self.read_map_body(length)
    }

    /// Reads a map header from the underlying buffer ensuring that the key
    /// and value types match the provided key and value types. Returns
    /// the length of the following content
    ///
    /// `exp_key_type`   The type of key to expect
    /// `exp_value_type` The type of value to expect
    pub fn read_map_header(
        &mut self,
        exp_key_type: TdfType,
        exp_value_type: TdfType,
    ) -> DecodeResult<usize> {
        let key_type: TdfType = self.read_type()?;
        if key_type != exp_key_type {
            return Err(DecodeError::InvalidType {
                expected: exp_key_type,
                actual: key_type,
            });
        }
        let value_type: TdfType = self.read_type()?;
        if value_type != exp_value_type {
            return Err(DecodeError::InvalidType {
                expected: exp_value_type,
                actual: value_type,
            });
        }
        self.read_usize()
    }

    /// Reads the contents of the map for the provided key value types
    /// and for the provided length
    ///
    /// `length` The length of the map (The number of entries)
    pub fn read_map_body<K: Decodable, V: Decodable>(
        &mut self,
        length: usize,
    ) -> DecodeResult<TdfMap<K, V>> {
        let mut map: TdfMap<K, V> = TdfMap::with_capacity(length);
        for _ in 0..length {
            let key: K = K::decode(self)?;
            let value: V = V::decode(self)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    /// Decodes tags from the reader until the tag with the provided tag name
    /// is found. If the tag type doesn't match the `expected_type` then an
    /// error will be returned.
    ///
    /// `tag` The tag name to read until
    /// `ty`  The expected type of the tag
    pub fn until_tag(&mut self, tag: &[u8], ty: TdfType) -> DecodeResult<()> {
        let tag = Tag::from(tag);
        loop {
            let next_tag = match self.read_tag() {
                Ok(value) => value,
                Err(DecodeError::UnexpectedEof { .. }) => {
                    return Err(DecodeError::MissingTag { tag, ty })
                }
                Err(err) => return Err(err),
            };

            if next_tag.tag != tag {
                self.skip_type(&next_tag.ty)?;
                continue;
            }

            if next_tag.ty.ne(&ty) {
                return Err(DecodeError::InvalidTagType {
                    tag,
                    expected: ty,
                    actual: next_tag.ty,
                });
            }

            return Ok(());
        }
    }

    /// Attempting version of decode_until that returns true if the value was decoded up to
    /// otherwise returns false. Marks the reader position before reading and resets the position
    /// if the tag was not found
    ///
    /// `tag` The tag name to read until
    /// `ty`  The expected type of the tag
    pub fn try_until_tag(&mut self, tag: &[u8], ty: TdfType) -> bool {
        let tag = Tag::from(tag);
        let start = self.cursor;

        while let Ok(next_tag) = self.read_tag() {
            if next_tag.tag != tag {
                if self.skip_type(&next_tag.ty).is_err() {
                    break;
                } else {
                    continue;
                }
            }

            if next_tag.ty.ne(&ty) {
                break;
            }

            return true;
        }
        self.cursor = start;
        false
    }

    /// Reads the provided tag from the buffer discarding values until it
    /// reaches the correct value.
    ///
    /// `tag` The tag name to read
    pub fn tag<C: Decodable + ValueType>(&mut self, tag: &[u8]) -> DecodeResult<C> {
        self.until_tag(tag, C::value_type())?;
        C::decode(self)
    }

    /// Reads the provided tag from the buffer discarding values until it
    /// reaches the correct value. If the tag is missing the cursor is reset
    /// back to where it was
    ///
    /// `tag` The tag name to read
    pub fn try_tag<C: Decodable + ValueType>(&mut self, tag: &[u8]) -> DecodeResult<Option<C>> {
        let start = self.cursor;
        match self.tag(tag) {
            Ok(value) => Ok(Some(value)),
            Err(DecodeError::MissingTag { .. }) => {
                self.cursor = start;
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    /// Reads the next TdfType value after the cursor
    pub fn read_type(&mut self) -> DecodeResult<TdfType> {
        let value = self.read_byte()?;
        TdfType::try_from(value)
    }

    /// Reads a tag from the underlying buffer
    pub fn read_tag(&mut self) -> DecodeResult<Tagged> {
        let input: [u8; 4] = self.read_byte_4()?;
        let ty: TdfType = TdfType::try_from(input[3])?;
        let mut output: [u8; 4] = [0, 0, 0, 0];

        output[0] |= (input[0] & 0x80) >> 1;
        output[0] |= (input[0] & 0x40) >> 2;
        output[0] |= (input[0] & 0x30) >> 2;
        output[0] |= (input[0] & 0x0C) >> 2;

        output[1] |= (input[0] & 0x02) << 5;
        output[1] |= (input[0] & 0x01) << 4;
        output[1] |= (input[1] & 0xF0) >> 4;

        output[2] |= (input[1] & 0x08) << 3;
        output[2] |= (input[1] & 0x04) << 2;
        output[2] |= (input[1] & 0x03) << 2;
        output[2] |= (input[2] & 0xC0) >> 6;

        output[3] |= (input[2] & 0x20) << 1;
        output[3] |= input[2] & 0x1F;

        let tag = Tag(output);

        Ok(Tagged { tag, ty })
    }

    /// Skips the 4 bytes required for a 32 bit float value
    pub fn skip_f32(&mut self) -> DecodeResult<()> {
        self.expect_length(4)?;
        self.cursor += 4;
        Ok(())
    }

    /// Skips the next string value
    pub fn skip_blob(&mut self) -> DecodeResult<()> {
        let length: usize = self.read_usize()?;
        self.expect_length(length)?;
        self.cursor += length;
        Ok(())
    }

    /// Skips the next var int value
    pub fn skip_var_int(&mut self) {
        while self.cursor < self.buffer.len() {
            let byte = self.buffer[self.cursor];
            self.cursor += 1;
            if byte < 128 {
                break;
            }
        }
    }

    /// Skips the starting 2 value at the beggining of the group
    /// if it exists
    pub fn skip_group_2(&mut self) -> DecodeResult<()> {
        let first = self.read_byte()?;
        if first != 2 {
            self.cursor -= 1;
        }
        Ok(())
    }

    /// Skips an entire group if one exists
    pub fn skip_group(&mut self) -> DecodeResult<()> {
        self.skip_group_2()?;
        while self.cursor < self.buffer.len() {
            let byte: u8 = self.buffer[self.cursor];
            if byte == 0 {
                self.cursor += 1;
                break;
            }
            self.skip()?;
        }
        Ok(())
    }

    /// Skips a list of items
    pub fn skip_list(&mut self) -> DecodeResult<()> {
        let ty: TdfType = self.read_type()?;
        let length: usize = self.read_usize()?;
        for _ in 0..length {
            self.skip_type(&ty)?;
        }
        Ok(())
    }

    /// Skips a map
    pub fn skip_map(&mut self) -> DecodeResult<()> {
        let key_ty: TdfType = self.read_type()?;
        let value_ty: TdfType = self.read_type()?;
        let length: usize = self.read_usize()?;
        for _ in 0..length {
            self.skip_type(&key_ty)?;
            self.skip_type(&value_ty)?;
        }
        Ok(())
    }

    /// Skips a union value
    pub fn skip_union(&mut self) -> DecodeResult<()> {
        let ty = self.read_byte()?;
        if ty != UNION_UNSET {
            self.skip()?;
        }
        Ok(())
    }

    /// Skips a var int list
    pub fn skip_var_int_list(&mut self) -> DecodeResult<()> {
        let length: usize = self.read_usize()?;
        for _ in 0..length {
            self.skip_var_int();
        }
        Ok(())
    }

    /// Skips the next tag value
    pub fn skip(&mut self) -> DecodeResult<()> {
        let tag = self.read_tag()?;
        self.skip_type(&tag.ty)
    }

    /// Skips a data type
    ///
    /// `ty` The type of data to skip
    pub fn skip_type(&mut self, ty: &TdfType) -> DecodeResult<()> {
        match ty {
            TdfType::VarInt => self.skip_var_int(),
            TdfType::String | TdfType::Blob => self.skip_blob()?,
            TdfType::Group => self.skip_group()?,
            TdfType::List => self.skip_list()?,
            TdfType::Map => self.skip_map()?,
            TdfType::Union => self.skip_union()?,
            TdfType::VarIntList => self.skip_var_int_list()?,
            TdfType::Pair => {
                self.skip_var_int();
                self.skip_var_int();
            }
            TdfType::Triple => {
                self.skip_var_int();
                self.skip_var_int();
                self.skip_var_int();
            }
            TdfType::Float => self.skip_f32()?,
        }
        Ok(())
    }

    /// Decodes all the contents within the reader into a string
    /// representation
    ///
    /// `out` The string output to append to
    pub fn stringify(&mut self, out: &mut String) -> DecodeResult<()> {
        while self.cursor < self.buffer.len() {
            if let Err(err) = self.stringify_tag(out, 1) {
                let remaining = &self.buffer[self.cursor..];
                out.push_str(&format!(
                    "... cause: {:?}, remaining {} {:?}",
                    err,
                    remaining.len(),
                    remaining,
                ));
                break;
            }
        }
        Ok(())
    }

    /// Decodes and converts the next tag into
    /// a string representation
    ///
    /// `out`    The string output to append to
    /// `indent` The current indent level
    pub fn stringify_tag(&mut self, out: &mut String, indent: usize) -> DecodeResult<()> {
        let tag = self.read_tag()?;
        out.push_str(&"  ".repeat(indent));
        out.push_str(&format!("\"{}\": ", &tag.tag));
        match self.stringify_type(out, indent, &tag.ty) {
            Ok(_) => {
                out.push_str(",\n");
                Ok(())
            }
            Err(err) => {
                out.push_str("...");
                Err(err)
            }
        }
    }

    /// Decodes and converts the next value of the provided type
    /// into a string representation
    ///
    /// `out`    The string output to append to
    /// `indent` The current indent level
    /// `ty`     The type
    pub fn stringify_type(
        &mut self,
        out: &mut String,
        indent: usize,
        ty: &TdfType,
    ) -> DecodeResult<()> {
        match ty {
            TdfType::VarInt => {
                let value = self.read_usize()?;
                out.push_str(&value.to_string());
            }
            TdfType::String => {
                let value = self.read_string()?;
                out.push('"');
                out.push_str(&value);
                out.push('"');
            }
            TdfType::Blob => {
                let value = self.read_blob()?;
                let length = value.len();
                out.push_str("Blob [");
                for (i, value) in value.iter().enumerate() {
                    out.push_str(&format!("0x{:X}", value));
                    if i < length - 1 {
                        out.push_str(", ");
                    }
                }
                out.push(']');
            }
            TdfType::Group => {
                out.push_str("{\n");
                let mut is_two: bool = false;
                while self.cursor < self.buffer.len() {
                    let byte: u8 = self.buffer[self.cursor];
                    if byte == 0 {
                        self.cursor += 1;
                        break;
                    }
                    if byte == 2 {
                        is_two = true;
                        self.cursor += 1;
                    }
                    self.stringify_tag(out, indent + 1)?;
                }
                out.push_str(&"  ".repeat(indent));
                out.push('}');
                if is_two {
                    out.push_str(" (2)");
                }
            }
            TdfType::List => {
                let value_type: TdfType = self.read_type()?;
                let length: usize = self.read_usize()?;
                let expand = matches!(value_type, TdfType::Map | TdfType::Group);
                out.push('[');
                if expand {
                    out.push('\n');
                }

                for i in 0..length {
                    if expand {
                        out.push_str(&"  ".repeat(indent + 1));
                    }
                    self.stringify_type(out, indent + 1, &value_type)?;
                    if i < length - 1 {
                        out.push_str(", ");
                    }
                    if expand {
                        out.push('\n');
                    }
                }
                if expand {
                    out.push_str(&"  ".repeat(indent));
                }
                out.push(']');
            }
            TdfType::Map => {
                let key_type: TdfType = self.read_type()?;
                let value_type: TdfType = self.read_type()?;
                let length: usize = self.read_usize()?;
                out.push_str(&format!(
                    "Map<{:?}, {:?}, {}>",
                    key_type, value_type, length
                ));
                out.push_str("{\n");

                let start = self.cursor;

                let mut proc = || -> DecodeResult<()> {
                    for i in 0..length {
                        out.push_str(&"  ".repeat(indent + 1));
                        self.stringify_type(out, indent + 1, &key_type)?;
                        out.push_str(": ");
                        self.stringify_type(out, indent + 1, &value_type)?;
                        if i < length - 1 {
                            out.push(',');
                        }
                        out.push('\n')
                    }
                    Ok(())
                };

                if let Err(err) = proc() {
                    out.push_str(&format!("Err: {}", err));
                    out.push_str(&format!("Full Bytes: {:?}", &self.buffer[start..]));
                }

                out.push_str(&"  ".repeat(indent));
                out.push('}');
            }
            TdfType::Union => {
                let ty = self.read_byte()?;
                if ty == UNION_UNSET {
                    out.push_str("Union(Unset)")
                } else {
                    let tag = self.read_tag()?;
                    out.push_str(&format!("Union(\"{}\", {}, ", &tag.tag, ty));
                    self.stringify_type(out, indent + 1, &tag.ty)?;
                    out.push(')')
                }
            }
            TdfType::VarIntList => {
                let length: usize = self.read_usize()?;
                out.push_str("VarList [");
                for i in 0..length {
                    let value = self.read_usize()?;
                    out.push_str(&value.to_string());
                    if i < length - 1 {
                        out.push_str(", ");
                    }
                }
                out.push(']');
            }
            TdfType::Pair => {
                let a = self.read_usize()?;
                let b = self.read_usize()?;

                out.push_str(&format!("({}, {})", a, b))
            }
            TdfType::Triple => {
                let a = self.read_usize()?;
                let b = self.read_usize()?;
                let c = self.read_usize()?;

                out.push_str(&format!("({}, {}, {})", a, b, c))
            }
            TdfType::Float => {
                let value = self.read_f32()?;
                out.push_str(&value.to_string());
            }
        };
        Ok(())
    }

    /// Reads until the next list values selection for the provided
    /// tag. Will read the value type and the length returning
    /// the length.
    ///
    /// `tag`        The tag to read
    /// `value_type` The expected value type
    pub fn until_list(&mut self, tag: &[u8], value_type: TdfType) -> DecodeResult<usize> {
        self.until_tag(tag, TdfType::List)?;
        let list_type = self.read_type()?;
        if list_type != value_type {
            return Err(DecodeError::InvalidType {
                expected: value_type,
                actual: list_type,
            });
        }
        let count = self.read_usize()?;
        Ok(count)
    }

    /// Reads until the next map values selection for the provided
    /// tag. Will read the key value types and the length returning
    /// the length.
    ///
    /// `tag`        The tag to read
    /// `key_type`   The expected key type
    /// `value_type` The expected value type
    pub fn until_map(
        &mut self,
        tag: &[u8],
        key_type: TdfType,
        value_type: TdfType,
    ) -> DecodeResult<usize> {
        self.until_tag(tag, TdfType::Map)?;

        let k_type = self.read_type()?;
        let v_type = self.read_type()?;

        if k_type != key_type {
            return Err(DecodeError::InvalidType {
                expected: key_type,
                actual: k_type,
            });
        }

        if v_type != value_type {
            return Err(DecodeError::InvalidType {
                expected: value_type,
                actual: v_type,
            });
        }

        let count = self.read_usize()?;
        Ok(count)
    }
}

/// Majority of reading tests are merged into the writing tests
#[cfg(test)]
mod test {
    use super::TdfReader;

    /// Tests reading a byte from the reader
    #[test]
    fn test_read_byte() {
        for value in 0..255 {
            let buffer = &[value];
            let mut reader = TdfReader::new(buffer);
            let read_value = reader.read_byte().unwrap();
            assert_eq!(value, read_value);
        }
    }
}
