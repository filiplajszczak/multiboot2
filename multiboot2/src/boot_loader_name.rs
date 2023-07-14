use crate::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use core::str::Utf8Error;
#[cfg(feature = "builder")]
use {crate::builder::BoxedDst, alloc::vec::Vec};

const METADATA_SIZE: usize = size_of::<TagTypeId>() + size_of::<u32>();

/// The bootloader name tag.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootLoaderNameTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl BootLoaderNameTag {
    #[cfg(feature = "builder")]
    pub fn new(name: &str) -> BoxedDst<Self> {
        let mut bytes: Vec<_> = name.bytes().collect();
        if !bytes.ends_with(&[0]) {
            // terminating null-byte
            bytes.push(0);
        }
        BoxedDst::new(&bytes)
    }

    /// Reads the name of the bootloader that is booting the kernel as Rust
    /// string slice without the null-byte.
    ///
    /// For example, this returns `"GRUB 2.02~beta3-5"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *mut BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(tag) = boot_info.boot_loader_name_tag() {
    ///     assert_eq!(Ok("GRUB 2.02~beta3-5"), tag.name());
    /// }
    /// ```
    pub fn name(&self) -> Result<&str, Utf8Error> {
        Tag::get_dst_str_slice(&self.name)
    }
}

impl Debug for BootLoaderNameTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("name", &self.name())
            .finish()
    }
}

impl TagTrait for BootLoaderNameTag {
    const ID: TagType = TagType::BootLoaderName;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use crate::{BootLoaderNameTag, Tag, TagTrait, TagType};

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in little endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        let size = (4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::BootLoaderName.val()).to_le_bytes()),
            &size.to_le_bytes(),
            MSG.as_bytes(),
            // Null Byte
            &[0],
        ]
        .iter()
        .flat_map(|bytes| bytes.iter())
        .copied()
        .collect()
    }

    /// Tests to parse a string with a terminating null byte from the tag (as the spec defines).
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_str() {
        let tag = get_bytes();
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<BootLoaderNameTag>();
        assert_eq!({ tag.typ }, TagType::BootLoaderName);
        assert_eq!(tag.name().expect("must be valid UTF-8"), MSG);
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        let tag = BootLoaderNameTag::new(MSG);
        let bytes = tag.as_bytes();
        assert_eq!(bytes, get_bytes());
        assert_eq!(tag.name(), Ok(MSG));

        // test also some bigger message
        let tag = BootLoaderNameTag::new("AbCdEfGhUjK YEAH");
        assert_eq!(tag.name(), Ok("AbCdEfGhUjK YEAH"));
        let tag = BootLoaderNameTag::new("AbCdEfGhUjK YEAH".repeat(42).as_str());
        assert_eq!(tag.name(), Ok("AbCdEfGhUjK YEAH".repeat(42).as_str()));
    }
}
