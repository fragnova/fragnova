use codec::{Decode, Encode, Compact};
use scale_info::prelude::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub struct ChainTrait(Compact<u32>);

// serde(rename_all = "camelCase") is needed or polkadot.js will not be able to deserialize

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum AudioCategories {
	/// A compressed audio file in the ogg container format
	OggFile,
	///
	/// A compressed audio file in the mp3 format
	Mp3File,
	/// A chainblocks script that returns an effect chain that requires an input, validated
	Effect,
	/// A chainblocks script that returns an instrument chain (no audio input), validated
	Instrument,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum ModelCategories {
	/// A GLTF binary model
	GltfFile,
	/// ???
	Sdf,
	/// A physics collision model
	PhysicsCollider,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum TextureCategories {
	PngFile,
	JpgFile,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum VectorCategories {
	SvgFile,
	FontFile,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum VideoCategories {
	Mp4File,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum TextCategories {
	Plain,
	Json,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum BinaryCategories {
	WasmModule,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum ChainCategories {
	/// A chainblocks script that returns a generic chain (we validate that)
	Generic,
	/// An animation sequence in chainblocks edn
	Animation,
	/// A chainblocks script that returns a shader chain constrained to be a vertex shader (we validate that)
	VertexShader,
	/// A chainblocks script that returns a shader chain constrained to be a fragment shader (we validate that)
	FragmentShader,
	/// A chainblocks script that returns a shader chain constrained to be a compute shader (we validate that)
	ComputeShader,
}

/// Types of categories that can be attached to a Proto-Fragment to describe it (e.g Code, Audio, Video etc.)
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum Categories {
	/// Text of the supported sub-categories
	Text(TextCategories),
	/// Chainblocks chains of various sub-categories
	/// Chains also can have interoperability traits to describe how they can be used in other chains
	Chain(ChainCategories, Vec<ChainTrait>),
	/// Audio files and effects
	Audio(AudioCategories),
	/// Textures of the supported sub-categories
	Texture(TextureCategories),
	/// Vectors of the supported sub-categories (e.g. SVG, Font)
	Vector(VectorCategories),
	/// Video file of the supported formats
	Video(VideoCategories),
	/// 2d/3d models of the supported formats
	Model(ModelCategories),
	/// Binary of the supported sub-categories
	Binary(BinaryCategories),
}
