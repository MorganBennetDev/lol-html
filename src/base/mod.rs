#[macro_use]
mod debug_trace;

mod align;
mod bytes;
mod encoding;
mod range;
mod spanned;

use rustc_hash::FxBuildHasher;
pub(crate) use self::align::Align;
pub(crate) use self::bytes::{Bytes, BytesCow, HasReplacementsError};
pub use self::encoding::SharedEncoding;
pub(crate) use self::range::Range;
pub use self::spanned::SourceLocation;
pub(crate) use self::spanned::{Spanned, SpannedRawBytes};

type CrateDefaultHasher = FxBuildHasher;
pub(crate) type HashSet<V, S = CrateDefaultHasher> = std::collections::HashSet<V, S>;
pub(crate) type HashMap<K, V, S = CrateDefaultHasher> = std::collections::HashMap<K, V, S>;