//! GPU textures shared by the worlds alive at once on one graphics context.
//!
//! A map change builds the next world while another is still alive
//! (`session_transition.rs`: on a server an empty one stands in for the old map; the
//! menu world and demos keep the old one), so for those seconds two worlds hold video
//! memory.
//! Textures that come out the same on both (the saber glows, the game fonts, the
//! effect atlas, the static menu art, and map textures from the same files) were
//! made twice; on a 6 GB card that second
//! copy was part of what ran out ("world installer stopped unexpectedly",
//! 18/10/2026). A world being built on this thread ([`building`]) now takes an
//! existing copy from its context's [`Cache`], keyed by what the texture is made
//! from. An entry lives while a world that took it lives ([`Holder`]) and is freed
//! with the last one.
//!
//! Only textures nothing writes after creation are shared; the icon atlas and the
//! animated menu art are rewritten while a world runs and stay per world. Each
//! context has its own cache: a graphics reload's new context may make the same
//! texture differently.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static NEXT_WORLD: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// The cache and world of the build running on this thread.
    static BUILDING: RefCell<Option<(Arc<Cache>, u64)>> = const { RefCell::new(None) };
}

struct Entry {
    value: Box<dyn Any + Send + Sync>,
    /// The worlds that took it.
    worlds: Vec<u64>,
}

/// One context's shared textures, by kind and content key.
#[derive(Default)]
pub(crate) struct Cache {
    entries: Mutex<HashMap<(&'static str, u64), Entry>>,
}

impl Cache {
    fn lock(&self) -> MutexGuard<'_, HashMap<(&'static str, u64), Entry>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Drop `world`'s claims; entries no world holds any more are freed.
    fn release(&self, world: u64) {
        self.lock().retain(|_, entry| {
            entry.worlds.retain(|&holder| holder != world);
            !entry.worlds.is_empty()
        });
    }

    /// Entries alive, for tests.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.lock().len()
    }
}

/// A world's claim on its context's shared textures, released when it drops.
pub(crate) struct Holder {
    cache: Arc<Cache>,
    world: u64,
}

impl Drop for Holder {
    fn drop(&mut self) {
        self.cache.release(self.world);
    }
}

/// While alive, textures made on this thread through [`share`] are shared through
/// the cache [`building`] was given.
pub(crate) struct Building {
    previous: Option<(Arc<Cache>, u64)>,
}

impl Drop for Building {
    fn drop(&mut self) {
        let previous = self.previous.take();
        BUILDING.with(|building| *building.borrow_mut() = previous);
    }
}

/// Start a world's build on this thread: its holder (kept by the world) and the scope
/// in which its textures are shared.
pub(crate) fn building(cache: &Arc<Cache>) -> (Holder, Building) {
    let world = NEXT_WORLD.fetch_add(1, Ordering::Relaxed);
    let previous =
        BUILDING.with(|building| building.borrow_mut().replace((Arc::clone(cache), world)));
    (
        Holder {
            cache: Arc::clone(cache),
            world,
        },
        Building { previous },
    )
}

/// A world is being built on this thread ([`building`]): the world installer, not the
/// render thread.
pub(crate) fn in_build() -> bool {
    BUILDING.with(|building| building.borrow().is_some())
}

/// The `kind` texture made from content `key`: another world's copy when one is alive
/// on this context, else `make`'s. Outside a build, always `make`'s.
pub(crate) fn share<T>(kind: &'static str, key: u64, make: impl FnOnce() -> T) -> T
where
    T: Clone + Send + Sync + 'static,
{
    let Some((cache, world)) = BUILDING.with(|building| building.borrow().clone()) else {
        return make();
    };
    if let Some(value) = take(&cache, world, kind, key) {
        return value;
    }
    // Made outside the lock: uploads take time and may share textures of their own.
    let value = make();
    cache.lock().insert(
        (kind, key),
        Entry {
            value: Box::new(value.clone()),
            worlds: vec![world],
        },
    );
    value
}

/// The `kind` texture made from content `key` if a world of this build's context holds
/// one, without making it: a caller can then skip preparing what it would be made from.
pub(crate) fn find<T>(kind: &'static str, key: u64) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    let (cache, world) = BUILDING.with(|building| building.borrow().clone())?;
    take(&cache, world, kind, key)
}

/// `world`'s claim on the entry, and its value.
fn take<T: Clone + 'static>(cache: &Cache, world: u64, kind: &'static str, key: u64) -> Option<T> {
    let mut entries = cache.lock();
    let entry = entries.get_mut(&(kind, key))?;
    let value = entry.value.downcast_ref::<T>()?.clone();
    if !entry.worlds.contains(&world) {
        entry.worlds.push(world);
    }
    Some(value)
}

/// A content key over `parts` (pixels, sizes, options).
pub(crate) fn key(parts: &[&[u8]]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_world_takes_the_first_ones_copy() {
        let cache = Arc::new(Cache::default());
        let made = std::cell::Cell::new(0);
        let make = || {
            made.set(made.get() + 1);
            Arc::new(made.get())
        };
        let (first, scope) = building(&cache);
        let a = share("t", 7, make);
        drop(scope);
        let (second, scope) = building(&cache);
        let b = share("t", 7, make);
        drop(scope);
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(made.get(), 1);
        drop(first);
        assert_eq!(cache.len(), 1, "the second world still holds it");
        drop(second);
        assert_eq!(cache.len(), 0, "freed with the last world");
    }

    #[test]
    fn find_claims_without_making() {
        let cache = Arc::new(Cache::default());
        let (first, scope) = building(&cache);
        assert_eq!(find::<u32>("t", 1), None);
        share("t", 1, || 5_u32);
        drop(scope);
        let (second, _scope) = building(&cache);
        assert_eq!(find::<u32>("t", 1), Some(5));
        drop(first);
        assert_eq!(cache.len(), 1, "found by the second world, which holds it");
        drop(second);
    }

    #[test]
    fn outside_a_build_nothing_is_shared() {
        let made = std::cell::Cell::new(0);
        let make = || made.set(made.get() + 1);
        share("t", 1, make);
        share("t", 1, make);
        assert_eq!(made.get(), 2);
    }

    #[test]
    fn another_key_or_kind_is_another_texture() {
        let cache = Arc::new(Cache::default());
        let (_world, _scope) = building(&cache);
        assert_eq!(share("t", 1, || 1), 1);
        assert_eq!(share("t", 2, || 2), 2);
        assert_eq!(share("u", 1, || 3), 3);
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn a_nested_build_restores_the_outer_one() {
        let outer_cache = Arc::new(Cache::default());
        let inner_cache = Arc::new(Cache::default());
        let (_outer, _outer_scope) = building(&outer_cache);
        {
            let (_inner, _inner_scope) = building(&inner_cache);
            share("t", 1, || 1);
        }
        share("t", 2, || 2);
        assert_eq!(inner_cache.len(), 0, "the inner world dropped");
        assert_eq!(outer_cache.len(), 1);
    }
}
