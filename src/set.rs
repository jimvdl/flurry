//! A concurrent hash set.
//!
//! See `HashSet` for details.

use crate::iter::Keys;
use crate::reclaim::Guard;
use crate::HashMap;
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};
use std::iter::FromIterator;

/// A concurrent hash set implemented as a `HashMap` where the value is `()`.
///
/// # Examples
///
/// ```
/// use flurry::HashSet;
///
/// // Initialize a new hash set.
/// let books = HashSet::new();
/// let guard = books.guard();
///
/// // Add some books
/// books.insert("Fight Club", &guard);
/// books.insert("Three Men In A Raft", &guard);
/// books.insert("The Book of Dust", &guard);
/// books.insert("The Dry", &guard);
///
/// // Check for a specific one.
/// if !books.contains(&"The Drunken Botanist", &guard) {
///     println!("We don't have The Drunken Botanist.");
/// }
///
/// // Remove a book.
/// books.remove(&"Three Men In A Raft", &guard);
///
/// // Iterate over everything.
/// for book in books.iter(&guard) {
///     println!("{}", book);
/// }
/// ```
pub struct HashSet<T, S = crate::DefaultHashBuilder> {
    pub(crate) map: HashMap<T, (), S>,
}

impl<T> HashSet<T, crate::DefaultHashBuilder> {
    /// Creates an empty `HashSet`.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    /// let set: HashSet<i32> = HashSet::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `HashSet` with the specified capacity.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    /// let map: HashSet<&str, _> = HashSet::with_capacity(10);
    /// ```
    ///
    /// # Notes
    ///
    /// There is no guarantee that the HashSet will not resize if `capacity`
    /// elements are inserted. The set will resize based on key collision, so
    /// bad key distribution may cause a resize before `capacity` is reached.
    /// For more information see the [`resizing behavior`] of HashMap.
    ///
    /// [`resizing behavior`]: index.html#resizing-behavior
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, crate::DefaultHashBuilder::default())
    }
}

impl<T, S> Default for HashSet<T, S>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<T, S> HashSet<T, S> {
    /// Creates an empty set which will use `hash_builder` to hash values.
    ///
    /// The created set has the default initial capacity.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed to
    /// allow the set to be resistant to attacks that cause many collisions and
    /// very poor performance. Setting it manually using this
    /// function can expose a DoS attack vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::{HashSet, DefaultHashBuilder};
    ///
    /// let set = HashSet::with_hasher(DefaultHashBuilder::default());
    /// let guard = set.guard();
    /// set.insert(1, &guard);
    /// ```
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: HashMap::with_hasher(hash_builder),
        }
    }

    /// Creates an empty set with the specified `capacity`, using `hash_builder` to hash the
    /// values.
    ///
    /// The set will be sized to accommodate `capacity` elements with a low chance of reallocating
    /// (assuming uniformly distributed hashes). If `capacity` is 0, the call will not allocate,
    /// and is equivalent to [`HashSet::new`].
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed to allow the set
    /// to be resistant to attacks that cause many collisions and very poor performance.
    /// Setting it manually using this function can expose a DoS attack vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let set = HashSet::with_capacity_and_hasher(10, s);
    /// let guard = set.guard();
    /// set.insert(1, &guard);
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    /// Pin a `Guard` for use with this set.
    ///
    /// Keep in mind that for as long as you hold onto this `Guard`, you are preventing the
    /// collection of garbage generated by the set.
    pub fn guard(&self) -> Guard<'_> {
        self.map.guard()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    ///
    /// let guard = set.guard();
    /// set.insert(1, &guard);
    /// set.insert(2, &guard);
    /// assert_eq!(set.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set is empty. Otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    /// assert!(set.is_empty());
    /// set.insert("a", &set.guard());
    /// assert!(!set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator visiting all elements in arbitrary order.
    ///
    /// The iterator element type is `&'g T`.
    ///
    /// See [`HashMap::keys`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    /// let guard = set.guard();
    /// set.insert(1, &guard);
    /// set.insert(2, &guard);
    ///
    /// for x in set.iter(&guard) {
    ///     println!("{}", x);
    /// }
    /// ```
    pub fn iter<'g>(&'g self, guard: &'g Guard<'_>) -> Keys<'g, T, ()> {
        self.map.keys(guard)
    }
}

impl<T, S> HashSet<T, S>
where
    T: Hash + Ord,
    S: BuildHasher,
{
    /// Returns `true` if the given value is an element of this set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Ord`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Ord`]: std::cmp::Ord
    /// [`Hash`]: std::hash::Hash
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    /// let guard = set.guard();
    /// set.insert(2, &guard);
    ///
    /// assert!(set.contains(&2, &guard));
    /// assert!(!set.contains(&1, &guard));
    /// ```
    #[inline]
    pub fn contains<'g, Q>(&self, value: &Q, guard: &'g Guard<'_>) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.contains_key(value, guard)
    }

    /// Returns a reference to the element in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Ord`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Ord`]: std::cmp::Ord
    /// [`Hash`]: std::hash::Hash
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set: HashSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let guard = set.guard();
    /// assert_eq!(set.get(&2, &guard), Some(&2));
    /// assert_eq!(set.get(&4, &guard), None);
    /// ```
    pub fn get<'g, Q>(&'g self, value: &Q, guard: &'g Guard<'_>) -> Option<&'g T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.get_key_value(value, guard).map(|(k, _)| k)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    ///
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let a = HashSet::from_iter(&[1, 2, 3]);
    /// let b = HashSet::new();
    ///
    /// assert!(a.pin().is_disjoint(&b.pin()));
    /// b.pin().insert(4);
    /// assert!(a.pin().is_disjoint(&b.pin()));
    /// b.pin().insert(1);
    /// assert!(!a.pin().is_disjoint(&b.pin()));
    ///
    /// ```
    pub fn is_disjoint(
        &self,
        other: &HashSet<T, S>,
        our_guard: &Guard<'_>,
        their_guard: &Guard<'_>,
    ) -> bool {
        for value in self.iter(our_guard) {
            if other.contains(value, their_guard) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if the set is a subset of another, i.e., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let sup = HashSet::from_iter(&[1, 2, 3]);
    /// let set = HashSet::new();
    ///
    /// assert!(set.pin().is_subset(&sup.pin()));
    /// set.pin().insert(2);
    /// assert!(set.pin().is_subset(&sup.pin()));
    /// set.pin().insert(4);
    /// assert!(!set.pin().is_subset(&sup.pin()));
    /// ```
    pub fn is_subset(
        &self,
        other: &HashSet<T, S>,
        our_guard: &Guard<'_>,
        their_guard: &Guard<'_>,
    ) -> bool {
        for value in self.iter(our_guard) {
            if !other.contains(value, their_guard) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if the set is a superset of another, i.e., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let sub = HashSet::from_iter(&[1, 2]);
    /// let set = HashSet::new();
    ///
    /// assert!(!set.pin().is_superset(&sub.pin()));
    ///
    /// set.pin().insert(0);
    /// set.pin().insert(1);
    /// assert!(!set.pin().is_superset(&sub.pin()));
    ///
    /// set.pin().insert(2);
    /// assert!(set.pin().is_superset(&sub.pin()));
    /// ```
    pub fn is_superset(
        &self,
        other: &HashSet<T, S>,
        our_guard: &Guard<'_>,
        their_guard: &Guard<'_>,
    ) -> bool {
        other.is_subset(self, their_guard, our_guard)
    }

    pub(crate) fn guarded_eq(
        &self,
        other: &Self,
        our_guard: &Guard<'_>,
        their_guard: &Guard<'_>,
    ) -> bool {
        self.map.guarded_eq(&other.map, our_guard, their_guard)
    }
}

impl<T, S> HashSet<T, S>
where
    T: Sync + Send + Clone + Hash + Ord,
    S: BuildHasher,
{
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    /// let guard = set.guard();
    ///
    /// assert_eq!(set.insert(2, &guard), true);
    /// assert_eq!(set.insert(2, &guard), false);
    /// assert!(set.contains(&2, &guard));
    /// ```
    pub fn insert(&self, value: T, guard: &Guard<'_>) -> bool {
        let old = self.map.insert(value, (), guard);
        old.is_none()
    }

    /// Removes a value from the set.
    ///
    /// If the set did not have this value present, `false` is returned.
    ///
    /// If the set did have this value present, `true` is returned.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Ord`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Ord`]: std::cmp::Ord
    /// [`Hash`]: std::hash::Hash
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    /// let guard = set.guard();
    /// set.insert(2, &guard);
    ///
    /// assert_eq!(set.remove(&2, &guard), true);
    /// assert!(!set.contains(&2, &guard));
    /// assert_eq!(set.remove(&2, &guard), false);
    /// ```
    pub fn remove<Q>(&self, value: &Q, guard: &Guard<'_>) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        let removed = self.map.remove(value, guard);
        removed.is_some()
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Ord`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// [`Ord`]: std::cmp::Ord
    /// [`Hash`]: std::hash::Hash
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let mut set: HashSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let guard = set.guard();
    /// assert_eq!(set.take(&2, &guard), Some(&2));
    /// assert_eq!(set.take(&2, &guard), None);
    /// ```
    pub fn take<'g, Q>(&'g self, value: &Q, guard: &'g Guard<'_>) -> Option<&'g T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.map.remove_entry(value, guard).map(|(k, _)| k)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    ///
    /// for i in 0..8 {
    ///     set.pin().insert(i);
    /// }
    /// set.pin().retain(|&e| e % 2 == 0);
    /// assert_eq!(set.pin().len(), 4);
    /// ```
    pub fn retain<F>(&self, mut f: F, guard: &Guard<'_>)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain(|value, ()| f(value), guard)
    }
}

impl<T, S> HashSet<T, S>
where
    T: Clone + Ord,
{
    /// Clears the set, removing all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use flurry::HashSet;
    ///
    /// let set = HashSet::new();
    ///
    /// set.pin().insert("a");
    /// set.pin().clear();
    /// assert!(set.pin().is_empty());
    /// ```
    pub fn clear(&self, guard: &Guard<'_>) {
        self.map.clear(guard)
    }

    /// Tries to reserve capacity for at least `additional` more elements to
    /// be inserted in the `HashSet`.
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    pub fn reserve(&self, additional: usize, guard: &Guard<'_>) {
        self.map.reserve(additional, guard)
    }
}

impl<T, S> PartialEq for HashSet<T, S>
where
    T: Ord + Hash,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl<T, S> Eq for HashSet<T, S>
where
    T: Ord + Hash,
    S: BuildHasher,
{
}

impl<T, S> fmt::Debug for HashSet<T, S>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let guard = self.guard();
        f.debug_set().entries(self.iter(&guard)).finish()
    }
}

impl<T, S> Extend<T> for &HashSet<T, S>
where
    T: Sync + Send + Clone + Hash + Ord,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        Extend::extend(&mut &self.map, iter.into_iter().map(|v| (v, ())))
    }
}

impl<'a, T, S> Extend<&'a T> for &HashSet<T, S>
where
    T: Sync + Send + Copy + Hash + Ord,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        Extend::extend(&mut &self.map, iter.into_iter().map(|&v| (v, ())))
    }
}

impl<T, S> FromIterator<T> for HashSet<T, S>
where
    T: Sync + Send + Clone + Hash + Ord,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            map: iter.into_iter().map(|v| (v, ())).collect(),
        }
    }
}

impl<'a, T, S> FromIterator<&'a T> for HashSet<T, S>
where
    T: Sync + Send + Copy + Hash + Ord,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        Self {
            map: iter.into_iter().map(|&v| (v, ())).collect(),
        }
    }
}

impl<T, S> Clone for HashSet<T, S>
where
    T: Sync + Send + Clone + Hash + Ord,
    S: BuildHasher + Clone,
{
    fn clone(&self) -> HashSet<T, S> {
        Self {
            map: self.map.clone(),
        }
    }
}
