use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES_MEMORY_ID: MemoryId = MemoryId::new(0);
// A memory for canister metadata
const METADATA_MEMORY_ID: MemoryId = MemoryId::new(1);

pub type VMemory = VirtualMemory<DefaultMemoryImpl>;

/// A helper type implementing Storable for all
/// serde-serializable types using the CBOR encoding.
#[derive(Default, Clone)]
pub struct Cbor<T>(pub T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone;

impl<T> std::ops::Deref for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Storable for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self.0, &mut buf).unwrap();
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(ciborium::de::from_reader(bytes.as_ref()).unwrap())
    }
}

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_upgrades_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES_MEMORY_ID))
}

pub fn get_metadata_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(METADATA_MEMORY_ID))
}
