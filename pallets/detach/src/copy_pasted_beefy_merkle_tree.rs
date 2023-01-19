//! Everything in this crate was copied from: https://crates.io/crates/beefy-merkle-tree/10.0.0

pub use sp_runtime::traits::Keccak256;
use sp_runtime::{app_crypto::sp_core, sp_std, traits::Hash as HashT};
use sp_std::{vec, vec::Vec};

/// A trait of object inspecting merkle root creation.
///
/// It can be passed to [`merkelize_row`] or [`merkelize`] functions and will be notified
/// about tree traversal.
trait Visitor<T> {
	/// We are moving one level up in the tree.
	fn move_up(&mut self);

	/// We are creating an inner node from given `left` and `right` nodes.
	///
	/// Note that in case of last odd node in the row `right` might be empty.
	/// The method will also visit the `root` hash (level 0).
	///
	/// The `index` is an index of `left` item.
	fn visit(&mut self, index: usize, left: &Option<T>, right: &Option<T>);
}

/// No-op implementation of the visitor.
impl<T> Visitor<T> for () {
	fn move_up(&mut self) {}
	fn visit(&mut self, _index: usize, _left: &Option<T>, _right: &Option<T>) {}
}

/// Construct a root hash of a Binary Merkle Tree created from given leaves.
///
/// See crate-level docs for details about Merkle Tree construction.
///
/// In case an empty list of leaves is passed the function returns a 0-filled hash.
pub fn merkle_root<H, I>(leaves: I) -> H::Output
	where
		H: HashT,
		H::Output: Default + AsRef<[u8]> + PartialOrd,
		I: IntoIterator,
		I::Item: AsRef<[u8]>,
{
	let iter = leaves.into_iter().map(|l| <H as HashT>::hash(l.as_ref()));
	merkelize::<H, _, _>(iter, &mut ()).into()
}

fn merkelize<H, V, I>(leaves: I, visitor: &mut V) -> H::Output
	where
		H: HashT,
		H::Output: Default + AsRef<[u8]> + PartialOrd,
		V: Visitor<H::Output>,
		I: Iterator<Item = H::Output>,
{
	let upper = Vec::with_capacity((leaves.size_hint().1.unwrap_or(0).saturating_add(1)) / 2);
	let mut next = match merkelize_row::<H, _, _>(leaves, upper, visitor) {
		Ok(root) => return root,
		Err(next) if next.is_empty() => return H::Output::default(),
		Err(next) => next,
	};

	let mut upper = Vec::with_capacity((next.len().saturating_add(1)) / 2);
	loop {
		visitor.move_up();

		match merkelize_row::<H, _, _>(next.drain(..), upper, visitor) {
			Ok(root) => return root,
			Err(t) => {
				// swap collections to avoid allocations
				upper = next;
				next = t;
			},
		};
	}
}

/// Processes a single row (layer) of a tree by taking pairs of elements,
/// concatenating them, hashing and placing into resulting vector.
///
/// In case only one element is provided it is returned via `Ok` result, in any other case (also an
/// empty iterator) an `Err` with the inner nodes of upper layer is returned.
fn merkelize_row<H, V, I>(
	mut iter: I,
	mut next: Vec<H::Output>,
	visitor: &mut V,
) -> Result<H::Output, Vec<H::Output>>
	where
		H: HashT,
		H::Output: AsRef<[u8]> + PartialOrd,
		V: Visitor<H::Output>,
		I: Iterator<Item = H::Output>,
{
	#[cfg(feature = "debug")]
	log::debug!("[merkelize_row]");
	next.clear();

	let hash_len = <H as sp_core::Hasher>::LENGTH;
	let mut index = 0;
	let mut combined = vec![0_u8; hash_len * 2];
	loop {
		let a = iter.next();
		let b = iter.next();
		visitor.visit(index, &a, &b);

		#[cfg(feature = "debug")]
		log::debug!(
			"  {:?}\n  {:?}",
			a.as_ref().map(|s| array_bytes::bytes2hex("", s.as_ref())),
			b.as_ref().map(|s| array_bytes::bytes2hex("", s.as_ref()))
		);

		index += 2;
		match (a, b) {
			(Some(a), Some(b)) => {
				if a < b {
					combined[..hash_len].copy_from_slice(a.as_ref());
					combined[hash_len..].copy_from_slice(b.as_ref());
				} else {
					combined[..hash_len].copy_from_slice(b.as_ref());
					combined[hash_len..].copy_from_slice(a.as_ref());
				}

				next.push(<H as HashT>::hash(&combined));
			},
			// Odd number of items. Promote the item to the upper layer.
			(Some(a), None) if !next.is_empty() => {
				next.push(a);
			},
			// Last item = root.
			(Some(a), None) => return Ok(a),
			// Finish up, no more items.
			_ => {
				#[cfg(feature = "debug")]
				log::debug!(
					"[merkelize_row] Next: {:?}",
					next.iter().map(|s| array_bytes::bytes2hex("", s.as_ref())).collect::<Vec<_>>()
				);
				return Err(next)
			},
		}
	}
}
