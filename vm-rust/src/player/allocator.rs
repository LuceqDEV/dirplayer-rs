use std::sync::Arc;

use async_std::sync::Mutex;
use nohash_hasher::IntMap;

use crate::{console_warn, director::lingo::datum::Datum};

use super::{DatumId, DatumRef, ScriptError, VOID_DATUM_REF};
use lazy_static::lazy_static;


struct DatumRefEntry {
  pub id: u32,
  pub ref_count: u32,
  pub datum: Datum,
}

pub trait DatumAllocatorTrait {
  fn alloc_datum(&mut self, datum: Datum) -> Result<DatumRef, ScriptError>;
  fn get_datum(&self, id: &DatumRef) -> &Datum;
  fn get_datum_mut(&mut self, id: &DatumRef) -> &mut Datum;
  fn on_datum_ref_added(&mut self, id: DatumId);
  fn on_datum_ref_dropped(&mut self, id: DatumId);
  fn reset(&mut self);
}

pub struct DatumAllocator {
  datums: IntMap<u32, DatumRefEntry>,
  datum_id_counter: u32,
  void_datum: Datum,
}

const MAX_DATUM_ID: DatumId = u32::MAX;

impl DatumAllocator {
  pub fn default() -> Self {
    DatumAllocator {
      datums: IntMap::default(),
      datum_id_counter: 0,
      void_datum: Datum::Void,
    }
  }

  fn get_free_id(&self) -> Option<DatumId> {
    if !self.datums.contains_key(&self.datum_id_counter) {
      Some(self.datum_id_counter)
    } else if self.datum_id_counter + 1 < MAX_DATUM_ID {
      Some(self.datum_id_counter + 1)
    } else {
      console_warn!("Maxium datum id reached");
      let first_free_id = (1..MAX_DATUM_ID).find(|id| !self.datums.contains_key(&id));
      first_free_id
    }
  }

  fn dealloc_datum(&mut self, id: DatumId) {
    console_warn!("deallocating datum {}", id);
    self.datums.remove(&id);
  }
}

impl DatumAllocatorTrait for DatumAllocator {
  fn alloc_datum(&mut self, datum: Datum) -> Result<DatumRef, ScriptError> {
    if datum.is_void() {
      return Ok(VOID_DATUM_REF.clone());
    }
    
    if let Some(id) = self.get_free_id() {
      let entry = DatumRefEntry {
        id,
        ref_count: 1,
        datum,
      };
      self.datum_id_counter += 1;
      self.datums.insert(id, entry);
      Ok(DatumRef::from_id(id))
    } else {
      Err(ScriptError::new("Failed to allocate datum".to_string()))
    }
  }

  fn get_datum(&self, id: &DatumRef) -> &Datum {
    match id {
      DatumRef::Ref(id) => {
        let entry = self.datums.get(id).unwrap();
        &entry.datum
      }
      DatumRef::Void => &Datum::Void,
    }
  }

  fn get_datum_mut(&mut self, id: &DatumRef) -> &mut Datum {
    match id {
      DatumRef::Ref(id) => {
        let entry = self.datums.get_mut(id).unwrap();
        &mut entry.datum
      }
      DatumRef::Void => &mut self.void_datum,
    }
  }

  fn on_datum_ref_added(&mut self, id: DatumId) {
    let entry = self.datums.get_mut(&id).unwrap();
    entry.ref_count += 1;
  }

  fn on_datum_ref_dropped(&mut self, id: DatumId) {
    let entry = self.datums.get_mut(&id).unwrap();
    entry.ref_count -= 1;
    if entry.ref_count <= 0 {
      self.dealloc_datum(id);
    }
  }

  fn reset(&mut self) {
    self.datums.clear();
    self.datum_id_counter = 0;
  }
}

// lazy_static! {
//   pub static ref DATUM_ALLOCATOR: Arc<Mutex<DatumAllocator>> = Arc::new(Mutex::new(DatumAllocator::default()));
// }

// pub fn reserve_allocator_mut<F, R>(f: F) -> R
// where
//   F: FnOnce(&mut DatumAllocator) -> R,
// {
//   let mut allocator = DATUM_ALLOCATOR.try_lock().unwrap();
//   f(&mut allocator)
// }

// pub fn reserve_allocator_ref<F, R>(f: F) -> R
// where
//   F: FnOnce(&DatumAllocator) -> R,
// {
//   let allocator = DATUM_ALLOCATOR.try_lock().unwrap();
//   f(&allocator)
// }

// pub fn alloc_datum(datum: Datum) -> Result<DatumRef, ScriptError> {
//   reserve_allocator_mut(|allocator| allocator.alloc_datum(datum))
// }

// pub fn force_alloc_datum(datum: Datum) -> DatumRef {
//   alloc_datum(datum).unwrap()
// }