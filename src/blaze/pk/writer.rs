//! Writer buffer implementation for writing different kinds of tdf values
//! to byte form without creating a new structure [`TdfWriter`]

use super::{
    codec::{Encodable, ValueType},
    tag::TdfType,
    types::{VarInt, UNION_UNSET},
};

/// Writer implementation for writing values to an underlying buffer
/// this writer implementation provides functions for writing certain
/// data types in their Blaze format
#[derive(Default)]
pub struct TdfWriter {
    /// The buffer that will be written to
    pub buffer: Vec<u8>,
}

/// Macro for implementing VarInt encoding for a specific number type
/// to prevent the need for casting up to a u64 for the u16 and u32 types
macro_rules! impl_encode_var {
    ($value:ident, $output:ident) => {
        if $value < 64 {
            $output.write_byte($value as u8);
            return;
        }
        let mut byte: u8 = (($value & 63) as u8) | 128;
        $output.write_byte(byte);
        let mut cur_shift = $value >> 6;
        while cur_shift >= 128 {
            byte = ((cur_shift & 127) | 128) as u8;
            cur_shift >>= 7;
            $output.write_byte(byte);
        }
        $output.write_byte(cur_shift as u8)
    };
}

impl TdfWriter {
    /// Writes a single byte to the underlying buffer. This just
    /// appends the byte to the buffer.
    ///
    /// `value` The value to write
    #[inline]
    pub fn write_byte(&mut self, value: u8) {
        self.buffer.push(value)
    }

    /// Extends the underlying buffer with the provided slice
    /// value.
    ///
    /// `value` The slice value to write
    #[inline]
    pub fn write_slice(&mut self, value: &[u8]) {
        self.buffer.extend_from_slice(value);
    }

    /// Writes the value type byte of the provided TdfType
    ///
    /// `ty` The type to write
    #[inline]
    pub fn write_type(&mut self, ty: TdfType) {
        self.write_byte(ty as u8);
    }

    /// Writes a tag vvalue to the underlying buffer
    ///
    /// `tag`        The tag bytes to write
    /// `value_type` The value type for the tag
    pub fn tag(&mut self, tag: &[u8], value_type: TdfType) {
        let mut output: [u8; 4] = [0, 0, 0, value_type as u8];
        let length: usize = tag.len();
        if length > 0 {
            output[0] |= (tag[0] & 0x40) << 1;
            output[0] |= (tag[0] & 0x10) << 2;
            output[0] |= (tag[0] & 0x0F) << 2;
        }
        if length > 1 {
            output[0] |= (tag[1] & 0x40) >> 5;
            output[0] |= (tag[1] & 0x10) >> 4;
            output[1] |= (tag[1] & 0x0F) << 4;
        }
        if length > 2 {
            output[1] |= (tag[2] & 0x40) >> 3;
            output[1] |= (tag[2] & 0x10) >> 2;
            output[1] |= (tag[2] & 0x0C) >> 2;
            output[2] |= (tag[2] & 0x03) << 6;
        }
        if length > 3 {
            output[2] |= (tag[3] & 0x40) >> 1;
            output[2] |= tag[3] & 0x1F;
        }
        self.buffer.extend_from_slice(&output);
    }

    /// Writes a new tag to the buffer with a boolean as the
    /// tag value.
    ///
    /// `tag`   The tag to write
    /// `value` The tag value boolean
    pub fn tag_bool(&mut self, tag: &[u8], value: bool) {
        self.tag(tag, TdfType::VarInt);
        self.write_bool(value);
    }

    /// Writes a new tag where the value is a VarInt that is
    /// simply zero so the encoding can skip all the var int
    /// logic and directly write zero
    ///
    /// `tag` The tag to write
    pub fn tag_zero(&mut self, tag: &[u8]) {
        self.tag(tag, TdfType::VarInt);
        self.write_byte(0);
    }

    /// Writes a new tag where the value is a u8 value using
    /// the var int encoding
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_u8(&mut self, tag: &[u8], value: u8) {
        self.tag(tag, TdfType::VarInt);
        self.write_u8(value);
    }

    /// Writes a new tag where the value is a u16 value using
    /// the var int encoding
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_u16(&mut self, tag: &[u8], value: u16) {
        self.tag(tag, TdfType::VarInt);
        self.write_u16(value);
    }

    /// Writes a new tag where the value is a u32 value using
    /// the var int encoding
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_u32(&mut self, tag: &[u8], value: u32) {
        self.tag(tag, TdfType::VarInt);
        self.write_u32(value);
    }

    /// Writes a new tag where the value is a u64 value using
    /// the var int encoding
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_u64(&mut self, tag: &[u8], value: u64) {
        self.tag(tag, TdfType::VarInt);
        self.write_u64(value);
    }

    /// Writes a new tag where the value is a usize value using
    /// the var int encoding
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_usize(&mut self, tag: &[u8], value: usize) {
        self.tag(tag, TdfType::VarInt);
        self.write_usize(value);
    }

    /// Writes a new tag where the value is an empty string
    ///
    /// `tag` The tag to write
    pub fn tag_str_empty(&mut self, tag: &[u8]) {
        self.tag(tag, TdfType::String);
        self.write_empty_str();
    }

    /// Writes a new tag where the value is an empty blob.
    /// Empty blobs are simply encoded with a zero length
    ///
    /// `tag` The tag to write
    pub fn tag_empty_blob(&mut self, tag: &[u8]) {
        self.tag(tag, TdfType::Blob);
        self.buffer.push(0);
    }

    /// Writes a new tag where the value is a string.
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_str(&mut self, tag: &[u8], value: &str) {
        self.tag(tag, TdfType::String);
        self.write_str(value)
    }

    /// Writes a new tag indicating the start of a new group
    ///
    /// `tag` The tag to write
    pub fn tag_group(&mut self, tag: &[u8]) {
        self.tag(tag, TdfType::Group);
    }

    /// Writes the zero value that indicates the end of a group
    pub fn tag_group_end(&mut self) {
        self.buffer.push(0);
    }

    /// Writes a group opening tag and then completes the group function
    /// and closes the group tag
    ///
    /// `tag` The tag to write
    /// `gr`  The group closure
    #[inline]
    pub fn group<F>(&mut self, tag: &[u8], gr: F)
    where
        F: FnOnce(&mut Self),
    {
        self.tag_group(tag);
        gr(self);
        self.tag_group_end();
    }

    /// Writes a new tag indicating that a list is begining and writes the list
    /// type and length
    ///
    /// `tag`    The tag to write
    /// `ty`     The type of items being written after
    /// `length` The number of items that will be written
    pub fn tag_list_start(&mut self, tag: &[u8], ty: TdfType, length: usize) {
        self.tag(tag, TdfType::List);
        self.write_type(ty);
        self.write_usize(length);
    }

    /// Writes a new tag indicating that a union with the provided key is
    /// starting
    ///
    /// `tag` The tag to write
    /// `key` The key to write
    pub fn tag_union_start(&mut self, tag: &[u8], key: u8) {
        self.tag(tag, TdfType::Union);
        self.buffer.push(key);
    }

    /// Writes a new union tag with its value
    ///
    /// `tag`       The tag to write
    /// `key`       The key of the union
    /// `value_tag` The tag for the value
    /// `value`     The value to write
    pub fn tag_union_value<C: Encodable + ValueType>(
        &mut self,
        tag: &[u8],
        key: u8,
        value_tag: &[u8],
        value: &C,
    ) {
        self.tag_union_start(tag, key);
        self.tag(value_tag, C::value_type());
        value.encode(self);
    }

    /// Writes a new tag indicating a union with no value
    ///
    /// `tag` The tag to write
    pub fn tag_union_unset(&mut self, tag: &[u8]) {
        self.tag_union_start(tag, UNION_UNSET);
    }

    /// Writes a tag and its value where the value implements ValueType
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_value<C: Encodable + ValueType>(&mut self, tag: &[u8], value: &C) {
        self.tag(tag, C::value_type());
        value.encode(self);
    }

    /// Writes a tag for indiciating a list with no contents
    ///
    /// `tag` The tag to write
    /// `ty`  The type of the empty list
    pub fn tag_list_empty(&mut self, tag: &[u8], ty: TdfType) {
        self.tag(tag, TdfType::List);
        self.write_type(ty);
        self.buffer.push(0);
    }

    /// Slices are already borrowed so they confuse the `tag_value` type using this
    /// function instead makes them work
    pub fn tag_slice_list<C: Encodable + ValueType>(&mut self, tag: &[u8], value: &[C]) {
        self.tag(tag, TdfType::List);
        value.encode(self);
    }

    /// Writes a tag for indiciating a var int list with no contents
    ///
    /// `tag` The tag to write
    pub fn tag_var_int_list_empty(&mut self, tag: &[u8]) {
        self.tag(tag, TdfType::VarIntList);
        self.buffer.push(0);
    }

    /// Writes a tag indicating that a map will be written for the
    /// provided types and length
    ///
    /// `tag`    The tag to write
    /// `key`    The key tdf type
    /// `value`  The value tdf type
    /// `length` The total number of entires that will be written
    pub fn tag_map_start(&mut self, tag: &[u8], key: TdfType, value: TdfType, length: usize) {
        self.tag(tag, TdfType::Map);
        self.write_type(key);
        self.write_type(value);
        self.write_usize(length);
    }

    /// Writes a list of tuples as a map of key value paris
    ///
    /// `tag`    The tag to write
    /// `values` The tuples of key value pairs to write
    pub fn tag_map_tuples<K, V>(&mut self, tag: &[u8], values: &[(K, V)])
    where
        K: Encodable + ValueType,
        V: Encodable + ValueType,
    {
        self.tag_map_start(tag, K::value_type(), V::value_type(), values.len());
        for (key, value) in values {
            key.encode(self);
            value.encode(self);
        }
    }

    /// Writes a tag with a pair of values
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_pair<A, B>(&mut self, tag: &[u8], value: (A, B))
    where
        A: VarInt,
        B: VarInt,
    {
        self.tag(tag, TdfType::Pair);
        value.encode(self);
    }

    /// Writes a tag with a triple of values
    ///
    /// `tag`   The tag to write
    /// `value` The value to write
    pub fn tag_triple<A, B, C>(&mut self, tag: &[u8], value: (A, B, C))
    where
        A: VarInt,
        B: VarInt,
        C: VarInt,
    {
        self.tag(tag, TdfType::Triple);
        value.encode(self);
    }

    /// Writes an empty string. This is simply two bytes a 1 and a 0 which
    /// indicate a string consisting of only a null terminator
    pub fn write_empty_str(&mut self) {
        self.buffer.extend_from_slice(&[1, 0])
    }

    /// Writes 32 bit float value to the underlying buffer in
    /// big-endian byte order.
    ///
    /// `value` The float value to write
    pub fn write_f32(&mut self, value: f32) {
        let bytes: [u8; 4] = value.to_be_bytes();
        self.buffer.extend_from_slice(&bytes);
    }

    /// Writes a u8 value using the VarInt encoding
    ///
    /// `value` The value to write
    pub fn write_u8(&mut self, value: u8) {
        // Values < 64 are directly appended to buffer
        if value < 64 {
            self.buffer.push(value);
            return;
        }
        self.buffer.push((value & 63) | 128);
        self.buffer.push(value >> 6);
    }

    /// Writes a u16 value using the VarInt encoding
    ///
    /// `value` The value to write
    pub fn write_u16(&mut self, value: u16) {
        if value < 64 {
            self.buffer.push(value as u8);
            return;
        }
        let mut byte: u8 = ((value & 63) as u8) | 128;
        let mut shift: u16 = value >> 6;
        self.buffer.push(byte);
        byte = ((shift & 127) | 128) as u8;
        shift >>= 7;
        self.buffer.push(byte);
        self.buffer.push(shift as u8);
    }

    /// Writes a u32 value using the VarInt encoding
    ///
    /// `value` The value to write
    pub fn write_u32(&mut self, value: u32) {
        impl_encode_var!(value, self);
    }

    /// Writes a u64 value using the VarInt encoding
    ///
    /// `value` The value to write
    pub fn write_u64(&mut self, value: u64) {
        impl_encode_var!(value, self);
    }

    /// Writes a usize value using the VarInt encoding
    ///
    /// `value` The value to write
    pub fn write_usize(&mut self, value: usize) {
        impl_encode_var!(value, self);
    }

    /// Writes a string to the underlying buffer. The bytes
    /// are encoded an a null terminator is appended to the
    /// end then the size and bytes are written to the buffer
    ///
    /// `value` The string value to write
    pub fn write_str(&mut self, value: &str) {
        let mut bytes = value.as_bytes().to_vec();
        match bytes.last() {
            // Ignore if already null terminated
            Some(0) => {}
            // Null terminate
            _ => bytes.push(0),
        }

        self.write_usize(bytes.len());
        self.write_slice(&bytes);
    }

    /// Writes a boolean value which uses the VarInt encoding
    /// except because the values are < 64 they are just directly
    /// appended as bytes
    pub fn write_bool(&mut self, value: bool) {
        match value {
            false => self.buffer.push(0),
            true => self.buffer.push(1),
        }
    }

    /// Writes the header for a map in order to begin writing map values
    ///
    /// `key_type`   The type of the map keys
    /// `value_type` The type of the map values
    /// `length`     The total number of items that will be written
    pub fn write_map_header(&mut self, key_type: TdfType, value_type: TdfType, length: usize) {
        self.write_type(key_type);
        self.write_type(value_type);
        self.write_usize(length);
    }

    /// Clears the contents of the underlying buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Implementation for converting tdf writer into its underlying buffer with from
impl From<TdfWriter> for Vec<u8> {
    fn from(value: TdfWriter) -> Self {
        value.buffer
    }
}

#[cfg(test)]
mod test {
    use super::TdfWriter;
    use crate::blaze::pk::{codec::Encodable, reader::TdfReader, tag::TdfType, types::UNION_UNSET};

    /// Test for ensuring some common tags of different
    /// length are encoded to the correct values. The tags
    /// being tested are.
    ///
    /// - TEST
    /// - VALU
    /// - IP
    /// - A
    #[test]
    fn test_tagging() {
        /// Slice of some common tags and their encoded bytes
        const TAGS: [(&[u8], &[u8]); 4] = [
            (b"TEST", &[210, 92, 244]),
            (b"VALU", &[218, 27, 53]),
            (b"IP", &[167, 0, 0]),
            (b"A", &[132, 0, 0]),
        ];
        let mut writer = TdfWriter::default();
        for (tag, expected) in TAGS {
            writer.tag(tag, TdfType::VarInt);
            assert_eq!(
                writer.buffer.len(),
                4,
                "Ensuring that buffer length is 4 bytes"
            );
            assert_eq!(
                &writer.buffer[0..3],
                expected,
                "Checking the tag bytes match"
            );
            assert_eq!(
                writer.buffer[3],
                TdfType::VarInt as u8,
                "Checking that tdf type matches VarInt"
            );
            writer.clear();
        }
    }

    /// Tests writing a single byte writes the correct value.
    /// Writes 0 - 255 and checks each value matches
    #[test]
    fn test_write_byte() {
        let mut writer = TdfWriter::default();
        for i in 0..255 {
            writer.write_byte(i);
            assert_eq!(writer.buffer.len(), 1);
            assert_eq!(writer.buffer[0], i);
            writer.clear();
        }
    }

    /// Tests writing a slice of bytes and ensuring that
    /// the slice was appended
    #[test]
    fn test_write_slice() {
        const SLICE: &[u8] = &[0, 125, 21, 1, 3, 15, 50, 30];
        let mut writer = TdfWriter::default();
        writer.write_slice(SLICE);
        assert_eq!(&writer.buffer, SLICE)
    }

    /// Tests writing all the type values
    #[test]
    fn test_write_type() {
        const TYPES: [TdfType; 11] = [
            TdfType::VarInt,
            TdfType::String,
            TdfType::Blob,
            TdfType::Group,
            TdfType::List,
            TdfType::Map,
            TdfType::Union,
            TdfType::VarIntList,
            TdfType::Pair,
            TdfType::Triple,
            TdfType::Float,
        ];
        let mut writer = TdfWriter::default();
        for ty in TYPES {
            writer.write_type(ty);
            assert_eq!(writer.buffer.len(), 1);
            assert_eq!(writer.buffer[0], ty as u8);
            writer.clear();
        }
    }

    /// Tests tagging a boolean value
    #[test]
    fn test_tag_bool() {
        // Possible boolean values and their expected u8 value
        const VALUES: [(bool, u8); 2] = [(true, 1), (false, 0)];
        let mut writer = TdfWriter::default();
        for (value, expected) in VALUES {
            writer.tag_bool(b"TEST", value);
            assert_eq!(writer.buffer.len(), 5);
            assert_eq!(writer.buffer[3], TdfType::VarInt as u8);
            assert_eq!(writer.buffer[4], expected);
            writer.clear();
        }
    }

    /// Tests tagging a zero value
    #[test]
    fn test_tag_zero() {
        let mut writer = TdfWriter::default();
        writer.tag_zero(b"TEST");
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::VarInt as u8);
        assert_eq!(writer.buffer[4], 0);
    }

    /// Tests tagging all the different u8 values. Writing and
    /// then reading them to see if they are correct
    ///
    #[test]
    fn test_tag_u8() {
        let mut writer = TdfWriter::default();
        for value in u8::MIN..u8::MAX {
            writer.tag_u8(b"TEST", value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u8 = reader.tag(b"TEST").expect("Failed to decode tag u8 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging all the different u16 values. Writing and
    /// then reading them to see if they are correct
    ///
    #[test]
    fn test_tag_u16() {
        let mut writer = TdfWriter::default();
        for value in u16::MIN..u16::MAX {
            writer.tag_u16(b"TEST", value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u16 = reader.tag(b"TEST").expect("Failed to decode tag u16 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of u32 values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_tag_u32() {
        let mut writer = TdfWriter::default();
        for value in (u32::MAX - 65535)..u32::MAX {
            writer.tag_u32(b"TEST", value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u32 = reader.tag(b"TEST").expect("Failed to decode tag u32 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of u64 values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_tag_u64() {
        let mut writer = TdfWriter::default();
        for value in (u64::MAX - 65535)..u64::MAX {
            writer.tag_u64(b"TEST", value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u64 = reader.tag(b"TEST").expect("Failed to decode tag u64 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of usize values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_tag_usize() {
        let mut writer = TdfWriter::default();
        for value in (usize::MAX - 65535)..usize::MAX {
            writer.tag_usize(b"TEST", value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: usize = reader
                .tag(b"TEST")
                .expect("Failed to decode tag usize value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging an empty string
    #[test]
    fn test_tag_str_empty() {
        let mut writer = TdfWriter::default();
        writer.tag_str_empty(b"TEST");
        assert_eq!(writer.buffer.len(), 6);
        assert_eq!(writer.buffer[3], TdfType::String as u8);
        assert_eq!(&writer.buffer[4..6], &[1, 0]);
    }

    /// Tests tagging an empty blob
    #[test]
    fn test_tag_empty_blob() {
        let mut writer = TdfWriter::default();
        writer.tag_empty_blob(b"TEST");
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::Blob as u8);
        assert_eq!(writer.buffer[4], 0);
    }

    /// Tests tagging a string value
    #[test]
    fn test_tag_str() {
        const TEXT: &str = "Test string";
        const TEXT_BYTES: &[u8] = b"Test string\0";

        let mut writer = TdfWriter::default();
        writer.tag_str(b"TEST", TEXT);

        // 3) tag 1) type 1) length TEXT.len()) bytes 1) terminator
        assert_eq!(writer.buffer.len(), 5 + TEXT.len() + 1);
        assert_eq!(writer.buffer[3], TdfType::String as u8);

        let length_bytes = (TEXT.len() + 1).encode_bytes();

        assert_eq!(&writer.buffer[4..4 + length_bytes.len()], &length_bytes);
        assert_eq!(&writer.buffer[4 + length_bytes.len()..], TEXT_BYTES);

        let mut reader = TdfReader::new(&writer.buffer);
        let value: String = reader.tag(b"TEST").unwrap();

        assert_eq!(value, TEXT)
    }

    /// Tests tagging a group
    #[test]
    fn test_tag_group() {
        let mut writer = TdfWriter::default();
        writer.tag_group(b"TEST");
        writer.tag_group_end();

        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::Group as u8);
        assert_eq!(writer.buffer[4], 0);
    }

    /// Tests tagging a group with the closure way
    #[test]
    fn test_tag_group_alt() {
        let mut writer = TdfWriter::default();

        writer.group(b"TEST", |_| {});

        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::Group as u8);
        assert_eq!(writer.buffer[4], 0);
    }

    /// Tests tagging a union
    #[test]
    fn test_tag_union() {
        let mut writer = TdfWriter::default();
        writer.tag_union_start(b"TEST", 15);
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::Union as u8);
        assert_eq!(writer.buffer[4], 15);
        writer.clear();

        writer.tag_union_unset(b"TEST");
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::Union as u8);
        assert_eq!(writer.buffer[4], UNION_UNSET);
        writer.clear();

        writer.tag_union_value(b"TEST", 5, b"TEST2", &15);
        assert_eq!(writer.buffer.len(), 10);
        assert_eq!(writer.buffer[3], TdfType::Union as u8);
        assert_eq!(writer.buffer[4], 5);
        assert_eq!(writer.buffer[8], TdfType::VarInt as u8);
        assert_eq!(writer.buffer[9], 15);
    }

    /// Tests tagging for value types
    #[test]
    fn test_tag_value() {
        let mut writer = TdfWriter::default();
        writer.tag_value(b"TEST", &12u8);
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::VarInt as u8);
        assert_eq!(writer.buffer[4], 12);
    }

    /// Tests writing an empty list
    #[test]
    fn test_tag_list_empty() {
        let mut writer = TdfWriter::default();
        writer.tag_list_empty(b"TEST", TdfType::VarInt);
        assert_eq!(writer.buffer.len(), 6);
        assert_eq!(writer.buffer[3], TdfType::List as u8);
        assert_eq!(writer.buffer[4], TdfType::VarInt as u8);
        assert_eq!(writer.buffer[5], 0);
    }

    /// Tests writing an empty list of varints
    #[test]
    fn test_tag_var_int_list_empty() {
        let mut writer = TdfWriter::default();
        writer.tag_var_int_list_empty(b"TEST");
        assert_eq!(writer.buffer.len(), 5);
        assert_eq!(writer.buffer[3], TdfType::VarIntList as u8);
        assert_eq!(writer.buffer[4], 0);
    }

    /// Tests writing a map tag and details
    #[test]
    fn test_tag_map_start() {
        let mut writer = TdfWriter::default();
        writer.tag_map_start(b"TEST", TdfType::String, TdfType::VarInt, 0);
        assert_eq!(writer.buffer.len(), 7);
        assert_eq!(writer.buffer[3], TdfType::Map as u8);
        assert_eq!(writer.buffer[4], TdfType::String as u8);
        assert_eq!(writer.buffer[5], TdfType::VarInt as u8);
        assert_eq!(writer.buffer[6], 0);
    }

    /// Tests writing a pair
    #[test]
    fn test_tag_pair() {
        let mut writer = TdfWriter::default();
        writer.tag_pair(b"TEST", (5, 10));
        assert_eq!(writer.buffer.len(), 6);
        assert_eq!(writer.buffer[3], TdfType::Pair as u8);
        assert_eq!(writer.buffer[4], 5);
        assert_eq!(writer.buffer[5], 10);
    }

    /// Tests writing a triple
    #[test]
    fn test_tag_triple() {
        let mut writer = TdfWriter::default();
        writer.tag_triple(b"TEST", (5, 10, 50));
        assert_eq!(writer.buffer.len(), 7);
        assert_eq!(writer.buffer[3], TdfType::Triple as u8);
        assert_eq!(writer.buffer[4], 5);
        assert_eq!(writer.buffer[5], 10);
        assert_eq!(writer.buffer[6], 50);
    }

    /// Tests writing an empty string
    #[test]
    fn test_write_empty_str() {
        let mut writer = TdfWriter::default();
        writer.write_empty_str();
        assert_eq!(&writer.buffer, &[1, 0]);
    }

    /// Tests writing float values
    #[test]
    fn test_write_f32() {
        let mut writer = TdfWriter::default();
        let mut value: f32 = 1.0;
        while value < f32::MAX {
            let expected = value.to_be_bytes();
            writer.write_f32(value);
            assert_eq!(&writer.buffer, &expected);
            writer.clear();
            value *= 2.0;
        }
    }

    /// Tests writing all the different u8 values
    #[test]
    fn test_write_u8() {
        let mut writer = TdfWriter::default();
        for value in u8::MIN..u8::MAX {
            writer.write_u8(value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u8 = reader.read_u8().expect("Failed to decode tag u8 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging all the different u16 values. Writing and
    /// then reading them to see if they are correct
    ///
    #[test]
    fn test_write_u16() {
        let mut writer = TdfWriter::default();
        for value in u16::MIN..u16::MAX {
            writer.write_u16(value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u16 = reader.read_u16().expect("Failed to decode tag u16 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of u32 values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_write_u32() {
        let mut writer = TdfWriter::default();
        for value in (u32::MAX - 65535)..u32::MAX {
            writer.write_u32(value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u32 = reader.read_u32().expect("Failed to decode tag u32 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of u64 values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_write_u64() {
        let mut writer = TdfWriter::default();
        for value in (u64::MAX - 65535)..u64::MAX {
            writer.write_u64(value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: u64 = reader.read_u64().expect("Failed to decode tag u64 value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a bunch of usize values. Writing and
    /// then reading them to see if they are correct
    /// (Takes the last 65535 numbers)
    #[test]
    fn test_write_usize() {
        let mut writer = TdfWriter::default();
        for value in (usize::MAX - 65535)..usize::MAX {
            writer.write_usize(value);
            let mut reader = TdfReader::new(&writer.buffer);
            let decoded: usize = reader
                .read_usize()
                .expect("Failed to decode tag usize value");
            assert_eq!(value, decoded);
            writer.clear();
        }
    }

    /// Tests tagging a boolean value
    #[test]
    fn test_write_bool() {
        // Possible boolean values and their expected u8 value
        const VALUES: [(bool, u8); 2] = [(true, 1), (false, 0)];
        let mut writer = TdfWriter::default();
        for (value, expected) in VALUES {
            writer.write_bool(value);
            assert_eq!(writer.buffer.len(), 1);
            assert_eq!(writer.buffer[0], expected);
            writer.clear();
        }
    }

    /// Tests tagging a string value
    #[test]
    fn test_write_str() {
        const TEXT: &str = "Test string";
        const TEXT_BYTES: &[u8] = b"Test string\0";

        let mut writer = TdfWriter::default();
        writer.write_str(TEXT);

        // 3) tag 1) type 1) length TEXT.len()) bytes 1) terminator
        assert_eq!(writer.buffer.len(), 1 + TEXT.len() + 1);

        let length_bytes = (TEXT.len() + 1).encode_bytes();

        assert_eq!(&writer.buffer[..length_bytes.len()], &length_bytes);
        assert_eq!(&writer.buffer[length_bytes.len()..], TEXT_BYTES);

        let mut reader = TdfReader::new(&writer.buffer);
        let value: String = reader.read_string().unwrap();

        assert_eq!(value, TEXT)
    }

    /// Tests clearing the buffer
    #[test]
    fn test_clear() {
        let mut writer = TdfWriter::default();
        writer.write_empty_str();
        writer.write_empty_str();
        writer.write_empty_str();
        writer.write_empty_str();
        writer.clear();
        assert_eq!(writer.buffer.len(), 0)
    }
}
