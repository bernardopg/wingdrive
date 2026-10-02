use crate::Error;

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use zeroize::{Zeroize, Zeroizing};

/// This RNG should be used throughout the entire crate.
///
/// On `Drop`, it re-seeds the inner RNG, erasing the previous state and making all future
/// values unpredictable.
#[derive(Debug, Clone)]
pub struct CryptoRng(ChaCha20Rng);

impl CryptoRng {
	/// This creates a new [`ChaCha20Rng`]-backed [`rand::CryptoRng`] from entropy
	/// (via the [getrandom](https://docs.rs/getrandom) crate).
	#[inline]
	pub fn new() -> Result<Self, Error> {
		// rand 0.10 removed `try_from_os_rng`; seed from the OS-backed SysRng
		// via the fallible `try_from_rng` path instead.
		ChaCha20Rng::try_from_rng(&mut rand::rngs::SysRng)
			.map(Self)
			.map_err(|_| Error::Encrypt) // Convert getrandom error to our error type
	}

	/// Used to generate completely random bytes, with the use of [`ChaCha20Rng`]
	///
	/// Ideally this should be used for small amounts only (as it's stack allocated)
	#[inline]
	#[must_use]
	pub fn generate_fixed<const I: usize>(&mut self) -> [u8; I] {
		let mut bytes = Zeroizing::new([0u8; I]);
		self.fill_bytes(bytes.as_mut());
		*bytes
	}

	/// Used to generate completely random bytes, with the use of [`ChaCha20Rng`]
	#[inline]
	#[must_use]
	pub fn generate_vec(&mut self, size: usize) -> Vec<u8> {
		let mut bytes = vec![0u8; size];
		self.fill_bytes(bytes.as_mut());
		bytes
	}
}

impl rand::TryRng for CryptoRng {
	type Error = core::convert::Infallible;

	#[inline]
	fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
		Ok(self.0.next_u32())
	}

	#[inline]
	fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
		Ok(self.0.next_u64())
	}

	#[inline]
	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
		self.0.fill_bytes(dest);
		Ok(())
	}
}

impl rand::TryCryptoRng for CryptoRng {}

impl SeedableRng for CryptoRng {
	type Seed = <ChaCha20Rng as SeedableRng>::Seed;

	fn from_seed(seed: Self::Seed) -> Self {
		Self(ChaCha20Rng::from_seed(seed))
	}
}

impl Zeroize for CryptoRng {
	#[inline]
	fn zeroize(&mut self) {
		let mut seed = <Self as SeedableRng>::Seed::default();
		self.0.fill_bytes(&mut seed);

		self.0 = ChaCha20Rng::from_seed(seed);
	}
}

impl Drop for CryptoRng {
	#[inline]
	fn drop(&mut self) {
		self.zeroize();
	}
}

// implementing old-rand-core traits for compatibility with old code
impl old_rand_core::CryptoRng for CryptoRng {}

impl old_rand_core::RngCore for CryptoRng {
	fn next_u32(&mut self) -> u32 {
		<Self as Rng>::next_u32(self)
	}

	fn next_u64(&mut self) -> u64 {
		<Self as Rng>::next_u64(self)
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		<Self as Rng>::fill_bytes(self, dest);
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), old_rand_core::Error> {
		<Self as Rng>::fill_bytes(self, dest);
		Ok(())
	}
}
