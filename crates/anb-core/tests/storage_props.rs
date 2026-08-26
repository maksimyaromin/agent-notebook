//! Property tests at the Storage seam: strings in, exact strings out.

use anb_core::{MemoryStorage, Storage};
use proptest::prelude::*;

proptest! {
    #[test]
    fn write_then_read_returns_exact_bytes(content in ".*") {
        let mut storage = MemoryStorage::new();
        storage.write("tasks/x.md", &content).unwrap();
        prop_assert_eq!(storage.read("tasks/x.md").unwrap(), content);
    }

    #[test]
    fn mutating_one_record_leaves_every_other_byte_unchanged(
        a in ".*",
        b in ".*",
        a2 in ".*",
    ) {
        let mut storage = MemoryStorage::from_files([("tasks/a.md", a), ("tasks/b.md", b.clone())]);
        storage.write("tasks/a.md", &a2).unwrap();
        prop_assert_eq!(storage.read("tasks/b.md").unwrap(), b);
    }
}
